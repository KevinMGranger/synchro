use std::fs::Metadata;
use std::os::windows::fs::MetadataExt;
use std::ptr;

use crate::util::windows::prelude::*;
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

// TODO: PCWSTR is some bullshit so we're going to just leak memory for now.

pub(crate) fn create_placeholder(base_path: PCWSTR, relative_name: PCWSTR)