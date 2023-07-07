// use windows::core::Result;
// use windows::Win32::Foundation::BOOL;
use std::{
    ffi::{c_void, OsString},
    marker::PhantomData,
    mem::transmute,
    ops::{Deref, DerefMut},
    os::windows::prelude::OsStringExt,
};
use windows::{
    core::{Error as WinError, Result as WinResult, PWSTR},
    Win32::{
        Foundation::{GetLastError, BOOL, ERROR_INSUFFICIENT_BUFFER},
        System::Memory::LocalFree,
    },
};

pub(crate) struct DynamicBufPtr<T> {
    buf: Vec<u8>,
    r#type: PhantomData<T>,
}

impl<T> DynamicBufPtr<T> {
    pub(crate) fn new(
        mut f: impl FnMut(Option<*mut c_void>, u32, *mut u32) -> BOOL,
    ) -> WinResult<Self> {
        let mut len = 0;

        if !(f)(None, 0, &mut len).as_bool() {
            let last_err = unsafe { GetLastError() };
            if last_err != ERROR_INSUFFICIENT_BUFFER {
                return Err(WinError::from(last_err));
            }
        }

        let mut buf: Vec<u8> = Vec::with_capacity(len as usize);

        (f)(Some(buf.as_mut_ptr() as *mut c_void), len, &mut len).ok()?;

        Ok(Self {
            buf,
            r#type: PhantomData,
        })
    }
}

impl<T> AsRef<T> for DynamicBufPtr<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*(self.buf.as_ptr() as *const T) }
    }
}

impl<T> AsMut<T> for DynamicBufPtr<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.buf.as_mut_ptr() as *mut T) }
    }
}

impl<T> Deref for DynamicBufPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for DynamicBufPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

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

impl Into<OsString> for LocalPWSTR {
    fn into(self) -> OsString {
        OsStringExt::from_wide(unsafe { self.0.as_wide() })
    }
}
