#![no_std]
#![cfg_attr(feature = "alloc", feature(allocator_api))]

#[cfg(feature = "alloc")]
pub mod alloc;

#[cfg(feature = "hashbrown")]
pub mod hashbrown;

#[cfg(feature = "integer")]
pub mod integer;

#[cfg(feature = "nt-string")]
pub mod nt_string;

use windows_sys::Win32::Foundation::{NTSTATUS, STATUS_SUCCESS};

/// A specialized `Result` type used throughout kernel-mode driver code,
/// where errors are represented by Windows [`NTSTATUS`] codes.
///
/// This alias simplifies function signatures by defaulting the error type to [`Error`],
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// The error type representing [`NTSTATUS`] codes.
///
/// It's a transparent wrapper over `NTSTATUS` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Error(pub(crate) NTSTATUS);

impl core::error::Error for Error {}

impl Error {
    /// Create an error from `status`.
    #[must_use]
    pub fn from_ntstatus(status: NTSTATUS) -> Error {
        Error(status)
    }

    /// Get nested status code.
    #[must_use]
    pub fn ntstatus(self) -> NTSTATUS {
        self.0
    }
}

pub trait IntoResult {
    fn into_result(self) -> Result<(), Error>;

    #[must_use]
    fn into_error(self) -> Error;
}

impl IntoResult for NTSTATUS {
    /// Convert [`NTSTATUS`] to a `Result<(), Error>`.
    ///
    /// # Returns
    /// - `Ok(())` - Ok if status is [`STATUS_SUCCESS`].
    /// - `Err(Error(NTSTATUS))` - All other cases.
    fn into_result(self) -> Result<(), Error> {
        match self {
            STATUS_SUCCESS => Ok(()),
            status => Err(Error::from_ntstatus(status)),
        }
    }

    /// Convert a status to the [`Error`].
    fn into_error(self) -> Error {
        Error::from_ntstatus(self)
    }
}

impl core::fmt::Display for Error {
    /// Displays the error code as an 8-character hexadecimal number.
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(fmt, "{:#08x}", self.0)
    }
}

#[cfg(test)]
mod tests {}
