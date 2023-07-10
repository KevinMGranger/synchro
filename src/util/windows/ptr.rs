use std::{borrow::Cow, ffi::c_void};

pub(crate) trait FromVoidPointer {
    /// # Safety
    /// can't keep a ref to the ptr for any longer than...
    /// well, I guess I don't know.
    unsafe fn from_void_ptr(ptr: *const c_void, len: usize) -> Self;
}

impl<T> FromVoidPointer for T
where
    T: for<'a> From<&'a [c_void]>,
{
    unsafe fn from_void_ptr(ptr: *const c_void, len: usize) -> Self {
        let slice = std::slice::from_raw_parts(ptr, len);
        T::from(slice)
    }
}

// TODO: what would a safer abstraction for this look like, if any?
// or at least something more ergonomic?

// also, is there a dictinction between "to" and "as" here, really?
// cuz it's ambiguous based on the impl.
pub(crate) trait AsVoidPointer {
    unsafe fn as_void_ptr(&self) -> (*const c_void, usize);
}
