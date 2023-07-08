use std::{
    ffi::{OsStr, OsString},
    ptr,
    thread::sleep,
    time::Duration,
};
use tap::Conv;

use crate::util::windows::{get_token_user, DynamicBufPtr, LocalPWSTR};
use anyhow::{Context, Result};
use clap::Parser;
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

pub(crate) extern "system" fn callback_test_fetch(
    callbackinfo: *const CF_CALLBACK_INFO,
    callbackparams: *const CF_CALLBACK_PARAMETERS,
) {
    let callback_info = unsafe { &*callbackinfo };
    let fetch_info = unsafe { (*callbackparams).Anonymous.FetchData };
    println!("{callback_info:?}");
    println!("{fetch_info:?}");
}

pub(crate) const CALLBACK_TABLE: &[CF_CALLBACK_REGISTRATION] = &[
    CF_CALLBACK_REGISTRATION {
        Type: CF_CALLBACK_TYPE_FETCH_DATA,
        Callback: Some(callback_test_fetch),
    },
    CF_CALLBACK_REGISTRATION {
        Type: CF_CALLBACK_TYPE_NONE,
        Callback: None,
    },
];

// TODO: how do we properly handle the lifecycle of the callbacktable?

pub(crate) unsafe fn connect_callbacks(
    client_dir: impl Into<PCWSTR>,
) -> WinResult<CF_CONNECTION_KEY> {
    CfConnectSyncRoot(
        client_dir,
        CALLBACK_TABLE.as_ptr(),
        None,
        CF_CONNECT_FLAG_NONE,
    )
}
