//! Lightweight `NTSTATUS`-based error handling for Windows kernel-mode Rust code.
//!
//! `kerror` provides a minimal and idiomatic interface for working with Windows
//! `NTSTATUS` values in Rust, especially in `#![no_std]` and kernel-mode
//! environments.
//!
//! # Overview
//!
//! - Converts `NTSTATUS` into [`Result`]
//! - Represents failures as [`Error`]
//! - Supports returning expected status values through `Result<NTSTATUS>`
//! - Provides conversions from selected Rust core/alloc errors
//!
//! # Design
//!
//! `kerror` treats only `STATUS_SUCCESS` as success. All other `NTSTATUS` values,
//! including warning and informational codes, are treated as errors unless they
//! are intentionally returned through `Result<NTSTATUS>`.
//!
//! # Example
//!
//! ```rust
//! use kerror::{IntoResult, NtStatus, Result};
//! use windows_sys::Win32::Foundation::{NTSTATUS, STATUS_SUCCESS};
//!
//! fn init_driver() -> Result<()> {
//!     Ok(())
//! }
//!
//! fn driver_entry() -> NTSTATUS {
//!     let result = init_driver();
//!     result.ntstatus()
//! }
//! ```
//!
//! # Features
//! - `alloc` - Enables conversions from allocation-related errors like `TryReserveError` and `AllocError`.
//! - `addr-parse` - Enables conversions from address parsing errors like `AddrParseError`.
//! - `integer` - Enables conversions from integer conversion errors like `TryFromIntError`.
//! - More features may be added in the future to support additional common error types.
//!

#![no_std]
#![warn(missing_docs)]
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
    /// Create an error from a `status`.
    ///
    /// # Parameters
    /// - `status`: The `NTSTATUS` code to create the error from.
    ///
    /// # Returns
    /// An `Error` instance containing the provided `NTSTATUS` code.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::STATUS_ACCESS_DENIED;
    /// use kerror::Error;
    ///
    /// let error = Error::from_ntstatus(STATUS_ACCESS_DENIED);
    /// assert_eq!(error.ntstatus(), STATUS_ACCESS_DENIED);
    /// ```
    #[must_use]
    pub fn from_ntstatus(status: NTSTATUS) -> Error {
        Error(status)
    }

    /// Retrieve the `NTSTATUS` code from the error.
    ///
    /// # Returns
    /// The `NTSTATUS` code contained in the error.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::STATUS_ACCESS_DENIED;
    /// use kerror::Error;
    ///
    /// let error = Error::from_ntstatus(STATUS_ACCESS_DENIED);
    /// assert_eq!(error.ntstatus(), STATUS_ACCESS_DENIED);
    /// ```
    #[must_use]
    #[inline]
    pub fn ntstatus(self) -> NTSTATUS {
        self.0
    }

    /// Check if the error matches a specific `NTSTATUS` code.
    ///
    /// # Parameters
    /// - `code`: The `NTSTATUS` code to compare against.
    ///
    /// # Returns
    /// `true` if the error's `NTSTATUS` code matches the provided code, otherwise `false`.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
    /// use kerror::Error;
    ///
    /// let error = Error::from_ntstatus(STATUS_ACCESS_DENIED);
    /// assert!(error.is(STATUS_ACCESS_DENIED));
    /// assert!(!error.is(STATUS_SUCCESS));
    /// ```
    #[must_use]
    #[inline]
    pub fn is(&self, code: NTSTATUS) -> bool {
        self.ntstatus() == code
    }
}

/// A trait for converting various types into a `Result<T, Error>`.
/// This trait allows for flexible error handling by enabling different types to be converted into
/// a standardized `Result` type with `Error` as the default error type.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
/// use kerror::{Error, IntoResult};
///
/// let err_result = STATUS_ACCESS_DENIED.into_result();
/// assert_eq!(err_result, Err(Error::from_ntstatus(STATUS_ACCESS_DENIED)));
///
/// let ok_result = STATUS_SUCCESS.into_result();
/// assert_eq!(ok_result, Ok(()));
/// ```
pub trait IntoResult<T, E = Error> {
    /// Convert the type into a `Result<T, E>`.
    ///
    /// # Returns
    /// A `Result<T, E>` representing the success or failure of the conversion.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
    /// use kerror::{Error, IntoResult};
    ///
    /// let err_result = STATUS_ACCESS_DENIED.into_result();
    /// assert_eq!(err_result, Err(Error::from_ntstatus(STATUS_ACCESS_DENIED)));
    ///
    /// let ok_result = STATUS_SUCCESS.into_result();
    /// assert_eq!(ok_result, Ok(()));
    /// ```
    fn into_result(self) -> Result<T, E>;
}

