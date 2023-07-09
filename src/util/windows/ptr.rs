use std::{borrow::Cow, ffi::c_void};

/// # Safety
/// don't keep a reference to the slice, not that I think you can?
pub(crate) unsafe trait FromVoid {
    unsafe fn from_void_slice(void: &[c_void]) -> Self;
}

// test
struct VoidHolder<'a>(&'a [c_void]);

unsafe impl<'a> FromVoid for VoidHolder<'a> {
    unsafe fn from_void_slice(void: &[c_void]) -> Self {
        Self(void)
    }
}

fn try_it() {
    let buf = Vec::<c_void>::new();

    let holder = unsafe { VoidHolder::from_void_slice(buf.as_ref()) };

    drop(buf);

    println!("{:?}", holder.0);
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
