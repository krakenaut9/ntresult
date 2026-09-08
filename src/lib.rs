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
//!
//! All features are additive: each one only adds [`From`] conversions into [`Error`].
//! Enabling a feature never changes the behaviour of an existing conversion.
//!
//! | Feature | Default | Toolchain | Conversion added |
//! |---|---|---|---|
//! | `alloc` | yes | stable | `alloc::collections::TryReserveError` |
//! | `allocator-api` | no | **nightly** | `core::alloc::AllocError` |
//!
//! Conversions that need nothing beyond `core` -- `core::num::TryFromIntError`
//! and `core::net::AddrParseError` -- are always available and are not gated.
//!
//! `allocator-api` enables the unstable `allocator_api` language feature and so
//! requires a nightly compiler. The default feature set builds on stable Rust.
//!
//! `allocator-api` does not imply `alloc`: `AllocError` lives in `core`, so the
//! conversion is available even without an allocator.
//!
//! More features may be added in the future to support additional common error types.
//!

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "allocator-api", feature(allocator_api))]
// `docsrs` is set only by docs.rs, via `rustdoc-args` in Cargo.toml.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod common_error;

mod layout;

use layout::status_accessors;

use windows_sys::Win32::Foundation::{NTSTATUS, STATUS_SUCCESS};

/// A specialized `Result` type used throughout kernel-mode driver code,
/// where errors are represented by Windows [`NTSTATUS`] codes.
///
/// This alias simplifies function signatures by defaulting the error type to [`Error`],
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// The error type representing [`NTSTATUS`] codes.
///
/// It's a transparent wrapper over `NTSTATUS` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    #[inline]
    pub const fn from_ntstatus(status: NTSTATUS) -> Error {
        Error(status)
    }

    /// Create an error from a raw 32-bit status pattern.
    ///
    /// `NTSTATUS` is signed, but status codes are written as unsigned
    /// hexadecimal by convention. This constructor accepts them in that form,
    /// so no `as i32` cast is needed at the call site.
    ///
    /// # Examples
    /// ```
    /// use kerror::Error;
    ///
    /// // A customer-defined error code; leading nibble `0xE`.
    /// let error = Error::from_bits(0xE000_0001);
    ///
    /// assert!(error.is_customer());
    /// assert!(error.is_error());
    /// assert_eq!(error.code(), 0x0001);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_bits(bits: u32) -> Error {
        Error(layout::from_bits(bits))
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
    pub const fn ntstatus(self) -> NTSTATUS {
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
    pub const fn is(self, code: NTSTATUS) -> bool {
        self.ntstatus() == code
    }
}

/// The severity class encoded in the top two bits of an [`NTSTATUS`].
///
/// An `NTSTATUS` is laid out as:
///
/// ```text
///  31 30 | 29 | 28 | 27 ------ 16 | 15 ------- 0
///   Sev  | C  | R  |   Facility   |     Code
/// ```
///
/// Variants are ordered by increasing severity, so comparisons such as
/// `severity >= Severity::Warning` work as expected.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{STATUS_BUFFER_OVERFLOW, STATUS_SUCCESS};
/// use kerror::Severity;
///
/// assert_eq!(Severity::from_ntstatus(STATUS_SUCCESS), Severity::Success);
/// assert_eq!(Severity::from_ntstatus(STATUS_BUFFER_OVERFLOW), Severity::Warning);
/// assert!(Severity::from_ntstatus(STATUS_BUFFER_OVERFLOW) >= Severity::Warning);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Severity {
    /// `STATUS_SEVERITY_SUCCESS` (`0b00`).
    ///
    /// Includes `STATUS_SUCCESS`, but also codes like `STATUS_PENDING` and
    /// `STATUS_TIMEOUT`, which this crate still treats as errors.
    Success = 0,
    /// `STATUS_SEVERITY_INFORMATIONAL` (`0b01`).
    Information = 1,
    /// `STATUS_SEVERITY_WARNING` (`0b10`).
    Warning = 2,
    /// `STATUS_SEVERITY_ERROR` (`0b11`).
    Error = 3,
}

