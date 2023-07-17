use std::{
    ffi::{OsStr, OsString},
    iter::Peekable,
    path::{Component, Components, Path, PathBuf},
    sync::{Arc, Mutex, RwLock, Weak},
};

use anyhow::Result;
use dashmap::DashMap;
use widestring::{U16CString, U16String};
use windows::Win32::Storage::FileSystem::FILE_BASIC_INFO;

use crate::cloud_filter::prelude::*;

#[derive(Clone)]
pub(crate) enum FSItem {
    Dir(Arc<RwLock<Directory>>),
    File(Arc<RwLock<File>>),
    // TODO symlink
}

pub(crate) enum FSItemKind {
    Dir,
    File,
}

pub(crate) struct Directory {
    pub(crate) parent: Weak<Mutex<Directory>>,
    pub(crate) metadata: FILE_BASIC_INFO,
    children: DashMap<OsString, FSItem>,
}

pub(crate) struct File {
    pub(crate) content: Vec<u8>,
    pub(crate) metadata: FILE_BASIC_INFO,
}

pub(crate) struct FileSystem {
    root: Directory
}

impl FileSystem {
    // pub(crate) fn walker<'a>(&self, path: impl Iterator<Item=&'a OsStr>) -> impl Iterator<Item=Arc<Mutex<FSItem>>> {

    // }

    pub(crate) fn get_at_path<'a>(
        &self,
        mut path: Peekable<impl Iterator<Item = Component<'a>>>,
    ) -> Result<FSItem> {
        let part = path.next();
        anyhow::ensure!(part.is_some(), "got an empty path");
        let part = part.unwrap();
        while let Some(part) = components.next() {
            if part == ".." {
                todo!() // idk, gotta normalize at some step. Do we also allow symlinks?
                        // heck, maybe using `.components()` _is_ the right idea.
            }
            let child_ref = self.children.get(part);
            if child_ref.is_none() {
                return Err(walked_so_far);
            }
            match (components.peek(), child_ref.unwrap().value()) {
                (None, value) => return Ok(value.clone()),
                (Some(_), FSItem::Dir(dir)) => {}
            }
            // let child = match  {
            //     Some(item) => {
            //         item.value().clone()
            //     }
            // };
        }
        todo!()
    }

    // pub(crate) fn create_at_path(&self, )
}
