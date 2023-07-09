use std::{borrow::Cow, ffi::c_void};

/// # Safety
/// don't keep a reference to the slice, not that I think you can?
pub(crate) unsafe trait FromVoid: 'static {
    unsafe fn from_void_slice(void: &[c_void]) -> Self;
}

pub(crate) trait ToBytes {
    fn to_bytes<'bytes, 'this: 'bytes>(&'this self) -> Cow<'bytes, [u8]>;
}

impl<T> ToBytes for Option<T>
where
    T: ToBytes,
{
    fn to_bytes<'bytes, 'this: 'bytes>(&'this self) -> Cow<'bytes, [u8]> {
        match self {
            Some(x) => x.to_bytes(),
            None => Cow::Owned(Vec::new()),
        }
    }
}

impl ToBytes for () {
    fn to_bytes<'bytes, 'this: 'bytes>(&'this self) -> Cow<'bytes, [u8]> {
        Cow::Owned(Vec::new())
    }
}