impl Severity {
    /// Extract the severity class from an [`NTSTATUS`].
    ///
    /// Accepts any `NTSTATUS`, so a raw status can be classified without
    /// wrapping it in an [`Error`] first.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
    /// use kerror::Severity;
    ///
    /// assert_eq!(Severity::from_ntstatus(STATUS_SUCCESS), Severity::Success);
    /// assert_eq!(Severity::from_ntstatus(STATUS_ACCESS_DENIED), Severity::Error);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_ntstatus(status: NTSTATUS) -> Severity {
        match layout::SEVERITY.get(status) {
            0 => Severity::Success,
            1 => Severity::Information,
            2 => Severity::Warning,
            _ => Severity::Error,
        }
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
    /// Formats the status as `0x` followed by eight zero-padded uppercase
    /// hexadecimal digits, matching how `NTSTATUS` values are written in the
    /// Windows headers.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
    /// use kerror::Error;
    ///
    /// assert_eq!(
    ///     Error::from_ntstatus(STATUS_ACCESS_DENIED).to_string(),
    ///     "0xC0000022"
    /// );
    /// assert_eq!(
    ///     Error::from_ntstatus(STATUS_SUCCESS).to_string(),
    ///     "0x00000000"
    /// );
    /// ```
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(fmt, "{:#010X}", self.0)
    }
}

/// A trait for retrieving the [`NTSTATUS`] code from a type.
/// This trait allows for a standardized way to extract the [`NTSTATUS`] code from various types
/// that may represent errors or results in kernel-mode driver code.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{NTSTATUS, STATUS_ACCESS_DENIED, STATUS_SUCCESS};
/// use kerror::{Error, NtStatus};
///
/// // The point of the trait: one function covering every implementor.
/// fn to_status<T: NtStatus>(value: &T) -> NTSTATUS {
///     value.ntstatus()
/// }
///
/// let success: kerror::Result<()> = Ok(());
/// let failure = Error::from_ntstatus(STATUS_ACCESS_DENIED);
///
/// assert_eq!(to_status(&success), STATUS_SUCCESS);
/// assert_eq!(to_status(&failure), STATUS_ACCESS_DENIED);
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
    /// use kerror::{Error, NtStatus};
    ///
    /// let success: kerror::Result<()> = Ok(());
    /// let failure: kerror::Result<()> =
    ///     Err(Error::from_ntstatus(STATUS_ACCESS_DENIED));
    ///
    /// assert_eq!(success.ntstatus(), STATUS_SUCCESS);
    /// assert_eq!(failure.ntstatus(), STATUS_ACCESS_DENIED);
    /// ```
    fn ntstatus(&self) -> NTSTATUS;
}

impl NtStatus for Result<()> {
    fn ntstatus(&self) -> NTSTATUS {
        match self {
            Ok(()) => STATUS_SUCCESS,
            Err(err) => err.ntstatus(),
        }
    }
}

impl NtStatus for StatusResult {
    fn ntstatus(&self) -> NTSTATUS {
        match self {
            Ok(status) => status.ntstatus(),
            Err(err) => err.ntstatus(),
        }
    }
}

/// Collapse any [`Result`] into an `NTSTATUS`, treating every `Ok` as success.
///
/// Use this when the `Ok` payload is data rather than a status. The name states
/// the discard; [`NtStatus::ntstatus`] is the non-discarding counterpart and is
/// implemented only where an `Ok` carries a meaningful status.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_SUCCESS};
/// use kerror::{Error, NtStatusOrSuccess};
///
/// let data: kerror::Result<[u8; 4]> = Ok([0; 4]);
/// let failed: kerror::Result<[u8; 4]> =
///     Err(Error::from_ntstatus(STATUS_ACCESS_DENIED));
///
/// assert_eq!(data.ntstatus_or_success(), STATUS_SUCCESS);
/// assert_eq!(failed.ntstatus_or_success(), STATUS_ACCESS_DENIED);
/// ```
///
/// [`NtStatus`] is deliberately *not* implemented for such a `Result`, so
/// reaching for it is a compile error rather than a silent `STATUS_SUCCESS`:
///
/// ```compile_fail
/// use kerror::NtStatus;
///
/// // 42 is a byte count, not a status code.
/// let written: kerror::Result<i32> = Ok(42);
/// let _ = written.ntstatus();
/// ```
pub trait NtStatusOrSuccess {
    /// Return the error's status, or `STATUS_SUCCESS` for any `Ok`.
    #[must_use]
    fn ntstatus_or_success(&self) -> NTSTATUS;
}

