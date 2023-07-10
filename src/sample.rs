use std::borrow::Cow;
use std::mem::transmute;
use std::slice;

use std::ffi::c_void;
use std::fmt;
use std::str::Utf8Error;

// TODO: ah crap, you give it a copy going in, but it's a borrow coming out.
// so maybe I was right with requiring a separate type??? idk
// or keep it as a separate unsafe trait or smth.
// helpers around Cow?
// or let it truly be separate types? get some feedback on erognomics.
#[derive(Debug)] // why does this conditionally work here but not on the other thing???
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

impl TryFrom<&[c_void]> for NameIdentity<&str> {
    type Error = Utf8Error;

    fn try_from(value: &[c_void]) -> Result<Self, Self::Error> {
        std::str::from_utf8(unsafe { transmute(value) }).map(NameIdentity)
    }
}

const FILE_CONTENTS: &str = "Hello, world!\n";

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
