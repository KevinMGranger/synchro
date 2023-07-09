mod cloud_filter;
mod sample;
mod util;

use std::{thread::sleep, time::Duration};

use crate::cloud_filter::{callbacks::*, sync_root::*};

use anyhow::{Context, Result};
use clap::Parser;

use cloud_filter::placeholders::{create_placeholders, PlaceholderCreateInfo};
use sample::TestName;
use util::windows::OwnedWSTR;
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
    sync_root_path: HSTRING,
}

fn main() -> Result<()> {
    let args = Args::parse(); // todo: check path is absolute
    register_sync_root(&args.sync_root_path)
        .context("reg_sync_r")
        .unwrap();
    unsafe { connect_callbacks(&args.sync_root_path) }.unwrap();

    let identity = TestName("foo".to_string());

    let placeholders = &mut [PlaceholderCreateInfo {
        relative_file_name: OwnedWSTR::from("foo"),
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

    create_placeholders(&args.sync_root_path, placeholders, CF_CREATE_FLAG_NONE)
        .context("placeholders")?;

    loop {
        sleep(Duration::MAX);
    }

    // Ok(())
}
