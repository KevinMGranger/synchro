mod cloud_filter;
mod sample;
mod util;

use std::{thread::sleep, time::Duration};

use crate::cloud_filter::{callbacks::*, sync_root::*};

use anyhow::{Context, Result};
use clap::Parser;

use windows::core::HSTRING;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    sync_root_path: HSTRING,
}

fn main() -> Result<()> {
    let args = Args::parse();
    register_sync_root(&args.sync_root_path)
        .context("reg_sync_r")
        .unwrap();
    unsafe { connect_callbacks(&args.sync_root_path) }.unwrap();

    loop {
        sleep(Duration::MAX);
    }

    // Ok(())
}
