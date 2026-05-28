//! This module contains error types related to memory allocation.
//!
//! Error types:
//! - [`TryReserveError`] - converted to `STATUS_INSUFFICIENT_RESOURCES`
//! - [`AllocError`] - converted to `STATUS_INSUFFICIENT_RESOURCES`
//!
//! # Examples
//! ```
//! extern crate alloc;
//!
//! use kerror::{Error, IntoError};
//! use alloc::collections::TryReserveError;
//! use windows_sys::Win32::Foundation::STATUS_INSUFFICIENT_RESOURCES;
//! fn allocate_memory(size: usize) -> kerror::Result<Vec<u8>> {
//!    let mut vec = Vec::new();
//!    vec.try_reserve(size)?;
//!    Ok(vec)
//! }
//!
//! let result = allocate_memory(usize::MAX);
//! assert!(result.is_err());
//! assert!(result.err().unwrap().is(STATUS_INSUFFICIENT_RESOURCES));
//! ```

extern crate alloc;

use crate::Error;
use alloc::{alloc::AllocError, collections::TryReserveError};
use windows_sys::Win32::Foundation::STATUS_INSUFFICIENT_RESOURCES;

impl From<TryReserveError> for Error {
    /// Convert [`TryReserveError`] to `Error(STATUS_INSUFFICIENT_RESOURCES)`
    fn from(_: TryReserveError) -> Self {
        Error(STATUS_INSUFFICIENT_RESOURCES)
    }
}

impl From<AllocError> for Error {
    /// Convert [`AllocError`] to `Error(STATUS_INSUFFICIENT_RESOURCES)`
    fn from(_: AllocError) -> Self {
        Error(STATUS_INSUFFICIENT_RESOURCES)
    }
}
