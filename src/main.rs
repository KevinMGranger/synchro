mod cloud_filter;
mod real_main;
mod sample;
mod util;

use std::path::{Path, PathBuf};
use std::{ffi::OsString, thread::sleep, time::Duration};

use crate::cloud_filter::{callbacks::*, sync_root::*};
use crate::util::windows::prelude::*;

use anyhow::{Context, Result};
use clap::Parser;

use cloud_filter::placeholders::{create_placeholders, PlaceholderCreateInfo};
use windows::{
    core::{HRESULT, HSTRING},
    Win32::Storage::{
        CloudFilters::{CF_CREATE_FLAG_NONE, CF_FS_METADATA, CF_PLACEHOLDER_CREATE_FLAG_NONE},
        FileSystem::{FILE_ATTRIBUTE_NORMAL, FILE_BASIC_INFO},
    },
};

fn main() -> Result<()> {
    real_main::main()
}
