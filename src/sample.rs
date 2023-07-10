use std::borrow::Cow;

use std::fmt;

use crate::util::windows::ToVoid;

pub(crate) struct TestName(pub(crate) String);

impl fmt::Display for TestName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ToVoid for TestName {
    fn to_bytes<'bytes, 'this: 'bytes>(&'this self) -> Cow<'bytes, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
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
