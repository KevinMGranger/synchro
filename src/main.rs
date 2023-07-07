mod util;
use std::{
    ffi::{OsStr, OsString},
    ptr,
};
use tap::Conv;

use clap::Parser;
use util::{DynamicBufPtr, LocalPWSTR};
use windows::{
    core::{Result as WinResult, HSTRING, PCWSTR},
    h,
    Foundation::Uri,
    Storage::{
        Provider::{
            StorageProviderHardlinkPolicy, StorageProviderHydrationPolicy,
            StorageProviderHydrationPolicyModifier, StorageProviderInSyncPolicy,
            StorageProviderPopulationPolicy, StorageProviderSyncRootInfo,
            StorageProviderSyncRootManager,
        },
        StorageFolder,
    },
    Win32::{
        Foundation::HANDLE,
        Security::{
            Authorization::ConvertSidToStringSidW, GetTokenInformation, TokenUser, TOKEN_QUERY,
            TOKEN_USER,
        },
        Storage::CloudFilters::{
            CfConnectSyncRoot, CF_CALLBACK, CF_CALLBACK_INFO, CF_CALLBACK_PARAMETERS,
            CF_CALLBACK_REGISTRATION, CF_CALLBACK_TYPE_FETCH_DATA, CF_CALLBACK_TYPE_NONE,
            CF_CONNECTION_KEY, CF_CONNECT_FLAG_NONE,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    },
};

extern "system" fn callback_test_fetch(
    callbackinfo: *const CF_CALLBACK_INFO,
    callbackparams: *const CF_CALLBACK_PARAMETERS,
) {
    let callback_info = unsafe { &*callbackinfo };
    let fetch_info = unsafe { (*callbackparams).Anonymous.FetchData };
    println!("{callback_info:?}");
    println!("{fetch_info:?}");
}

const CALLBACK_TABLE: &[CF_CALLBACK_REGISTRATION] = &[
    CF_CALLBACK_REGISTRATION {
        Type: CF_CALLBACK_TYPE_FETCH_DATA,
        Callback: Some(callback_test_fetch),
    },
    CF_CALLBACK_REGISTRATION {
        Type: CF_CALLBACK_TYPE_NONE,
        Callback: None,
    },
];

unsafe fn get_token_user() -> WinResult<DynamicBufPtr<TOKEN_USER>> {
    let mut process_token = HANDLE::default();
    OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut process_token).ok()?;

    DynamicBufPtr::new(|ptr, size, ret_len| {
        GetTokenInformation(process_token, TokenUser, ptr, size, ret_len)
    })
}

const STORAGE_PROVIDER_ID: &str = "TestStorageProvider";
const STORAGE_PROVIDER_ACCOUNT: &str = "TestAccount1";

// TODO: check how thread info works in addition to process token--
// never really used impersonation before.
unsafe fn get_sync_root_id() -> WinResult<OsString> {
    let user_token = get_token_user()?;
    let sid = user_token.User.Sid;
    let sid_str: OsString = LocalPWSTR::new(|ptr| ConvertSidToStringSidW(sid, ptr))?.into();

    // todo: reserve
    // todo: link to specification for this format
    let mut id = OsString::from(STORAGE_PROVIDER_ID);
    id.push("!");
    id.push(sid_str);
    id.push("!");
    id.push(STORAGE_PROVIDER_ACCOUNT);

    Ok(id)
}

// TODO: lifecycle of PCWSTR also?
unsafe fn register_sync_root(client_dir: &HSTRING) -> WinResult<()> {
    let info = StorageProviderSyncRootInfo::new()?;
    info.SetId(&get_sync_root_id()?.conv::<HSTRING>())?;

    let folder = StorageFolder::GetFolderFromPathAsync(client_dir)?.get()?;

    info.SetPath(&folder)?;

    info.SetDisplayNameResource(&"CloudMirror".into())?;

    info.SetIconResource(h!("%SystemRoot%\\system32\\charmap.exe,0"))?;
    info.SetHydrationPolicy(StorageProviderHydrationPolicy::Full)?;
    info.SetHydrationPolicyModifier(StorageProviderHydrationPolicyModifier::None)?;
    info.SetPopulationPolicy(StorageProviderPopulationPolicy::AlwaysFull)?;
    info.SetInSyncPolicy(
        StorageProviderInSyncPolicy::FileCreationTime
            | StorageProviderInSyncPolicy::DirectoryCreationTime,
    )?;
    info.SetVersion(h!("1.0.0"))?;
    info.SetShowSiblingsAsGroup(false)?;
    info.SetHardlinkPolicy(StorageProviderHardlinkPolicy::None)?;

    info.SetRecycleBinUri(&Uri::CreateUri(h!(
        "http://cloudmirror.example.com/recyclebin"
    ))?)?;

    // no context field for some reason?

    // skipping custom states

    StorageProviderSyncRootManager::Register(&info)
}

// TODO: how do we properly handle the lifecycle of the callbacktable?

unsafe fn connect_callbacks(client_dir: impl Into<PCWSTR>) -> WinResult<CF_CONNECTION_KEY> {
    CfConnectSyncRoot(
        client_dir,
        CALLBACK_TABLE.as_ptr(),
        None,
        CF_CONNECT_FLAG_NONE,
    )
}

#[derive(Parser, Debug)]
#[command()]
struct Args {
    sync_root_path: HSTRING,
}

fn main() {
    let args = Args::parse();
    unsafe { connect_callbacks(&args.sync_root_path) }.unwrap();
}