impl<T> NtStatusOrSuccess for Result<T> {
    fn ntstatus_or_success(&self) -> NTSTATUS {
        match self {
            Ok(_) => STATUS_SUCCESS,
            Err(err) => err.ntstatus(),
        }
    }
}

impl NtStatus for Error {
    fn ntstatus(&self) -> NTSTATUS {
        self.0
    }
}

impl From<Error> for NTSTATUS {
    /// Unwrap an [`Error`] back into its [`NTSTATUS`] code.
    ///
    /// The reverse conversion is deliberately absent: `NTSTATUS` is an alias
    /// for `i32`, so `From<NTSTATUS> for Error` would turn *every* integer into
    /// a status code, silently, through `?`. Use [`IntoError`] instead, which
    /// names the conversion at the call site.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::{NTSTATUS, STATUS_ACCESS_DENIED};
    /// use kerror::Error;
    ///
    /// let error = Error::from_ntstatus(STATUS_ACCESS_DENIED);
    /// let status: NTSTATUS = error.into();
    ///
    /// assert_eq!(status, STATUS_ACCESS_DENIED);
    /// ```
    fn from(error: Error) -> NTSTATUS {
        error.0
    }
}

/// A status that is an expected outcome rather than a failure.
///
/// Wrapping the status distinguishes "this `i32` is a status code" from "this
/// `i32` is data". Without it, a `Result<i32>` carrying a byte count would be
/// indistinguishable from one carrying a status, since `NTSTATUS` is an alias
/// for `i32`.
///
/// A `Status` may legitimately hold a failure code: `Ok(Status::new(
/// STATUS_BUFFER_TOO_SMALL))` means "the operation completed, return this
/// status verbatim". Whether a status is an expected outcome or an error is the
/// caller's decision, not a function of its severity.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::STATUS_PENDING;
/// use kerror::{NtStatus, Status, StatusResult};
///
/// fn begin_io() -> StatusResult {
///     Ok(Status::new(STATUS_PENDING))
/// }
///
/// assert_eq!(begin_io().ntstatus(), STATUS_PENDING);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Status(NTSTATUS);

impl Status {
    /// `STATUS_SUCCESS`.
    pub const SUCCESS: Status = Status(STATUS_SUCCESS);

    /// Wrap an `NTSTATUS` as an expected outcome.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::STATUS_PENDING;
    /// use kerror::Status;
    ///
    /// let status = Status::new(STATUS_PENDING);
    /// assert_eq!(status.ntstatus(), STATUS_PENDING);
    /// ```
    #[must_use]
    #[inline]
    pub const fn new(status: NTSTATUS) -> Status {
        Status(status)
    }

    /// Wrap a raw 32-bit status pattern as an expected outcome.
    ///
    /// `NTSTATUS` is signed, but status codes are written as unsigned
    /// hexadecimal by convention. This constructor accepts them in that form,
    /// so no `as i32` cast is needed at the call site.
    ///
    /// # Examples
    /// ```
    /// use kerror::Status;
    ///
    /// let status = Status::from_bits(0xE000_0001);
    /// assert!(status.is_customer());
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_bits(bits: u32) -> Status {
        Status(layout::from_bits(bits))
    }

    /// Retrieve the wrapped `NTSTATUS`.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::STATUS_SUCCESS;
    /// use kerror::Status;
    ///
    /// assert_eq!(Status::SUCCESS.ntstatus(), STATUS_SUCCESS);
    /// ```
    #[must_use]
    #[inline]
    pub const fn ntstatus(self) -> NTSTATUS {
        self.0
    }
}

impl From<NTSTATUS> for Status {
    /// Wrap an `NTSTATUS` as an expected outcome.
    ///
    /// Unlike [`Error`], `Status` never occupies the error position of a
    /// [`Result`], so `?` can never apply this conversion implicitly.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::STATUS_PENDING;
    /// use kerror::Status;
    ///
    /// let status: Status = STATUS_PENDING.into();
    /// assert_eq!(status.ntstatus(), STATUS_PENDING);
    /// ```
    #[inline]
    fn from(status: NTSTATUS) -> Status {
        Status(status)
    }
}

impl From<Status> for NTSTATUS {
    /// Unwrap a [`Status`] back into its `NTSTATUS` code.
    ///
    /// # Examples
    /// ```
    /// use windows_sys::Win32::Foundation::{NTSTATUS, STATUS_PENDING};
    /// use kerror::Status;
    ///
    /// let status: NTSTATUS = Status::new(STATUS_PENDING).into();
    /// assert_eq!(status, STATUS_PENDING);
    /// ```
    #[inline]
    fn from(status: Status) -> NTSTATUS {
        status.0
    }
}

