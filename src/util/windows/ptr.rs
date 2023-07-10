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

// pub(crate) trait ToVoid {
//     fn to_bytes<'void, 'this: 'void>(&'this self) -> Cow<'void, [c_void]>;
// }

// impl<T> ToVoid for Option<T>
// where
//     T: ToVoid,
// {
//     fn to_bytes<'void, 'this: 'void>(&'this self) -> Cow<'void, [c_void]> {
//         match self {
//             Some(x) => x.to_bytes(),
//             None => Cow::Owned(Vec::new()),
//         }
//     }
// }

// impl ToVoid for () {
//     fn to_bytes<'bytes, 'this: 'bytes>(&'this self) -> Cow<'bytes, [c_void]> {
//         Cow::Owned(Vec::new())
//     }
// }
