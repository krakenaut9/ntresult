
#![no_std]
#![cfg_attr(feature = "alloc", feature(allocator_api))]

pub mod common_error;

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
    #[inline]
    pub fn ntstatus(self) -> NTSTATUS {
        self.0
    }

    /// Check if the error matches a specific `NTSTATUS` code.
    #[must_use]
    #[inline]
    pub fn is(&self, code: NTSTATUS) -> bool {
        self.ntstatus() == code
    }
}

pub trait IntoResult<T, E = Error> {
    fn into_result(self) -> Result<T, E>;
}

pub trait IntoError {
    fn into_error(self) -> Error;
}

impl IntoResult<()> for NTSTATUS {
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
}

impl<T, E> IntoResult<T> for Result<T, E>
where
    E: IntoError,
{
    fn into_result(self) -> Result<T, Error> {
        self.map_err(IntoError::into_error)
    }
}

impl IntoError for NTSTATUS {
    /// Convert [`NTSTATUS`] to `Error`.
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

pub type StatusResult = Result<NTSTATUS>;

pub trait NtStatusResult {
    #[must_use]
    fn ntstatus_res(&self) -> NTSTATUS;
}

impl NtStatusResult for StatusResult {
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
        let success: StatusResult = Ok(STATUS_SUCCESS);
        let error: StatusResult = Err(Error(0xDEAD));

        assert_eq!(success.ntstatus_res(), STATUS_SUCCESS);
        assert_eq!(error.ntstatus_res(), 0xDEAD);
    }

    #[test]
    fn test_ntstatus() {
        let success: Result<()> = Ok(());
        let error: Result<()> = Err(Error(0xDEAD));

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
        let success: Result<()> = Ok(());
        let error: Result<()> = Err(Error(0xDEAD));

        assert_eq!(success.ntstatus(), STATUS_SUCCESS);
        assert_eq!(error.ntstatus(), 0xDEAD);
    }
}
