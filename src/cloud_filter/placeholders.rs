use std::fs::Metadata;
use std::os::windows::fs::MetadataExt;
use std::ptr;

use crate::util::windows::{prelude::*, OwnedWSTR};
use windows::core::{HRESULT, PCWSTR};
use windows::Win32::Storage::CloudFilters::{
    CF_FS_METADATA, CF_PLACEHOLDER_CREATE_FLAG_NONE, CF_PLACEHOLDER_CREATE_INFO,
};
use windows::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_NORMAL, FILE_BASIC_INFO};

const FILE_CONTENTS: &str = "Hello, world!\n";

pub(crate) fn single_file_placeholder(relative_name: PCWSTR) -> CF_PLACEHOLDER_CREATE_INFO {
    CF_PLACEHOLDER_CREATE_INFO {
        RelativeFileName: relative_name,
        FsMetadata: CF_FS_METADATA {
            BasicInfo: FILE_BASIC_INFO {
                CreationTime: 0,
                LastAccessTime: 0,
                LastWriteTime: 0,
                ChangeTime: 0,
                FileAttributes: FILE_ATTRIBUTE_NORMAL.0,
            },
            FileSize: FILE_CONTENTS.len() as i64,
        },
        FileIdentity: ptr::null(),
        FileIdentityLength: 0,
        Flags: CF_PLACEHOLDER_CREATE_FLAG_NONE,
        Result: HRESULT::default(),
        CreateUsn: 0,
    }
}

pub(crate) struct PlaceholderCreateInfo<'a> {
    relative_file_name: &'a OwnedWSTR,
}

pub(crate) struct PlaceholderResults {
    processed: u32,
}

// TODO: PCWSTR is some bullshit. Maybe create a safer abstraction over this.
// Could even just _not_ allow the struct, and just give back results correlated with _safe_ strings.

// pub(crate) fn create_placeholder(base_path: PCWSTR, relative_name: PCWSTR) -> WinResult<>
