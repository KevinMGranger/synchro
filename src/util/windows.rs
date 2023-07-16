//! Utilities that make working with Windows FFI easier.
mod cloud_filter;
mod dynamic_buf_ptr;
pub(crate) mod prelude;
mod strings;
pub(crate) use dynamic_buf_ptr::*;
use prelude::*;
pub(crate) use strings::*;

use anyhow::{Context, Result};
use windows::Win32::{
    Foundation::CloseHandle,
    Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER},
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

/// Get the token for the user currently associated with this process.
pub(crate) fn get_token_user() -> Result<DynamicBufPtr<TOKEN_USER>> {
    let mut process_token = HANDLE::default();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut process_token) }
        .ok()
        .context("OpenprocessToken")?;

    DynamicBufPtr::new(|ptr, size, ret_len| unsafe {
        GetTokenInformation(process_token, TokenUser, ptr, size, ret_len)
    })
    .context("GetTokenInformation")
}

pub(crate) struct FileHandle(HANDLE);

impl From<HANDLE> for FileHandle {
    fn from(value: HANDLE) -> Self {
        Self(value)
    }
}

impl AsRef<HANDLE> for FileHandle {
    fn as_ref(&self) -> &HANDLE {
        &self.0
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}

impl From<&FileHandle> for HANDLE {
    fn from(value: &FileHandle) -> Self {
        value.0
    }
}
