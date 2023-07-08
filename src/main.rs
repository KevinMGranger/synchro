mod cloud_filter;
mod util;

use std::{
    ffi::{OsStr, OsString},
    ptr,
    thread::sleep,
    time::Duration,
};
use tap::Conv;

use crate::cloud_filter::{callbacks::*, sync_root::*};
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

    // Ok(())
}
