//! This module contains common error types and traits for the crate.
//! It convert these common error types to appropriate `Error` type, which can be easily converted
//! to `NTSTATUS` code.
//! The common error types include:
//! - [`TryFromIntError`] for integer conversion errors
//! - [`AddrParseError`] for address parsing errors
//! - [`TryReserveError`] and [`AllocError`] for allocation errors (if the `alloc` feature is enabled)
//! - And more in the future...
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

#[cfg(feature = "alloc")]
pub mod alloc;

#[cfg(feature = "addr-parse")]
pub mod addr_parse;

#[cfg(feature = "integer")]
pub mod integer;
