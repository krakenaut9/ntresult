extern crate alloc;

use alloc::{alloc::AllocError, collections::TryReserveError};
use windows_sys::Win32::Foundation::STATUS_INSUFFICIENT_RESOURCES;

use crate::Error;

impl From<TryReserveError> for Error {
    /// Convert [`TryReserveError`] to appropriate `Error`
    fn from(_: TryReserveError) -> Self {
        Error(STATUS_INSUFFICIENT_RESOURCES)
    }
}

impl From<AllocError> for Error {
    /// Convert [`AllocError`] to appropriate `Error`
    fn from(_: AllocError) -> Self {
        Error(STATUS_INSUFFICIENT_RESOURCES)
    }
}
