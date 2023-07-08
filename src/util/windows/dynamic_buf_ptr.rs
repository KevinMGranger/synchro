use windows::Win32::Foundation::{GetLastError, ERROR_INSUFFICIENT_BUFFER};

use super::prelude::*;
use std::{
    ffi::c_void,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// A struct that is dynamically sized from an API call.
///
/// Several Windows APIs will dynamically fill in a byte
/// buffer with a different struct's data, depending
/// upon some dynamic type argument.
/// This is a safe(ish) abstraction over such APIs.
pub(crate) struct DynamicBufPtr<T> {
    buf: Vec<u8>,
    r#type: PhantomData<T>,
}

impl<T> DynamicBufPtr<T> {
    /// Provide a closure over the API which takes:
    /// - the pointer to the buffer to fill in
    /// - the size of that buffer
    /// - a pointer to the "required size" parameter
    ///
    /// ...and the DynamicBufPtr will call the API
    /// with a null pointer to find the necessary size,
    /// allocate that memory, and then call it again to be filled.
    ///
    /// Any errors from the first call _except_ ERROR_INSUFFICIENT_BUFFER
    /// will be returned, as will any errors from the second call.
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
