use anyhow::{Context, Result};
use std::ffi::c_void;





use tap::Conv;

use crate::util::windows::{OwnedWSTR, ToBytes};
use windows::core::{HRESULT};
use windows::Win32::Storage::CloudFilters::{
    CfCreatePlaceholders, CF_CREATE_FLAGS, CF_FS_METADATA, CF_PLACEHOLDER_CREATE_FLAGS, CF_PLACEHOLDER_CREATE_INFO,
    CF_PLACEHOLDER_MAX_FILE_IDENTITY_LENGTH,
};


pub(crate) struct PlaceholderCreateInfo<Identity> {
    relative_file_name: OwnedWSTR,
    meta_data: CF_FS_METADATA,
    identity: Identity,
    flags: CF_PLACEHOLDER_CREATE_FLAGS,
    result: HRESULT,
    create_usn: i64,
}

impl<Identity: ToBytes> PlaceholderCreateInfo<Identity> {
    unsafe fn to_inner(&self) -> Result<CF_PLACEHOLDER_CREATE_INFO> {
        let RelativeFileName = unsafe { self.relative_file_name.loan_pcwstr() };
        let FsMetadata = self.meta_data;

        let file_identity_buf = self.identity.to_bytes();

        anyhow::ensure!(
            file_identity_buf.len() <= CF_PLACEHOLDER_MAX_FILE_IDENTITY_LENGTH as usize,
            "file identity buffer exceeds CF_PLACEHOLDER_MAX_FILE_IDENTITY_LENGTH"
        ); // todo more detail

        let FileIdentity = file_identity_buf.as_ptr() as *const c_void;
        let FileIdentityLength = file_identity_buf.len() as u32;
        let Flags = self.flags;

        Ok(CF_PLACEHOLDER_CREATE_INFO {
            RelativeFileName,
            FsMetadata,
            FileIdentity,
            FileIdentityLength,
            Flags,
            Result: Default::default(),
            CreateUsn: Default::default(),
        })
    }
}

pub(crate) fn create_placeholders<Identity: ToBytes>(
    base_directory_path: impl Into<OwnedWSTR>,
    placeholders: &mut [PlaceholderCreateInfo<Identity>],
    create_flags: CF_CREATE_FLAGS,
) -> Result<u32> {
    let mut unsafe_placeholders = placeholders
        .iter()
        .map(|info| unsafe { info.to_inner() })
        .collect::<Result<Vec<CF_PLACEHOLDER_CREATE_INFO>>>()?;

    let mut entries_processed = 0u32;

    let basedirectorypath = base_directory_path.conv::<OwnedWSTR>();

    let res = unsafe {
        CfCreatePlaceholders(
            basedirectorypath.loan_pcwstr(),
            &mut unsafe_placeholders,
            create_flags,
            Some(&mut entries_processed),
        )
    };

    for (i, unsafe_placeholder) in unsafe_placeholders.iter().enumerate() {
        let safe_boy = &mut placeholders[i];
        safe_boy.result = unsafe_placeholder.Result;
        safe_boy.create_usn = unsafe_placeholder.CreateUsn;
    }

    res.map(|_| entries_processed)
        .context("failed to process placeholders") // TODO more specific context or custom error, including entries processed
}
