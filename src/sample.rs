use anyhow::{Context, Result};
use std::borrow::Cow;
use std::ffi::c_void;
use std::fs::Metadata;
use std::os::windows::fs::MetadataExt;
use std::os::windows::prelude::OsStrExt;
use std::path::Path;
use std::{fmt, ptr};
use tap::Conv;

use crate::util::windows::{prelude::*, OwnedWSTR, ToBytes};
use windows::core::{HRESULT, PCWSTR};
use windows::Win32::Storage::CloudFilters::{
    CfCreatePlaceholders, CF_CREATE_FLAGS, CF_FS_METADATA, CF_PLACEHOLDER_CREATE_FLAGS,
    CF_PLACEHOLDER_CREATE_FLAG_NONE, CF_PLACEHOLDER_CREATE_INFO,
    CF_PLACEHOLDER_MAX_FILE_IDENTITY_LENGTH,
};
use windows::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_NORMAL, FILE_BASIC_INFO};

pub(crate) struct TestName(pub(crate) String);

impl fmt::Display for TestName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ToBytes for TestName {
    fn to_bytes<'bytes, 'this: 'bytes>(&'this self) -> Cow<'bytes, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }
}

const FILE_CONTENTS: &str = "Hello, world!\n";

// pub(crate) fn single_file_placeholder(relative_name: PCWSTR) -> CF_PLACEHOLDER_CREATE_INFO {
//     CF_PLACEHOLDER_CREATE_INFO {
//         RelativeFileName: relative_name,
//         FsMetadata: CF_FS_METADATA {
//             BasicInfo: FILE_BASIC_INFO {
//                 CreationTime: 0,
//                 LastAccessTime: 0,
//                 LastWriteTime: 0,
//                 ChangeTime: 0,
//                 FileAttributes: FILE_ATTRIBUTE_NORMAL.0,
//             },
//             FileSize: FILE_CONTENTS.len() as i64,
//         },
//         FileIdentity: ptr::null(),
//         FileIdentityLength: 0,
//         Flags: CF_PLACEHOLDER_CREATE_FLAG_NONE,
//         Result: HRESULT::default(),
//         CreateUsn: 0,
//     }
// }
