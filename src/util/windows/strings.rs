use super::prelude::*;
use std::ffi::{OsStr, OsString};
use std::os::windows::prelude::{OsStrExt, OsStringExt};
use std::{mem::transmute, ops::Deref};
use windows::Win32::System::Memory::LocalFree;

/// A wrapper over PWSTRs that are meant to be
/// freed with `LocalFree`.
pub(crate) struct LocalPWSTR(PWSTR);

impl Drop for LocalPWSTR {
    fn drop(&mut self) {
        // sure hope this is safe, this is confusing
        let asi: isize = unsafe { transmute(self.0) };
        unsafe { LocalFree(asi) };
    }
}

impl LocalPWSTR {
    pub(crate) fn new(f: impl FnOnce(*mut PWSTR) -> BOOL) -> WinResult<Self> {
        let mut pwstr = PWSTR::null();
        (f)(&mut pwstr).ok()?;

        Ok(Self(pwstr))
    }
}

impl Deref for LocalPWSTR {
    type Target = PWSTR;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<LocalPWSTR> for OsString {
    fn from(value: LocalPWSTR) -> Self {
        OsStringExt::from_wide(unsafe { value.0.as_wide() })
    }
}

pub(crate) struct OwnedWSTR {
    buf: Vec<u16>,
}

impl OwnedWSTR {
    pub(crate) unsafe fn loan_ptr(&self) -> *const u16 {
        self.buf.as_ptr()
    }

    pub(crate) unsafe fn loan_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.loan_ptr())
    }
}

impl From<&OsStr> for OwnedWSTR {
    fn from(value: &OsStr) -> Self {
        let mut buf = value.encode_wide().collect::<Vec<_>>();
        buf.push(0);

        Self { buf }
    }
}

impl From<&str> for OwnedWSTR {
    fn from(value: &str) -> Self {
        let mut buf = value.encode_utf16().collect::<Vec<_>>();
        buf.push(0);

        Self { buf }
    }
}