/// A trait for converting various types into an `Error`.
/// This trait allows for flexible error handling by enabling different types to be converted into
/// a standardized `Error` type.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::STATUS_ACCESS_DENIED;
/// use kerror::{Error, IntoError};
///
/// let error = STATUS_ACCESS_DENIED.into_error();
/// assert_eq!(error, Error::from_ntstatus(STATUS_ACCESS_DENIED));
/// ```
pub trait IntoError {
    /// Convert the type into an `Error`.
    ///
    /// # Returns
    /// An `Error` representing the failure of the conversion.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::STATUS_ACCESS_DENIED;
    /// use kerror::{Error, IntoError};
    ///
    /// let error = STATUS_ACCESS_DENIED.into_error();
    /// assert_eq!(error, Error::from_ntstatus(STATUS_ACCESS_DENIED));
    /// ```
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

/// A trait for retrieving the [`NTSTATUS`] code from a type.
/// This trait allows for a standardized way to extract the [`NTSTATUS`] code from various types
/// that may represent errors or results in kernel-mode driver code.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
/// use kerror::{Error, NtStatus, IntoError};
///
/// let error = STATUS_ACCESS_DENIED.into_error();
/// assert_eq!(error.ntstatus(), STATUS_ACCESS_DENIED);
/// ```
pub trait NtStatus {
    #[must_use]
    /// Retrieve the `NTSTATUS` code from the type.
    ///
    /// # Returns
    /// The `NTSTATUS` code associated with the type.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
    /// use kerror::{Error, NtStatus, IntoError};
    ///
    /// let error = STATUS_ACCESS_DENIED.into_error();
    /// assert_eq!(error.ntstatus(), STATUS_ACCESS_DENIED);
    /// ```
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

/// A specialized `Result` type where the success case contains an `NTSTATUS` code, and the error case contains an `Error`.
/// This type is useful for functions that primarily return an `NTSTATUS` code to indicate success or failure, while still
///  allowing for detailed error information in the case of failure.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
/// use kerror::{Error, NtStatusResult, StatusResult, IntoError};
///
/// let success: StatusResult = Ok(STATUS_SUCCESS);
/// let error: StatusResult = Err(STATUS_ACCESS_DENIED.into_error());
///
/// assert_eq!(success.ntstatus_res(), STATUS_SUCCESS);
/// assert_eq!(error.ntstatus_res(), STATUS_ACCESS_DENIED);
/// ```
pub type StatusResult = Result<NTSTATUS>;

/// A trait for retrieving the [`NTSTATUS`] code from a `StatusResult`.
/// This trait provides a convenient method to extract the [`NTSTATUS`] code from a `StatusResult`,
/// whether it represents a success or an error.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
/// use kerror::{Error, NtStatusResult, StatusResult, IntoError};
///
/// let success: StatusResult = Ok(STATUS_SUCCESS);
/// let error: StatusResult = Err(STATUS_ACCESS_DENIED.into_error());
///
/// assert_eq!(success.ntstatus_res(), STATUS_SUCCESS);
/// assert_eq!(error.ntstatus_res(), STATUS_ACCESS_DENIED);
/// ```
pub trait NtStatusResult {
    #[must_use]
    /// Retrieve the `NTSTATUS` code from the `StatusResult`.
    ///
    /// # Returns
    /// The `NTSTATUS` code associated with the `StatusResult`, whether it represents success or error.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
    /// use kerror::{Error, NtStatusResult, StatusResult, IntoError};
    ///
    /// let success: StatusResult = Ok(STATUS_SUCCESS);
    /// let error: StatusResult = Err(STATUS_ACCESS_DENIED.into_error());
    ///
    /// assert_eq!(success.ntstatus_res(), STATUS_SUCCESS);
    /// assert_eq!(error.ntstatus_res(), STATUS_ACCESS_DENIED);
    /// ```
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

/// A macro for converting a NTSTATUS code into an Ok containing a value of any type.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{STATUS_SUCCESS, NTSTATUS};
/// use kerror::{krok, StatusResult, Error};
///
/// let status: NTSTATUS = STATUS_SUCCESS;
/// assert_eq!(krok!(status), StatusResult::Ok(STATUS_SUCCESS));
///
/// let data = 42;
/// assert_eq!(krok!(data), Ok::<_, Error>(data));
/// ```
#[macro_export]
macro_rules! krok {
    ($val:expr) => {
        ::core::result::Result::Ok($val)
    };
}

/// A macro for converting a NTSTATUS code into an Error containing this code.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
/// use kerror::{Error, IntoError, krerr};
///
/// let error = krerr!(STATUS_ACCESS_DENIED);
/// assert_eq!(error, Err::<(), _>(Error::from_ntstatus(STATUS_ACCESS_DENIED)));
/// ```
#[macro_export]
macro_rules! krerr {
    ($status:expr) => {
        ::core::result::Result::Err($crate::Error::from_ntstatus($status))
    };
}

/// A macro for converting a NTSTATUS code into an Ok containing a value of any type
/// and returning it immediately.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{STATUS_SUCCESS, NTSTATUS};
/// use kerror::{krokret, Error, StatusResult};
///
/// fn example_ret_status() -> kerror::StatusResult {
///     krokret!(STATUS_SUCCESS);
/// }
/// assert_eq!(example_ret_status(), StatusResult::Ok(STATUS_SUCCESS));
///
/// fn example_ret_data() -> kerror::Result<i32> {
///     krokret!(42);
/// }
/// assert_eq!(example_ret_data(), Ok::<_, Error>(42));
/// ```
#[macro_export]
macro_rules! krokret {
    ($val:expr) => {
        return $crate::krok!($val)
    };
}

/// A macro for converting a NTSTATUS code into a `Result<(), Error>`.
/// For more details, see the [`IntoResult`] trait and its implementation for `NTSTATUS`.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{NTSTATUS, STATUS_ACCESS_DENIED, STATUS_SUCCESS};
/// use kerror::{IntoResult, kres};
///
/// fn success_func() -> NTSTATUS {
///    STATUS_SUCCESS
/// }
/// let ok_result = kres!(success_func());
/// assert_eq!(ok_result, Ok(()));
///
/// fn error_func() -> NTSTATUS {
///    STATUS_ACCESS_DENIED
/// }
/// let err_result = kres!(error_func());
/// assert_eq!(err_result, Err(kerror::Error::from_ntstatus(STATUS_ACCESS_DENIED)));
/// ```
#[macro_export]
macro_rules! kres {
    ($status:expr) => {
        $crate::IntoResult::into_result($status)
    };
}

/// A macro for converting a NTSTATUS code into a `Result<(), Error>` and returning it immediately.
/// For more details, see the [`IntoResult`] trait and its implementation for `NTSTATUS`.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{NTSTATUS, STATUS_ACCESS_DENIED, STATUS_SUCCESS};
/// use kerror::{IntoResult, kresret};
///
/// fn success_func() -> NTSTATUS {
///    STATUS_SUCCESS
/// }
/// fn example_ret_success() -> kerror::Result<()> {
///    kresret!(success_func());
/// }
/// assert_eq!(example_ret_success(), Ok(()));
///
/// fn error_func() -> NTSTATUS {
///    STATUS_ACCESS_DENIED
/// }
/// fn example_ret_error() -> kerror::Result<()> {
///    kresret!(error_func());
/// }
/// assert_eq!(example_ret_error(), Err(kerror::Error::from_ntstatus(STATUS_ACCESS_DENIED)));
/// ```
#[macro_export]
macro_rules! kresret {
    ($status:expr) => {
        return $crate::kres!($status)
    };
}

/// A macro for converting a NTSTATUS code into an Error containing this code
/// and returning it immediately.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
/// use kerror::{Error, IntoError, krerret};
///
/// fn example_ret_error() -> kerror::Result<()> {
///     krerret!(STATUS_ACCESS_DENIED);
/// }
/// assert_eq!(example_ret_error(), Err::<(), _>(Error::from_ntstatus(STATUS_ACCESS_DENIED)));
/// ```
#[macro_export]
macro_rules! krerret {
    ($status:expr) => {
        return $crate::krerr!($status)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::Foundation::STATUS_ACCESS_DENIED;

    #[test]
    fn test_nt_status_result() {
        let success: StatusResult = Ok(STATUS_SUCCESS);
        let error: StatusResult = Err(Error::from_ntstatus(STATUS_ACCESS_DENIED));

        assert_eq!(success.ntstatus_res(), STATUS_SUCCESS);
        assert_eq!(error.ntstatus_res(), STATUS_ACCESS_DENIED);
    }

    #[test]
    fn test_ntstatus() {
        let success: Result<()> = Ok(());
        let error: Result<()> = Err(Error::from_ntstatus(STATUS_ACCESS_DENIED));

        assert_eq!(success.ntstatus(), STATUS_SUCCESS);
        assert_eq!(error.ntstatus(), STATUS_ACCESS_DENIED);
    }

    #[test]
    fn test_into_result() {
        let success: NTSTATUS = STATUS_SUCCESS;
        let error: NTSTATUS = STATUS_ACCESS_DENIED;

        assert_eq!(success.into_result(), Ok(()));
        assert_eq!(
            error.into_result(),
            Err(Error::from_ntstatus(STATUS_ACCESS_DENIED))
        );
    }

    #[test]
    fn test_into_error() {
        let status: NTSTATUS = STATUS_ACCESS_DENIED;
        assert_eq!(
            status.into_error(),
            Error::from_ntstatus(STATUS_ACCESS_DENIED)
        );
    }

    #[test]
    fn test_from_ntstatus() {
        let status: NTSTATUS = STATUS_ACCESS_DENIED;
        let error = Error::from_ntstatus(status);
        assert_eq!(error.ntstatus(), STATUS_ACCESS_DENIED);
    }

    #[test]
    fn test_ntstatus_trait() {
        let success: Result<()> = Ok(());
        let error: Result<()> = Err(Error::from_ntstatus(STATUS_ACCESS_DENIED));

        assert_eq!(success.ntstatus(), STATUS_SUCCESS);
        assert_eq!(error.ntstatus(), STATUS_ACCESS_DENIED);
    }
}
