use windows::Win32::Storage::CloudFilters::{
    CfCloseHandle, CfOpenFileWithOplock, CF_OPEN_FILE_FLAGS,
};

use crate::util::windows::prelude::*;

pub(crate) struct CfFileHandle(HANDLE);

impl CfFileHandle {
    pub(crate) fn open_with_oplock(path: &U16CStr, flags: CF_OPEN_FILE_FLAGS) -> WinResult<Self> {
        unsafe { CfOpenFileWithOplock(PCWSTR(path.as_ptr()), flags) }.map(Self)
    }
}

impl Drop for CfFileHandle {
    fn drop(&mut self) {
        unsafe { CfCloseHandle(self.0) }
    }
}

// TODO: similar for CfGetTransferKey