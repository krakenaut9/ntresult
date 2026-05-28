//! This module contains error types related to address parsing.
//!
//! Error types:
//! - [`AddrParseError`] - converted to `STATUS_INVALID_ADDRESS`
//!
//! # Examples
//! ```
//! use kerror::{Error, IntoError};
//! use core::net::{AddrParseError, IpAddr};
//! use windows_sys::Win32::Foundation::STATUS_INVALID_ADDRESS;
//!
//! fn parse_address(addr: &str) -> kerror::Result<IpAddr> {
//!     Ok(addr.parse::<IpAddr>()?)
//! }
//!
//! let result = parse_address("invalid_address");
//!
//! assert!(result.is_err());
//! assert!(result.err().unwrap().is(STATUS_INVALID_ADDRESS));
//! ```

use crate::Error;
use core::net::AddrParseError;
use windows_sys::Win32::Foundation::STATUS_INVALID_ADDRESS;

impl From<AddrParseError> for Error {
    /// Convert [`AddrParseError`] to `Error(STATUS_INVALID_ADDRESS)`
    fn from(_: AddrParseError) -> Self {
        Error(STATUS_INVALID_ADDRESS)
    }
}
