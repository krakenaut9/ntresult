//! This module contains error types related to integer conversion errors.
//!
//! Error types:
//! - [`TryFromIntError`] - converted to `STATUS_INTEGER_OVERFLOW`
//!
//! # Examples
//! ```
//! use kerror::{Error, IntoError};
//! use core::num::TryFromIntError;
//! use windows_sys::Win32::Foundation::STATUS_INTEGER_OVERFLOW;
//!
//! fn do_something() -> kerror::Result<u8> {
//!   Ok(u8::try_from(256)?)
//! }
//!
//! let result = do_something();
//! assert!(result.is_err());
//! assert!(result.err().unwrap().is(STATUS_INTEGER_OVERFLOW));
//! ```

use crate::Error;
use core::num::TryFromIntError;
use windows_sys::Win32::Foundation::STATUS_INTEGER_OVERFLOW;

impl From<TryFromIntError> for Error {
    /// Convert [`TryFromIntError`] to `Error(STATUS_INTEGER_OVERFLOW)`
    fn from(_: TryFromIntError) -> Self {
        Error(STATUS_INTEGER_OVERFLOW)
    }
}
