//! The meat and potatoes of the file sync example.
//! This is kept separate so `main` can be used as a quick playground.
use std::path::PathBuf;
use std::sync::Mutex;
use std::{thread::sleep, time::Duration};

use crate::cloud_filter::operations::{Operation, TransferDataParams};
use crate::cloud_filter::placeholders::{
    create_placeholders, set_sync_status, PlaceholderCreateInfo,
};
use crate::cloud_filter::{callbacks::*, sync_root::*};
use crate::util::windows::prelude::*;

use anyhow::{Context, Result};
use clap::Parser;
use widestring::u16cstr;
use windows::Win32::Foundation::{NTSTATUS, STATUS_CLOUD_FILE_INVALID_REQUEST, STATUS_SUCCESS};
use windows::Win32::System::Console::{SetConsoleCtrlHandler, CTRL_C_EVENT};

use std::slice;
use windows::{
    core::HRESULT,
    Win32::Storage::{
        CloudFilters::{CF_CREATE_FLAG_NONE, CF_FS_METADATA, CF_PLACEHOLDER_CREATE_FLAG_NONE},
        FileSystem::{FILE_ATTRIBUTE_NORMAL, FILE_BASIC_INFO},
    },
};

use std::ffi::c_void;

use std::str::Utf8Error;

use windows::Win32::Storage::CloudFilters::{
    CfDisconnectSyncRoot, CF_CALLBACK_INFO, CF_CALLBACK_PARAMETERS, CF_CALLBACK_REGISTRATION,
    CF_CALLBACK_TYPE_FETCH_DATA, CF_CALLBACK_TYPE_NONE, CF_CONNECTION_KEY,
};

use crate::util::proper_cast_slice;

// TODO: ah crap, you give it a copy going in, but it's a borrow coming out.
// so maybe I was right with requiring a separate type??? idk
// or keep it as a separate unsafe trait or smth.
// helpers around Cow?
// or let it truly be separate types? get some feedback on erognomics.

/// A simple string to be associated with a file placeholder.
/// Generic so that it can be owned going into placeholder creation,
/// and borrowed within the callback.
#[derive(Debug)]
pub(crate) struct NameIdentity<T>(T);

impl From<String> for NameIdentity<String> {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<[c_void]> for NameIdentity<String> {
    fn as_ref(&self) -> &[c_void] {
        unsafe { slice::from_raw_parts(self.0.as_ptr() as *const c_void, self.0.len()) }
    }
}

impl<'a> TryFrom<&'a [c_void]> for NameIdentity<&'a str> {
    type Error = Utf8Error;

    fn try_from(value: &'a [c_void]) -> Result<Self, Self::Error> {
        std::str::from_utf8(proper_cast_slice(value)).map(NameIdentity)
    }
}

const FILE_CONTENTS: &str = "Hello, world!\n";

type BorrowedNameIdentResult<'a> = Result<NameIdentity<&'a str>, Utf8Error>;

fn the_cooler_fetch_callback(
    info: CallbackInfo<'_, BorrowedNameIdentResult<'_>, BorrowedNameIdentResult<'_>>,
    params: FetchDataParams,
) {
    dbg!(&info);
    dbg!(params);

    // TODO: there are no docs I can find about these statuses.
    // TODO: is it more proper to fail if the range is wack, or just not return the whole thing?
    // should experiment with both.
    // TODO: clearly need my own struct to convert these to usizes, goodness. Could use a Range too!
    // TODO: make it more clear that this is about bytes, by using c_void
    let range = params.RequiredFileOffset as usize
        ..(params.RequiredFileOffset + params.RequiredLength) as usize;
    // TODO: try transferring more than requested? especially with a large amount (on a slow disk (can we emulate that))

    let mut transfer = match dbg!(FILE_CONTENTS.get(range)) {
        Some(slice) => TransferDataParams {
            status: STATUS_SUCCESS,
            buf: proper_cast_slice(slice.as_ref()),
            offset: params.RequiredFileOffset,
        },
        None => TransferDataParams {
            status: STATUS_CLOUD_FILE_INVALID_REQUEST,
            buf: Default::default(),
            offset: 0,
        },
    };

    let _ = dbg!(transfer.execute(&info));
}

pub(crate) extern "system" fn callback_test_fetch(
    callbackinfo: *const CF_CALLBACK_INFO,
    callbackparams: *const CF_CALLBACK_PARAMETERS,
) {
    let callback_info = dbg!(unsafe { &*callbackinfo });
    let params = dbg!(unsafe { (*callbackparams).Anonymous.FetchData });

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

#[derive(Parser, Debug)]
#[command()]
struct Args {
    sync_root_path: PathBuf,
}

// TODO: find a way to map this to each provider.
static CONNECTION_KEY: Mutex<CF_CONNECTION_KEY> = Mutex::new(CF_CONNECTION_KEY(0));

// TODO: check if handler is already running?
unsafe extern "system" fn unregister_on_ctrl_c(ctrltype: u32) -> BOOL {
    if ctrltype != CTRL_C_EVENT {
        return false.into();
    }

    // TODO: timeout using parking_lot mutex
    match CONNECTION_KEY.lock() {
        Err(_) => {
            // TODO LOG
            false.into()
        }
        Ok(mut connection_key) => {
            println!("shutting down cleanly");
            CfDisconnectSyncRoot(*connection_key).unwrap(); // TODO: handle this properly
                                                            // TODO: wait for shutdown from handler
                                                            // what the heck did rustfmt do above
            *connection_key = CF_CONNECTION_KEY::default();
            std::process::exit(0);
        }
    }
}

pub(crate) fn main() -> Result<()> {
    let args = Args::parse(); // todo: check path is absolute
    let sync_root_path_h = args.sync_root_path.as_os_str().into();
    let sync_root_path_u = u16cstr_from_hstring(&sync_root_path_h);

    register_sync_root(&sync_root_path_h).context("reg_sync_r")?;

    let connection_key = unsafe { connect_callbacks(sync_root_path_u, CALLBACK_TABLE) }.unwrap();
    *(CONNECTION_KEY.lock().unwrap()) = connection_key;
    unsafe { SetConsoleCtrlHandler(Some(unregister_on_ctrl_c), true) }.ok()?;

    let identity = NameIdentity::from("asdf".to_owned());
    let placeholders = &mut [PlaceholderCreateInfo {
        relative_file_name: u16cstr!("foo").to_owned(), // todo COULD be ref... do we care?
        meta_data: CF_FS_METADATA {
            BasicInfo: FILE_BASIC_INFO {
                CreationTime: 0,
                LastAccessTime: 0,
                LastWriteTime: 0,
                ChangeTime: 0,
                FileAttributes: FILE_ATTRIBUTE_NORMAL.0,
            },
            FileSize: <str as AsRef<[u8]>>::as_ref(FILE_CONTENTS).len() as i64,
        },
        identity,
        flags: CF_PLACEHOLDER_CREATE_FLAG_NONE,
        result: HRESULT::default(),
        create_usn: 0,
    }];

    create_placeholders(sync_root_path_u, placeholders, CF_CREATE_FLAG_NONE)
        .context("placeholders")?;

    // mark as sync't
    let foo_path = args.sync_root_path.join("foo");
    let foo_path_u = U16CString::from_os_str(foo_path)?;
    set_sync_status(&foo_path_u, true)?;

    loop {
        sleep(Duration::MAX);
    }

    // Ok(())
}
