pub(crate) use windows::{
    core::{Error as WinError, Result as WinResult, HSTRING, PCWSTR, PWSTR},
    h,
    Win32::Foundation::{GetLastError, BOOL, HANDLE},
};
