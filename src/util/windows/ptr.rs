use std::ffi::c_void;

/// # Safety
/// don't keep a reference to the slice, not that I think you can?
pub(crate) unsafe trait FromVoid: 'static {
    unsafe fn from_void_slice(void: &[c_void]) -> Self;
}

pub(crate) trait ToBytes {
    fn to_bytes(self) -> Vec<u16>;
}

impl<T> ToBytes for Option<T>
where
    T: ToBytes,
{
    fn to_bytes(self) -> Vec<u16> {
        match self {
            Some(x) => x.to_bytes(),
            None => Vec::new(),
        }
    }
}
