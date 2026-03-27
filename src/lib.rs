
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

pub trait NtStatus {
    #[must_use]
    fn ntstatus(&self) -> NTSTATUS;
}

impl<T> NtStatus for Result<T> {
    fn ntstatus(&self) -> NTSTATUS {
        match self {
            Ok(_) => STATUS_SUCCESS,
            Err(err) => err.ntstatus(),
        }
    }
}

pub trait NtStatusResult {
    fn ntstatus_res(&self) -> NTSTATUS;
}

impl NtStatusResult for Result<NTSTATUS, Error> {
    fn ntstatus_res(&self) -> NTSTATUS {
        match self {
            Ok(status) => *status,
            Err(err) => err.ntstatus(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nt_status_result() {
        let success: Result<NTSTATUS, Error> = Ok(STATUS_SUCCESS);
        let error: Result<NTSTATUS, Error> = Err(Error(0xDEAD));

        assert_eq!(success.ntstatus_res(), STATUS_SUCCESS);
        assert_eq!(error.ntstatus_res(), 0xDEAD);
    }

    #[test]
    fn test_ntstatus() {
        let success: Result<(), Error> = Ok(());
        let error: Result<(), Error> = Err(Error(0xDEAD));

        assert_eq!(success.ntstatus(), STATUS_SUCCESS);
        assert_eq!(error.ntstatus(), 0xDEAD);
    }

    #[test]
    fn test_into_result() {
        let success: NTSTATUS = STATUS_SUCCESS;
        let error: NTSTATUS = 0xDEAD;

        assert_eq!(success.into_result(), Ok(()));
        assert_eq!(error.into_result(), Err(Error(0xDEAD)));
    }

    #[test]
    fn test_into_error() {
        let status: NTSTATUS = 0xDEAD;
        assert_eq!(status.into_error(), Error(0xDEAD));
    }

    #[test]
    fn test_from_ntstatus() {
        let status: NTSTATUS = 0xDEAD;
        let error = Error::from_ntstatus(status);
        assert_eq!(error.ntstatus(), 0xDEAD);
    }

    #[test]
    fn test_ntstatus_trait() {
        let success: Result<(), Error> = Ok(());
        let error: Result<(), Error> = Err(Error(0xDEAD));

        assert_eq!(success.ntstatus(), STATUS_SUCCESS);
        assert_eq!(error.ntstatus(), 0xDEAD);
    }
}
