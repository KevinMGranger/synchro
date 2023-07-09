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

pub(crate) struct SyncRootInfo<T> {
    pub(crate) file_id: i64,
    identity: *const c_void,
    identity_length: u32,
    identity_type: PhantomData<T>,
}

impl<T> SyncRootInfo<T>
where
    T: FromVoid,
{
    fn identity(&self) -> T {
        unsafe {
            let slice = std::slice::from_raw_parts(self.identity, self.identity_length as usize);
            T::from_void_slice(slice)
        }
    }
}

pub(crate) struct CallbackInfo<T> {
    connection_key: CF_CONNECTION_KEY,
    context: *mut c_void, // TODO keep private

    _d: PhantomData<T>,
}
