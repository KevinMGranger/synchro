use std::path::{Path, PathBuf};
use std::{ffi::OsString, thread::sleep, time::Duration};

use crate::cloud_filter::{callbacks::*, sync_root::*};
use crate::sample::{self, NameIdentity};
use crate::util::windows::prelude::*;

use anyhow::{Context, Result};
use clap::Parser;
use widestring::u16cstr;

use crate::cloud_filter::placeholders::{create_placeholders, PlaceholderCreateInfo};
use windows::{
    core::{HRESULT, HSTRING},
    Win32::Storage::{
        CloudFilters::{CF_CREATE_FLAG_NONE, CF_FS_METADATA, CF_PLACEHOLDER_CREATE_FLAG_NONE},
        FileSystem::{FILE_ATTRIBUTE_NORMAL, FILE_BASIC_INFO},
    },
};

#[derive(Parser, Debug)]
#[command()]
struct Args {
    sync_root_path: PathBuf,
}

pub(crate) fn main() -> Result<()> {
    let args = Args::parse(); // todo: check path is absolute
    let sync_root_path_h = args.sync_root_path.as_os_str().into();
    let sync_root_path_u = u16cstr_from_hstring(&sync_root_path_h);
    register_sync_root(&sync_root_path_h).context("reg_sync_r")?;
    unsafe { connect_callbacks(sync_root_path_u, sample::CALLBACK_TABLE) }.unwrap();

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
            FileSize: 4,
        },
        identity,
        flags: CF_PLACEHOLDER_CREATE_FLAG_NONE,
        result: HRESULT::default(),
        create_usn: 0,
    }];

    create_placeholders(sync_root_path_u, placeholders, CF_CREATE_FLAG_NONE)
        .context("placeholders")?;

    loop {
        sleep(Duration::MAX);
    }

    // Ok(())
}
