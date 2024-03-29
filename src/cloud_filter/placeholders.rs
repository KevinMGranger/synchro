use crate::util::windows::{prelude::*, FileHandle};

use anyhow::{Context, Result};
use std::ffi::c_void;
use std::fmt;
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_SHARE_NONE, OPEN_EXISTING,
    WRITE_DAC,
};

use windows::core::{HRESULT, PCWSTR};
use windows::Win32::Storage::CloudFilters::{
    CfCreatePlaceholders, CfSetInSyncState, CF_CREATE_FLAGS, CF_FS_METADATA,
    CF_IN_SYNC_STATE_IN_SYNC, CF_IN_SYNC_STATE_NOT_IN_SYNC, CF_PLACEHOLDER_CREATE_FLAGS,
    CF_PLACEHOLDER_CREATE_INFO, CF_PLACEHOLDER_MAX_FILE_IDENTITY_LENGTH, CF_SET_IN_SYNC_FLAG_NONE,
};

/// Information to create a placeholder.
pub(crate) struct PlaceholderCreateInfo<Identity> {
    pub(crate) relative_file_name: U16CString,
    pub(crate) meta_data: CF_FS_METADATA,
    pub(crate) identity: Identity,
    pub(crate) flags: CF_PLACEHOLDER_CREATE_FLAGS,
    pub(crate) result: HRESULT,
    pub(crate) create_usn: i64,
}

impl<Identity> PlaceholderCreateInfo<Identity>
where
    Identity: AsRef<[c_void]>,
{
    /// Convert to a [CF_PLACEHOLDER_CREATE_INFO].
    #[allow(non_snake_case)]
    unsafe fn to_inner(&self) -> Result<CF_PLACEHOLDER_CREATE_INFO> {
        let RelativeFileName = PCWSTR(self.relative_file_name.as_ptr());
        let FsMetadata = self.meta_data;

        let file_identity_buf = self.identity.as_ref();
        anyhow::ensure!(
            file_identity_buf.len() <= CF_PLACEHOLDER_MAX_FILE_IDENTITY_LENGTH as usize,
            "file identity buffer exceeds CF_PLACEHOLDER_MAX_FILE_IDENTITY_LENGTH"
        ); // todo more detail

        let FileIdentity = file_identity_buf.as_ptr();
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

/// Since creating placeholders can _partially_ fail,
/// this error context for [anyhow] lists how many
/// entries were processed regardless of the error.
struct CreateErrorContext(u32);

impl fmt::Display for CreateErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Processed {} entries", self.0)
    }
}

#[allow(overflowing_literals)]
pub(crate) const ALREADY_EXISTS: HRESULT = HRESULT(0x800700B7);

// TODO: each placeholder could have a different type of identity, right?
// how do we represent that properly when we make the whole server wiring everything together?
// Maybe it gets the whole context when deserializing.

/// Create placeholders within a base directory.
pub(crate) fn create_placeholders<Identity>(
    base_directory_path: &U16CStr,
    placeholders: &mut [PlaceholderCreateInfo<Identity>],
    create_flags: CF_CREATE_FLAGS,
) -> Result<u32>
where
    Identity: AsRef<[c_void]>,
{
    let mut unsafe_placeholders = placeholders
        .iter()
        .map(|info| unsafe { info.to_inner() })
        .collect::<Result<Vec<CF_PLACEHOLDER_CREATE_INFO>>>()?;

    let mut entries_processed = 0u32;

    let res = unsafe {
        CfCreatePlaceholders(
            PCWSTR(base_directory_path.as_ptr()),
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

    match res {
        Ok(_) => Ok(entries_processed),
        Err(e) if e.code() == ALREADY_EXISTS => Ok(entries_processed),
        Err(e) => Err(e).context(CreateErrorContext(entries_processed)),
    }
}

// todo: usn
// todo: allow passing a handle?
pub(crate) fn set_sync_status(path: &U16CStr, in_sync: bool) -> Result<()> {
    let handle = unsafe {
        CreateFileW(
            PCWSTR(path.as_ptr()),
            FILE_GENERIC_READ | WRITE_DAC,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE::default(),
        )
        .context("createfile")?
    };
    let handle = FileHandle::from(handle);

    unsafe {
        CfSetInSyncState(
            &handle,
            if in_sync {
                CF_IN_SYNC_STATE_IN_SYNC
            } else {
                CF_IN_SYNC_STATE_NOT_IN_SYNC
            },
            CF_SET_IN_SYNC_FLAG_NONE,
            None,
        )
        .context("set in sync")
    }
}
