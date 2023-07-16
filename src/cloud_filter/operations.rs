use windows::Win32::{
    Foundation::NTSTATUS,
    Storage::CloudFilters::{
        CfExecute, CF_OPERATION_INFO, CF_OPERATION_PARAMETERS, CF_OPERATION_PARAMETERS_0,
        CF_OPERATION_PARAMETERS_0_6, CF_OPERATION_TRANSFER_DATA_FLAG_NONE, CF_OPERATION_TYPE,
        CF_OPERATION_TYPE_TRANSFER_DATA,
    },
};

// TODO: macro to help create static SYNC_STATUS whatevers

use super::callbacks::CallbackInfo;
use crate::util::{proper_cast_slice, windows::prelude::*};
use std::mem::{align_of, size_of};

// TODO: SYNC_STATUS
// TODO: take `self` so it can just be implemented on various types of ref?

/// An operation in the [CfExecute] api.
pub(crate) trait Operation {
    /// Execute this operation.
    /// Note that you can't make any bounds on the callback info's generic identities.
    /// If you need those, add them to the implementing struct in the callback.
    fn execute<_T, _U>(&mut self, info: &CallbackInfo<'_, _T, _U>) -> WinResult<()>;
}

/// Create a [CF_OPERATION_INFO] by copying relevant fields from the [CallbackInfo]
/// object, adding the [CF_OPERATION_TYPE].
pub(crate) fn op_info_from_callback<_T, _U>(
    info: &CallbackInfo<'_, _T, _U>,
    op_type: CF_OPERATION_TYPE,
) -> CF_OPERATION_INFO {
    CF_OPERATION_INFO {
        StructSize: std::mem::size_of::<CF_OPERATION_INFO>() as u32,
        Type: op_type,
        ConnectionKey: info.connection_key,
        TransferKey: info.transfer_key,
        CorrelationVector: std::ptr::null(),
        SyncStatus: std::ptr::null(), // TODO
        RequestKey: 0,                // info.request_key, // they don't set this in the example???
    }
}

/// Compute the size of the [CF_OPERATION_PARAMETERS] struct
/// containing the given union member,
/// taking into account the padding necessary.
const fn cf_op_params_size_of<ParamType>() -> u32 {
    let offset = 4;
    let alignment = align_of::<ParamType>();

    let misalignment = offset % alignment;
    let padding_needed = if misalignment != 0 {
        alignment - misalignment
    } else {
        0
    };

    (offset + padding_needed + size_of::<ParamType>()) as u32
}

/// The parameters for a [CF_OPERATION_TYPE_TRANSFER_DATA] operation.
pub(crate) struct TransferDataParams<'a> {
    pub(crate) status: NTSTATUS,
    pub(crate) buf: &'a [c_void],
    pub(crate) offset: i64,
}

// TODO: have I been casting from slices properly??? like if it's just the num of elts
// but a different representation, then oh crap

impl<'a> TransferDataParams<'a> {
    const SIZE: u32 = cf_op_params_size_of::<CF_OPERATION_PARAMETERS_0_6>();
    /// Convert into the FFI version of this struct.
    unsafe fn to_inner(&self) -> CF_OPERATION_PARAMETERS {
        #[allow(non_snake_case)]
        let TransferData = dbg!(CF_OPERATION_PARAMETERS_0_6 {
            Flags: CF_OPERATION_TRANSFER_DATA_FLAG_NONE,
            CompletionStatus: self.status,
            Buffer: self.buf.as_ptr(),
            Offset: self.offset,
            Length: self.buf.len() as i64,
        });
        CF_OPERATION_PARAMETERS {
            ParamSize: Self::SIZE,
            Anonymous: CF_OPERATION_PARAMETERS_0 { TransferData },
        }
    }
}

impl<'p> Operation for TransferDataParams<'p> {
    fn execute<_T, _U>(&mut self, info: &CallbackInfo<'_, _T, _U>) -> WinResult<()> {
        let op_info = dbg!(op_info_from_callback(info, CF_OPERATION_TYPE_TRANSFER_DATA));
        let mut params = unsafe { self.to_inner() };
        // params.Anonymous.TransferData.Buffer = "Hello, world!\n".as_ptr() as *const c_void;

        // test buffer reinterpretation, in case that's the issue.
        let data: &[u8] = proper_cast_slice(self.buf);
        let _ = dbg!(std::str::from_utf8(data));

        unsafe { CfExecute(&op_info, &mut params) }
    }
}

// TODO: these are a little too low level, let's make adapters
// pub(crate) type TransferDataParams = CF_OPERATION_PARAMETERS_0_6;
// pub(crate) type RetrieveDataParams = CF_OPERATION_PARAMETERS_0_5;
// pub(crate) type AckDataParams = CF_OPERATION_PARAMETERS_0_0;
// pub(crate) type RestartHydrationParams = CF_OPERATION_PARAMETERS_0_4;
// pub(crate) type TransferPlaceholdersParams = CF_OPERATION_PARAMETERS_0_7;
// pub(crate) type AckDehydrateParams = CF_OPERATION_PARAMETERS_0_1;
// pub(crate) type AckRenameParams = CF_OPERATION_PARAMETERS_0_3;
// pub(crate) type AckDeleteParams = CF_OPERATION_PARAMETERS_0_2;
