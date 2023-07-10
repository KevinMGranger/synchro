use std::str::Utf8Error;
use std::{
    ffi::c_void,
    fmt::{self, Debug},
    marker::PhantomData,
    slice,
};

use crate::{sample::NameIdentity, util::windows::prelude::*};

use windows::{
    core::{Result as WinResult, PCWSTR},
    Win32::Storage::CloudFilters::{
        CfConnectSyncRoot, CF_CALLBACK_INFO, CF_CALLBACK_PARAMETERS, CF_CALLBACK_PARAMETERS_0_6,
        CF_CALLBACK_REGISTRATION, CF_CALLBACK_TYPE_FETCH_DATA, CF_CALLBACK_TYPE_NONE,
        CF_CONNECTION_KEY, CF_CONNECT_FLAG_NONE,
    },
};

fn the_cooler_fetch_callback(
    info: CallbackInfo<
        '_,
        Result<NameIdentity<&str>, Utf8Error>,
        Result<NameIdentity<&str>, Utf8Error>,
    >,
    params: CF_CALLBACK_PARAMETERS_0_6,
) {
    dbg!(info);
    dbg!(params);
}

pub(crate) extern "system" fn callback_test_fetch(
    callbackinfo: *const CF_CALLBACK_INFO,
    callbackparams: *const CF_CALLBACK_PARAMETERS,
) {
    let callback_info = unsafe { &*callbackinfo };
    let params = unsafe { (*callbackparams).Anonymous.FetchData };

    let callback_info = CallbackInfo::from(callback_info);
    let callback_info = callback_info.map_identities(
        NameIdentity::<&str>::try_from,
        NameIdentity::<&str>::try_from,
    );

    the_cooler_fetch_callback(callback_info, params);
}

pub(crate) const CALLBACK_TABLE: &[CF_CALLBACK_REGISTRATION] = &[
    CF_CALLBACK_REGISTRATION {
        Type: CF_CALLBACK_TYPE_FETCH_DATA,
        Callback: Some(callback_test_fetch),
    },
    CF_CALLBACK_REGISTRATION {
        Type: CF_CALLBACK_TYPE_NONE,
        Callback: None,
    },
];

// TODO: how do we properly handle the lifecycle of the callbacktable?

pub(crate) unsafe fn connect_callbacks(client_dir: &U16CStr) -> WinResult<CF_CONNECTION_KEY> {
    CfConnectSyncRoot(
        PCWSTR(client_dir.as_ptr()),
        CALLBACK_TABLE.as_ptr(),
        None,
        CF_CONNECT_FLAG_NONE,
    )
}

// TODO: I think we should keep doing the identity abstraction, for debugging's sake.
// There can be an identity (heh overloaded) / lazy impl for those who want it that way.

// TODO: here's where I really want to know who owns such pointers. I guess volume info is... always valid?
// Will it just stay alive til the end of the program?
#[derive(Debug)]
pub(crate) struct CallbackVolumeInfo<'a> {
    pub(crate) guid_name: &'a U16CStr,
    pub(crate) dos_name: &'a U16CStr,
    pub(crate) serial_number: u32,
}

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

impl<Identity> Debug for SyncRootInfo<Identity>
where
    Identity: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyncRootInfo")
            .field("file_id", &self.file_id)
            .field("identity", &self.identity)
            .finish()
    }
}

pub(crate) struct FileInfo<'a, Identity> {
    id: i64,
    size: i64,
    identity: Identity,
    normalized_path: &'a U16CStr,
}

impl<'a, Identity> Debug for FileInfo<'a, Identity>
where
    Identity: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileInfo")
            .field("id", &self.id)
            .field("size", &self.size)
            .field("identity", &self.identity)
            .field("normalized_path", &self.normalized_path)
            .finish()
    }
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

pub(crate) struct CallbackInfo<'a, SyncRootIdentity, FileIdentity> {
    pub(crate) connection_key: CF_CONNECTION_KEY,
    // pub(crate) // context: *mut c_void, // TODO keep private? seems like it doesn't work anyway
    pub(crate) volume_info: CallbackVolumeInfo<'a>,
    pub(crate) sync_root: SyncRootInfo<SyncRootIdentity>,
    pub(crate) file: FileInfo<'a, FileIdentity>,
    pub(crate) transfer_key: i64,
    pub(crate) priority_hint: u8,
    // pub(crate) // correlation_vector: *mut CORRELATION_VECTOR, // TODO
    // pub(crate) // process_info: *mut CF_PROCESS_INFO, // TODO
    pub(crate) request_key: i64,
}

impl<'a, SyncRootIdentity, FileIdentity> Debug for CallbackInfo<'a, SyncRootIdentity, FileIdentity>
where
    SyncRootIdentity: Debug,
    FileIdentity: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CallbackInfo")
            .field("connection_key", &self.connection_key)
            .field("volume_info", &self.volume_info)
            .field("sync_root", &self.sync_root)
            .field("file", &self.file)
            .field("transfer_key", &self.transfer_key)
            .field("priority_hint", &self.priority_hint)
            .field("request_key", &self.request_key)
            .finish()
    }
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
                normalized_path: unsafe { U16CStr::from_ptr_str(value.NormalizedPath.0) },
            },
            transfer_key: value.TransferKey,
            priority_hint: value.PriorityHint,
            request_key: value.RequestKey,
        }
    }
}
