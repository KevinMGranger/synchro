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
