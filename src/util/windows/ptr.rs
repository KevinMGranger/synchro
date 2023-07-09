use std::{borrow::Cow, ffi::c_void};

/// # Safety
/// don't keep a reference to the slice, not that I think you can?
pub(crate) trait FromVoidSlice<'a> {
    fn from_void_slice(void: &'a [c_void]) -> Self;
}

pub(crate) trait FromVoidPointer {
    unsafe fn from_void_ptr(ptr: *const c_void, len: usize) -> Self;
}

impl<'a, T> FromVoidPointer for T
where
    T: FromVoidSlice<'a>,
{
    unsafe fn from_void_ptr(ptr: *const c_void, len: usize) -> Self {
        let slice = std::slice::from_raw_parts(ptr, len);
        T::from_void_slice(slice)
    }
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
