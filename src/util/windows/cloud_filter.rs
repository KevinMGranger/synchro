use std::fs::Metadata;
use std::os::windows::fs::MetadataExt;
use std::ptr;

use windows::core::{HRESULT, PCWSTR};
use windows::Win32::Storage::CloudFilters::{
    CF_FS_METADATA, CF_PLACEHOLDER_CREATE_FLAG_NONE, CF_PLACEHOLDER_CREATE_INFO,
};
use windows::Win32::Storage::FileSystem::FILE_BASIC_INFO;

pub(crate) fn metadata_to_file_basic_info(meta: &Metadata) -> FILE_BASIC_INFO {
    FILE_BASIC_INFO {
        CreationTime: meta.creation_time() as i64,
        LastAccessTime: meta.last_access_time() as i64,
        LastWriteTime: meta.last_write_time() as i64,
        ChangeTime: meta.last_write_time() as i64,
        FileAttributes: meta.file_attributes(),
    }
}

pub(crate) fn metadata_to_placeholder(
    relative_name: PCWSTR,
    meta: &Metadata,
) -> CF_PLACEHOLDER_CREATE_INFO {
    CF_PLACEHOLDER_CREATE_INFO {
        RelativeFileName: relative_name,
        FsMetadata: CF_FS_METADATA {
            BasicInfo: metadata_to_file_basic_info(meta),
            FileSize: meta.file_size() as i64,
        },
        FileIdentity: ptr::null(),
        FileIdentityLength: 0,
        Flags: CF_PLACEHOLDER_CREATE_FLAG_NONE,
        Result: HRESULT::default(),
        CreateUsn: 0,
    }
}
