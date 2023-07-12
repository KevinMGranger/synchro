pub(crate) mod structures;
pub(crate) use self::structures::*;

use crate::util::windows::prelude::*;

use windows::Win32::Storage::CloudFilters::CF_CONNECT_FLAG_REQUIRE_FULL_FILE_PATH;
use windows::{
    core::{Result as WinResult, PCWSTR},
    Win32::Storage::CloudFilters::{
        CfConnectSyncRoot, CF_CALLBACK_REGISTRATION, CF_CONNECTION_KEY,
    },
};

// TODO: how do we properly handle the lifecycle of the callbacktable?
// and how do we

/// Connect the sync root to its related callbacks.
pub(crate) unsafe fn connect_callbacks(
    client_dir: &U16CStr,
    callback_table: &'static [CF_CALLBACK_REGISTRATION],
) -> WinResult<CF_CONNECTION_KEY> {
    CfConnectSyncRoot(
        PCWSTR(client_dir.as_ptr()),
        callback_table.as_ptr(),
        None,
        // CF_CONNECT_FLAG_NONE,
        CF_CONNECT_FLAG_REQUIRE_FULL_FILE_PATH,
    )
}
