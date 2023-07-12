use std::mem;
use std::slice;
use std::{marker::PhantomData, ops::Deref};

pub mod windows;

// TODO: borrow checker doesn't complain even when I dont' have these, gotta figure that out.
/// An easy way to associate a lifetime to something that otherwise wouldn't have it.
/// In the FFI we're doing, there are several structures that appear to be owned but are actually borrowed.
/// This helps with that.
pub(crate) struct BorrowWrapper<'a, T>(T, PhantomData<&'a ()>);

impl<'a, T> Deref for BorrowWrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> From<T> for BorrowWrapper<'a, T> {
    fn from(value: T) -> Self {
        BorrowWrapper(value, PhantomData)
    }
}

/// Cast a slice between two types, properly changing the size based
/// on the size of the elements.
pub(crate) fn proper_cast_slice<T, U>(slice: &[T]) -> &[U] {
    unsafe {
        slice::from_raw_parts(
            slice.as_ptr() as *const U,
            mem::size_of_val(slice) / mem::size_of::<U>(),
        )
    }
}