impl NtStatus for Status {
    fn ntstatus(&self) -> NTSTATUS {
        self.0
    }
}

// The `NTSTATUS` field accessors are generated once and shared by both
// newtypes, so `Error` and `Status` cannot drift apart.
status_accessors!(Error, "Error::from_ntstatus");
status_accessors!(Status, "Status::new");

/// A specialized `Result` type where the success case contains an `NTSTATUS` code, and the error case contains an `Error`.
/// This type is useful for functions that primarily return an `NTSTATUS` code to indicate success or failure, while still
///  allowing for detailed error information in the case of failure.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{
///     STATUS_ACCESS_DENIED, STATUS_BUFFER_TOO_SMALL, STATUS_SUCCESS,
/// };
/// use kerror::{Error, IntoError, NtStatus, Status, StatusResult};
///
/// let success: StatusResult = Ok(Status::SUCCESS);
/// let carried: StatusResult = Ok(Status::new(STATUS_BUFFER_TOO_SMALL));
/// let error: StatusResult = Err(STATUS_ACCESS_DENIED.into_error());
///
/// assert_eq!(success.ntstatus(), STATUS_SUCCESS);
/// // The carried status survives; it is not collapsed to STATUS_SUCCESS.
/// assert_eq!(carried.ntstatus(), STATUS_BUFFER_TOO_SMALL);
/// assert_eq!(error.ntstatus(), STATUS_ACCESS_DENIED);
/// ```
pub type StatusResult = Result<Status>;

