//! More idiomatic / safer abstractions over the Windows Cloud Filter API.
//! 
//! For example, most dynamically-sized pointers are changed into slices,
//! strange types for things like sizes (i64) are cast to usizes, etc.
//! 
//! Also contains functions to convert to/from the related FFI structures.
pub(crate) mod callbacks;
pub(crate) mod operations;
pub(crate) mod placeholders;
pub(crate) mod prelude;
pub(crate) mod sync_root;
