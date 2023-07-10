#![allow(unused_imports)]
pub(crate) use windows::{
    core::{Error as WinError, Result as WinResult, HSTRING, PCWSTR, PWSTR},
    h,
    Win32::Foundation::{GetLastError, BOOL, HANDLE},
};

pub(crate) use super::strings::*;
pub(crate) use widestring::{U16CStr, U16CString};
