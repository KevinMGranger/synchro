use super::prelude::*;

use std::ffi::OsString;

use std::os::windows::prelude::OsStringExt;
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

pub(crate) fn u16cstr_from_hstring(h: &HSTRING) -> &U16CStr {
    unsafe { U16CStr::from_ptr_unchecked(h.as_ptr(), h.len()) }
}
