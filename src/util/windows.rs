mod cloud_filter;
mod dynamic_buf_ptr;
pub(crate) mod prelude;
mod ptr;
mod strings;
pub(crate) use dynamic_buf_ptr::*;
use prelude::*;
pub(crate) use ptr::{FromVoid, ToBytes};
pub(crate) use strings::*;

use anyhow::{Context, Result};
use windows::Win32::{
    Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER},
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

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
