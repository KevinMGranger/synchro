mod util;
use std::{
    ffi::{OsStr, OsString},
    ptr,
    thread::sleep,
    time::Duration,
};
use tap::Conv;

use anyhow::{Context, Result};
use clap::Parser;
use util::windows::{DynamicBufPtr, LocalPWSTR};
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
            CF_CONNECTION_KEY, CF_CONNECT_FLAG_NONE, CF_FS_METADATA, CF_PLACEHOLDER_CREATE_INFO,
            CF_PLACEHOLDER_MAX_FILE_IDENTITY_LENGTH,
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

fn get_token_user() -> Result<DynamicBufPtr<TOKEN_USER>> {
    let mut process_token = HANDLE::default();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut process_token) }
        .ok()
        .context("OpenprocessToken")?;

    DynamicBufPtr::new(|ptr, size, ret_len| unsafe {
        GetTokenInformation(process_token, TokenUser, ptr, size, ret_len)
    })
    .context("GetTokenInformation")
}

const STORAGE_PROVIDER_ID: &str = "TestStorageProvider";
const STORAGE_PROVIDER_ACCOUNT: &str = "TestAccount1";

// TODO: check how thread info works in addition to process token--
// never really used impersonation before.
fn get_sync_root_id() -> Result<OsString> {
    let user_token = get_token_user().context("get_token_user")?;
    let sid = user_token.User.Sid;
    let sid_str: OsString =
        LocalPWSTR::new(|ptr| unsafe { ConvertSidToStringSidW(sid, ptr) })?.into();

    // todo: reserve
    // todo: link to specification for this format
    let mut id = OsString::from(STORAGE_PROVIDER_ID);
    id.push("!");
    id.push(sid_str);
    id.push("!");
    id.push(STORAGE_PROVIDER_ACCOUNT);

    Ok(id)
}

fn register_sync_root(client_dir: &HSTRING) -> Result<()> {
    let sync_root_id = get_sync_root_id()?;
    let info = StorageProviderSyncRootInfo::new()?;
    info.SetId(&sync_root_id.conv::<HSTRING>())
        .context("SetId")?;

    // TODO: must be absolute, whoops
    let folder = StorageFolder::GetFolderFromPathAsync(client_dir)
        .context("ggetfolpath")?
        .get()
        .context("get")?;

    info.SetPath(&folder).context("SetPath")?;

    info.SetDisplayNameResource(&"CloudMirror".into())
        .context("SetDisplayNameResource")?;

    info.SetIconResource(h!("%SystemRoot%\\system32\\charmap.exe,0"))
        .context("SetIconResource")?;
    info.SetHydrationPolicy(StorageProviderHydrationPolicy::Full)
        .context("SetHydrationPolicy")?;
    info.SetHydrationPolicyModifier(StorageProviderHydrationPolicyModifier::None)
        .context("SetHydrationPolicyModifier")?;
    info.SetPopulationPolicy(StorageProviderPopulationPolicy::AlwaysFull)
        .context("SetPopulationPolicy")?;
    info.SetInSyncPolicy(
        StorageProviderInSyncPolicy::FileCreationTime
            | StorageProviderInSyncPolicy::DirectoryCreationTime,
    )
    .context("SetInSyncPolicy")?;
    info.SetVersion(h!("1.0.0")).context("SetVersion")?;
    info.SetShowSiblingsAsGroup(false)
        .context("SetShowSiblingsAsGroup")?;
    info.SetHardlinkPolicy(StorageProviderHardlinkPolicy::None)
        .context("SetHardlinkPolicy")?;

    let uri =
        Uri::CreateUri(h!("http://cloudmirror.example.com/recyclebin")).context("create_uri")?;
    info.SetRecycleBinUri(&uri).context("SetRecycleBinUri")?;

    // no context field for some reason?

    // skipping custom states

    Ok(StorageProviderSyncRootManager::Register(&info).context("register")?)
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

fn main() -> Result<()> {
    let args = Args::parse();
    register_sync_root(&args.sync_root_path)
        .context("reg_sync_r")
        .unwrap();
    unsafe { connect_callbacks(&args.sync_root_path) }.unwrap();

    loop {
        sleep(Duration::MAX);
    }

    Ok(())
}
