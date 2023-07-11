use std::{ffi::c_void, fmt::Debug, slice};

use crate::util::windows::prelude::*;

use windows::Win32::Storage::CloudFilters::{
    CF_CALLBACK_INFO, CF_CALLBACK_PARAMETERS_0_6, CF_CONNECTION_KEY,
};

// TODO: here's where I really want to know who owns such pointers. I guess volume info is... always valid?
// Will it just stay alive til the end of the program?
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct CallbackVolumeInfo<'a> {
    pub(crate) guid_name: &'a U16CStr,
    pub(crate) dos_name: &'a U16CStr,
    pub(crate) serial_number: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct SyncRootInfo<Identity> {
    pub(crate) file_id: i64,
    pub(crate) identity: Identity,
}

impl<Identity> SyncRootInfo<Identity> {
    pub(crate) fn map_identity<NewIdentity>(
        self,
        f: impl FnOnce(Identity) -> NewIdentity,
    ) -> SyncRootInfo<NewIdentity> {
        // TODO: fallible versions?
        // allowing it to be internally Result is cool so you can still inspect other info, def doc that
        SyncRootInfo {
            file_id: self.file_id,
            identity: (f)(self.identity),
        }
    }
}

#[derive(Debug)]
pub(crate) struct FileInfo<'a, Identity> {
    id: i64,
    size: i64,
    identity: Identity,
    normalized_path: Option<&'a U16CStr>,
}

// TODO: transpose
impl<'a, Identity> FileInfo<'a, Identity> {
    pub(crate) fn map_identity<NewIdentity>(
        self,
        f: impl FnOnce(Identity) -> NewIdentity,
    ) -> FileInfo<'a, NewIdentity> {
        // TODO: fallible versions?
        // allowing it to be internally Result is cool so you can still inspect other info, def doc that
        FileInfo {
            id: self.id,
            size: self.size,
            identity: (f)(self.identity),
            normalized_path: self.normalized_path,
        }
    }
}

// TODO: context doesn't go in here, it's a pointer to the main thingy
#[derive(Debug)]
pub(crate) struct CallbackInfo<'a, SyncRootIdentity, FileIdentity> {
    pub(crate) connection_key: CF_CONNECTION_KEY,
    pub(crate) volume_info: CallbackVolumeInfo<'a>,
    pub(crate) sync_root: SyncRootInfo<SyncRootIdentity>,
    pub(crate) file: FileInfo<'a, FileIdentity>,
    pub(crate) transfer_key: i64,
    pub(crate) priority_hint: u8,
    // pub(crate) // correlation_vector: *mut CORRELATION_VECTOR, // TODO
    // pub(crate) // process_info: *mut CF_PROCESS_INFO, // TODO
    pub(crate) request_key: i64,
}

impl<'a, SyncRootIdentity, FileIdentity> CallbackInfo<'a, SyncRootIdentity, FileIdentity> {
    pub(crate) fn map_identities<NewSyncRootId, NewFileId>(
        self,
        sync_root_mapper: impl FnOnce(SyncRootIdentity) -> NewSyncRootId,
        file_mapper: impl FnOnce(FileIdentity) -> NewFileId,
    ) -> CallbackInfo<'a, NewSyncRootId, NewFileId> {
        CallbackInfo {
            connection_key: self.connection_key,
            volume_info: self.volume_info,
            sync_root: self.sync_root.map_identity(sync_root_mapper),
            file: self.file.map_identity(file_mapper),
            transfer_key: self.transfer_key,
            priority_hint: self.priority_hint,
            request_key: self.request_key,
        }
    }
}

impl<'a> From<&'a CF_CALLBACK_INFO> for CallbackInfo<'a, &'a [c_void], &'a [c_void]> {
    fn from(value: &'a CF_CALLBACK_INFO) -> Self {
        Self {
            connection_key: value.ConnectionKey,
            volume_info: unsafe {
                CallbackVolumeInfo {
                    guid_name: U16CStr::from_ptr_str(value.VolumeGuidName.0),
                    dos_name: U16CStr::from_ptr_str(value.VolumeDosName.0),
                    serial_number: value.VolumeSerialNumber,
                }
            },
            sync_root: SyncRootInfo {
                file_id: value.SyncRootFileId,
                identity: unsafe {
                    slice::from_raw_parts(
                        value.SyncRootIdentity,
                        value.SyncRootIdentityLength as usize,
                    )
                },
            },
            file: FileInfo {
                id: value.FileId,
                size: value.FileSize,
                identity: unsafe {
                    slice::from_raw_parts(value.FileIdentity, value.FileIdentityLength as usize)
                },
                normalized_path: if value.NormalizedPath.is_null() {
                    None
                } else {
                    Some(unsafe { U16CStr::from_ptr_str(value.NormalizedPath.0) })
                },
            },
            transfer_key: value.TransferKey,
            priority_hint: value.PriorityHint,
            request_key: value.RequestKey,
        }
    }
}

pub(crate) type FetchDataParams = CF_CALLBACK_PARAMETERS_0_6;
