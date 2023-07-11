use std::mem;
use std::slice;
use std::{marker::PhantomData, ops::Deref};

pub mod windows;

// TODO: borrow checker doesn't complain even when I dont' have these, gotta figure that out.
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

pub(crate) fn proper_cast_slice<T, U>(slice: &[T]) -> &[U] {
    unsafe {
        slice::from_raw_parts(
            slice.as_ptr() as *const U,
            mem::size_of_val(slice) / mem::size_of::<U>(),
        )
    }
}
