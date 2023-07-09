use std::{ffi::c_void, marker::PhantomData};

use crate::util::windows::prelude::*;

use windows::{
    core::{Result as WinResult, PCWSTR},
    Win32::Storage::CloudFilters::{
        CfConnectSyncRoot, CF_CALLBACK_INFO, CF_CALLBACK_PARAMETERS, CF_CALLBACK_REGISTRATION,
        CF_CALLBACK_TYPE_FETCH_DATA, CF_CALLBACK_TYPE_NONE, CF_CONNECTION_KEY,
        CF_CONNECT_FLAG_NONE,
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

pub(crate) struct CallbackVolumeInfo {
    guid_name: PCWSTR,
    dos_name: PCWSTR,
    pub(crate) serial_number: u32,
}

impl CallbackVolumeInfo {
    fn guid_name(&self) -> TrustedBorrowedCWSTR<'_> {
        unsafe { TrustedBorrowedCWSTR::from_raw(self.guid_name) }
    }

    fn dos_name(&self) -> TrustedBorrowedCWSTR<'_> {
        unsafe { TrustedBorrowedCWSTR::from_raw(self.dos_name) }
    }
}

pub(crate) struct SyncRootInfo<'a> {
    pub(crate) file_id: i64,
    pub(crate) identity: &'a [c_void],
}

pub(crate) struct CallbackInfo<'a> {
    connection_key: CF_CONNECTION_KEY,
    // context: *mut c_void, // TODO keep private? seems like it doesn't work anyway
    volume_info: CallbackVolumeInfo,
    sync_root_info: SyncRootInfo<'a>,
    file_id: i64,
    file_size: i64,
    file_identity: &'a [c_void],
    normalized_path: TrustedBorrowedCWSTR<'a>,
    transfer_key: i64,
    priority_hint: u8,
    // correlation_vector: *mut CORRELATION_VECTOR, // TODO
    // process_info: *mut CF_PROCESS_INFO, // TODO
    request_key: i64,
}

impl<'a> From<CF_CALLBACK_INFO> for CallbackInfo<'a> {
    fn from(value: CF_CALLBACK_INFO) -> Self {
        Self {
            connection_key: value.ConnectionKey,
            volume_info: unsafe {
                CallbackVolumeInfo {
                    guid_name: TrustedBorrowedCWSTR::from_raw(value.VolumeGuidName),
                    dos_name: TrustedBorrowedCWSTR::from_raw(value.VolumeDosName),
                    serial_number: value.VolumeSerialNumber,
                }
            },
        }
    }
}