/// A macro for converting a NTSTATUS code into an Ok containing a value of any type.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::{STATUS_SUCCESS, NTSTATUS};
/// use kerror::{krok, Error, Status, StatusResult};
///
/// let status = Status::new(STATUS_SUCCESS);
/// assert_eq!(krok!(status), StatusResult::Ok(status));
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
/// use kerror::{krokret, Error, Status, StatusResult};
///
/// fn example_ret_status() -> kerror::StatusResult {
///     krokret!(Status::SUCCESS);
/// }
/// assert_eq!(example_ret_status(), StatusResult::Ok(Status::SUCCESS));
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
    use windows_sys::Win32::Foundation::{
        STATUS_ACCESS_DENIED, STATUS_BUFFER_OVERFLOW, STATUS_OBJECT_NAME_EXISTS, STATUS_PENDING,
        STATUS_TIMEOUT,
    };

    /// Customer-defined error; leading nibble `0xE`.
    const CUSTOMER_ERROR: NTSTATUS = layout::from_bits(0xE000_0001);
    /// No real `STATUS_*` constant has a non-zero facility, so the facility
    /// mask can only be exercised with a synthetic value.
    const FACILITY_0X23: NTSTATUS = layout::from_bits(0xC023_0001);

    #[test]
    fn test_nt_status_result() {
        let success: StatusResult = Ok(Status::SUCCESS);
        let carried: StatusResult = Ok(Status::new(STATUS_BUFFER_OVERFLOW));
        let error: StatusResult = Err(Error::from_ntstatus(STATUS_ACCESS_DENIED));

        assert_eq!(success.ntstatus(), STATUS_SUCCESS);
        assert_eq!(carried.ntstatus(), STATUS_BUFFER_OVERFLOW);
        assert_eq!(error.ntstatus(), STATUS_ACCESS_DENIED);
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

    #[test]
    fn severity_classifies_every_class() {
        assert_eq!(Severity::from_ntstatus(STATUS_SUCCESS), Severity::Success);
        assert_eq!(Severity::from_ntstatus(STATUS_PENDING), Severity::Success);
        assert_eq!(Severity::from_ntstatus(STATUS_TIMEOUT), Severity::Success);
        assert_eq!(
            Severity::from_ntstatus(STATUS_OBJECT_NAME_EXISTS),
            Severity::Information
        );
        assert_eq!(
            Severity::from_ntstatus(STATUS_BUFFER_OVERFLOW),
            Severity::Warning
        );
        assert_eq!(
            Severity::from_ntstatus(STATUS_ACCESS_DENIED),
            Severity::Error
        );
    }

    #[test]
    fn severity_does_not_sign_extend() {
        // `NTSTATUS` is signed; an arithmetic shift would collapse both of
        // these to the same wrong answer. Both inputs must be negative for
        // this test to mean anything, so check that at compile time.
        const { assert!(STATUS_ACCESS_DENIED < 0) };
        const { assert!(STATUS_BUFFER_OVERFLOW < 0) };
        assert_eq!(
            Severity::from_ntstatus(STATUS_ACCESS_DENIED),
            Severity::Error
        );
        assert_eq!(
            Severity::from_ntstatus(STATUS_BUFFER_OVERFLOW),
            Severity::Warning
        );
    }

    #[test]
    fn severity_orders_by_increasing_severity() {
        assert!(Severity::Success < Severity::Information);
        assert!(Severity::Information < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn severity_is_const_evaluable() {
        const SEVERITY: Severity = Severity::from_ntstatus(STATUS_ACCESS_DENIED);
        assert_eq!(SEVERITY, Severity::Error);
    }

    #[test]
    fn facility_extracts_bits_16_to_27() {
        assert_eq!(Error::from_ntstatus(FACILITY_0X23).facility(), 0x023);
        assert_eq!(Error::from_ntstatus(STATUS_ACCESS_DENIED).facility(), 0);
    }

    #[test]
    fn code_extracts_low_16_bits() {
        assert_eq!(Error::from_ntstatus(STATUS_ACCESS_DENIED).code(), 0x0022);
        assert_eq!(Error::from_ntstatus(FACILITY_0X23).code(), 0x0001);
        assert_eq!(Error::from_ntstatus(STATUS_SUCCESS).code(), 0);
    }

    #[test]
    fn is_customer_reads_bit_29() {
        assert!(Error::from_ntstatus(CUSTOMER_ERROR).is_customer());
        assert!(!Error::from_ntstatus(STATUS_ACCESS_DENIED).is_customer());
        // The customer bit must not disturb the severity field.
        assert_eq!(
            Error::from_ntstatus(CUSTOMER_ERROR).severity(),
            Severity::Error
        );
    }

    #[test]
    fn severity_predicates_are_mutually_exclusive() {
        for status in [
            STATUS_SUCCESS,
            STATUS_OBJECT_NAME_EXISTS,
            STATUS_BUFFER_OVERFLOW,
            STATUS_ACCESS_DENIED,
        ] {
            let error = Error::from_ntstatus(status);
            let flags = [
                error.is_success(),
                error.is_information(),
                error.is_warning(),
                error.is_error(),
            ];

            assert_eq!(
                flags.iter().filter(|set| **set).count(),
                1,
                "exactly one predicate must hold for {status:#010x}"
            );
        }
    }

    #[test]
    fn error_constructors_are_const_evaluable() {
        const ERROR: Error = Error::from_ntstatus(STATUS_ACCESS_DENIED);
        const STATUS: NTSTATUS = ERROR.ntstatus();
        const SEVERITY: Severity = ERROR.severity();
        const { assert!(ERROR.is(STATUS_ACCESS_DENIED)) };

        assert_eq!(STATUS, STATUS_ACCESS_DENIED);
        assert_eq!(SEVERITY, Severity::Error);
    }

    #[test]
    fn from_bits_matches_from_ntstatus() {
        assert_eq!(
            Error::from_bits(0xC000_0022),
            Error::from_ntstatus(STATUS_ACCESS_DENIED)
        );
        assert_eq!(Error::from_bits(0), Error::from_ntstatus(STATUS_SUCCESS));
        assert_eq!(
            Error::from_bits(u32::MAX),
            Error::from_ntstatus(layout::from_bits(u32::MAX))
        );
    }

    #[test]
    fn from_bits_is_const_evaluable() {
        const CUSTOM: Error = Error::from_bits(0xE000_0001);

        const { assert!(CUSTOM.is_customer()) };
        const { assert!(CUSTOM.is_error()) };
        assert_eq!(CUSTOM.code(), 0x0001);
    }

    #[test]
    fn error_converts_into_ntstatus() {
        let error = Error::from_ntstatus(STATUS_ACCESS_DENIED);

        let status: NTSTATUS = error.into();
        assert_eq!(status, STATUS_ACCESS_DENIED);
        assert_eq!(NTSTATUS::from(error), STATUS_ACCESS_DENIED);
    }

    #[test]
    fn error_implements_the_nt_status_trait() {
        // A generic bound is what the inherent method cannot satisfy; before
        // `impl NtStatus for Error` this did not compile.
        fn to_status<T: NtStatus>(value: &T) -> NTSTATUS {
            value.ntstatus()
        }

        let error = Error::from_ntstatus(STATUS_ACCESS_DENIED);
        let failure: Result<()> = Err(error);
        let success: Result<()> = Ok(());

        assert_eq!(to_status(&error), STATUS_ACCESS_DENIED);
        assert_eq!(to_status(&failure), STATUS_ACCESS_DENIED);
        assert_eq!(to_status(&success), STATUS_SUCCESS);
    }

    #[test]
    fn status_result_carries_non_success_through_ntstatus() {
        // The whole purpose of StatusResult: a non-success status returned as
        // `Ok` must survive extraction rather than collapsing to success.
        let pending: StatusResult = Ok(Status::new(STATUS_PENDING));

        assert_eq!(pending.ntstatus(), STATUS_PENDING);
        // The discarding counterpart is honest about what it does.
        assert_eq!(pending.ntstatus_or_success(), STATUS_SUCCESS);
    }

    #[test]
    fn ntstatus_or_success_handles_arbitrary_payloads() {
        let data: Result<[u8; 4]> = Ok([1, 2, 3, 4]);
        let failed: Result<[u8; 4]> = Err(Error::from_ntstatus(STATUS_ACCESS_DENIED));

        assert_eq!(data.ntstatus_or_success(), STATUS_SUCCESS);
        assert_eq!(failed.ntstatus_or_success(), STATUS_ACCESS_DENIED);
    }

    #[test]
    fn status_and_error_accessors_agree() {
        // Both types share one generated definition; this pins that they can
        // never drift apart.
        for bits in [
            0x0000_0000u32,
            0x4000_0000,
            0x8000_0005,
            0xC000_0022,
            0xC014_000F,
            0xE000_0001,
            u32::MAX,
        ] {
            let status = Status::from_bits(bits);
            let error = Error::from_bits(bits);

            assert_eq!(status.severity(), error.severity(), "{bits:#010X}");
            assert_eq!(status.facility(), error.facility(), "{bits:#010X}");
            assert_eq!(status.code(), error.code(), "{bits:#010X}");
            assert_eq!(status.is_customer(), error.is_customer(), "{bits:#010X}");
            assert_eq!(status.is_success(), error.is_success(), "{bits:#010X}");
            assert_eq!(
                status.is_information(),
                error.is_information(),
                "{bits:#010X}"
            );
            assert_eq!(status.is_warning(), error.is_warning(), "{bits:#010X}");
            assert_eq!(status.is_error(), error.is_error(), "{bits:#010X}");
        }
    }

    #[test]
    fn status_accessors_are_const_evaluable() {
        const CARRIED: Status = Status::new(STATUS_BUFFER_OVERFLOW);
        const RAW: NTSTATUS = CARRIED.ntstatus();
        const SEVERITY: Severity = CARRIED.severity();

        const { assert!(CARRIED.is_warning()) };
        const { assert!(Status::from_bits(0xE000_0001).is_customer()) };

        assert_eq!(RAW, STATUS_BUFFER_OVERFLOW);
        assert_eq!(SEVERITY, Severity::Warning);
        assert_eq!(Status::SUCCESS.ntstatus(), STATUS_SUCCESS);
    }

    #[test]
    fn status_converts_both_ways() {
        let status: Status = STATUS_PENDING.into();
        assert_eq!(status.ntstatus(), STATUS_PENDING);

        let raw: NTSTATUS = status.into();
        assert_eq!(raw, STATUS_PENDING);

        assert_eq!(
            Status::from_bits(0xC000_0022),
            Status::new(STATUS_ACCESS_DENIED)
        );
    }

    #[test]
    fn status_is_transparent_over_ntstatus() {
        assert_eq!(
            core::mem::size_of::<Status>(),
            core::mem::size_of::<NTSTATUS>()
        );
        assert_eq!(
            core::mem::align_of::<Status>(),
            core::mem::align_of::<NTSTATUS>()
        );
    }

    #[test]
    fn error_is_transparent_over_ntstatus() {
        // `#[repr(transparent)]` is a documented guarantee: `Error` must be
        // usable anywhere an `NTSTATUS` is, including across FFI.
        assert_eq!(
            core::mem::size_of::<Error>(),
            core::mem::size_of::<NTSTATUS>()
        );
        assert_eq!(
            core::mem::align_of::<Error>(),
            core::mem::align_of::<NTSTATUS>()
        );
    }

    #[test]
    fn severity_discriminants_match_the_wire_encoding() {
        // `#[repr(u32)]` values must equal the two-bit field they come from.
        assert_eq!(Severity::Success as u32, 0);
        assert_eq!(Severity::Information as u32, 1);
        assert_eq!(Severity::Warning as u32, 2);
        assert_eq!(Severity::Error as u32, 3);
    }

    #[test]
    fn result_into_result_converts_foreign_error_type() {
        // Exercises the blanket `IntoResult for Result<T, E> where E: IntoError`,
        // which the NTSTATUS-only tests never reach.
        let ok: core::result::Result<u8, NTSTATUS> = Ok(7);
        let err: core::result::Result<u8, NTSTATUS> = Err(STATUS_ACCESS_DENIED);

        assert_eq!(ok.into_result(), Ok(7));
        assert_eq!(
            err.into_result(),
            Err(Error::from_ntstatus(STATUS_ACCESS_DENIED))
        );
    }

    #[test]
    fn error_is_usable_as_dyn_error() {
        let error = Error::from_ntstatus(STATUS_ACCESS_DENIED);
        let dynamic: &dyn core::error::Error = &error;

        assert!(dynamic.source().is_none());
    }

    /// Fixed-size `core::fmt::Write` sink, so `Display` can be exercised
    /// without `alloc` in every feature configuration.
    struct FmtBuf {
        bytes: [u8; 32],
        len: usize,
    }

    impl core::fmt::Write for FmtBuf {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            let end = self.len + text.len();
            if end > self.bytes.len() {
                return Err(core::fmt::Error);
            }
            self.bytes[self.len..end].copy_from_slice(text.as_bytes());
            self.len = end;
            Ok(())
        }
    }

    fn rendered(status: NTSTATUS) -> FmtBuf {
        use core::fmt::Write;

        let mut buf = FmtBuf {
            bytes: [0; 32],
            len: 0,
        };
        write!(buf, "{}", Error::from_ntstatus(status)).unwrap();
        buf
    }

    fn rendered_str(buf: &FmtBuf) -> &str {
        core::str::from_utf8(&buf.bytes[..buf.len]).unwrap()
    }

    #[test]
    fn display_zero_pads_to_eight_hex_digits() {
        // Regression: `{:#08x}` counted the `0x` prefix inside the width and
        // rendered STATUS_SUCCESS as `0x000000`.
        assert_eq!(rendered_str(&rendered(STATUS_SUCCESS)), "0x00000000");
        assert_eq!(rendered_str(&rendered(STATUS_TIMEOUT)), "0x00000102");
        assert_eq!(
            rendered_str(&rendered(STATUS_OBJECT_NAME_EXISTS)),
            "0x40000000"
        );
        assert_eq!(
            rendered_str(&rendered(STATUS_BUFFER_OVERFLOW)),
            "0x80000005"
        );
        assert_eq!(rendered_str(&rendered(STATUS_ACCESS_DENIED)), "0xC0000022");
    }

    #[test]
    fn display_is_uppercase_and_always_ten_characters() {
        for status in [
            STATUS_SUCCESS,
            STATUS_TIMEOUT,
            STATUS_PENDING,
            STATUS_OBJECT_NAME_EXISTS,
            STATUS_BUFFER_OVERFLOW,
            STATUS_ACCESS_DENIED,
            CUSTOMER_ERROR,
            FACILITY_0X23,
            layout::from_bits(u32::MAX),
        ] {
            let buf = rendered(status);
            let text = rendered_str(&buf);

            assert_eq!(text.len(), 10, "wrong width for {status:#010X}");
            assert!(text.starts_with("0x"), "missing prefix for {status:#010X}");
            assert!(
                !text[2..].chars().any(char::is_lowercase),
                "expected uppercase digits, got {text}"
            );
        }
    }

    #[test]
    fn is_success_reports_severity_not_status_success() {
        // `STATUS_PENDING` has Success severity but is still an error here.
        let pending = Error::from_ntstatus(STATUS_PENDING);

        assert!(pending.is_success());
        assert!(!pending.is(STATUS_SUCCESS));
        assert!(STATUS_PENDING.into_result().is_err());
    }
}
