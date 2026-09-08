//! Conversions from allocation-related error types.
//!
//! | Error type | `NTSTATUS` | Feature | Toolchain |
//! |---|---|---|---|
//! | `alloc::collections::TryReserveError` | `STATUS_INSUFFICIENT_RESOURCES` | `alloc` | stable |
//! | `core::alloc::AllocError` | `STATUS_INSUFFICIENT_RESOURCES` | `allocator-api` | nightly |
//!
//! The two features are independent. `allocator-api` does not require `alloc`,
//! because `AllocError` is defined in `core` -- a driver using a custom
//! `Allocator` without a global allocator can still use the conversion.

use crate::Error;
use windows_sys::Win32::Foundation::STATUS_INSUFFICIENT_RESOURCES;

#[cfg(feature = "alloc")]
use alloc::collections::TryReserveError;

#[cfg(feature = "allocator-api")]
use core::alloc::AllocError;

/// Convert [`TryReserveError`] into `Error(STATUS_INSUFFICIENT_RESOURCES)`.
///
/// A failed reservation is reported as `STATUS_INSUFFICIENT_RESOURCES`
/// regardless of whether it failed because of capacity overflow or because the
/// allocator refused the request; `TryReserveError` is opaque and the
/// distinction is not actionable for a caller returning an `NTSTATUS`.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::STATUS_INSUFFICIENT_RESOURCES;
///
/// fn allocate(len: usize) -> kerror::Result<Vec<u8>> {
///     let mut buf = Vec::new();
///     buf.try_reserve(len)?;
///     Ok(buf)
/// }
///
/// assert!(allocate(16).is_ok());
///
/// let err = allocate(usize::MAX).unwrap_err();
/// assert!(err.is(STATUS_INSUFFICIENT_RESOURCES));
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<TryReserveError> for Error {
    #[inline]
    fn from(_: TryReserveError) -> Self {
        Error(STATUS_INSUFFICIENT_RESOURCES)
    }
}

/// Convert [`AllocError`] into `Error(STATUS_INSUFFICIENT_RESOURCES)`.
///
/// Requires the `allocator-api` feature and a nightly compiler.
///
/// # Examples
/// ```
/// #![feature(allocator_api)]
/// use core::alloc::AllocError;
/// use kerror::Error;
/// use windows_sys::Win32::Foundation::STATUS_INSUFFICIENT_RESOURCES;
///
/// let err = Error::from(AllocError);
/// assert!(err.is(STATUS_INSUFFICIENT_RESOURCES));
/// ```
#[cfg(feature = "allocator-api")]
#[cfg_attr(docsrs, doc(cfg(feature = "allocator-api")))]
impl From<AllocError> for Error {
    #[inline]
    fn from(_: AllocError) -> Self {
        Error(STATUS_INSUFFICIENT_RESOURCES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    #[test]
    fn try_reserve_error_maps_to_insufficient_resources() {
        let err: TryReserveError = alloc::vec::Vec::<u8>::new()
            .try_reserve(usize::MAX)
            .unwrap_err();

        assert_eq!(Error::from(err).ntstatus(), STATUS_INSUFFICIENT_RESOURCES);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn try_reserve_error_propagates_through_question_mark() {
        fn allocate(len: usize) -> crate::Result<alloc::vec::Vec<u8>> {
            let mut buf = alloc::vec::Vec::new();
            buf.try_reserve(len)?;
            Ok(buf)
        }

        assert!(allocate(16).is_ok());
        assert!(
            allocate(usize::MAX)
                .unwrap_err()
                .is(STATUS_INSUFFICIENT_RESOURCES)
        );
    }

    #[cfg(feature = "allocator-api")]
    #[test]
    fn alloc_error_maps_to_insufficient_resources() {
        assert_eq!(
            Error::from(AllocError).ntstatus(),
            STATUS_INSUFFICIENT_RESOURCES
        );
    }

    #[cfg(feature = "allocator-api")]
    #[test]
    fn alloc_error_propagates_through_question_mark() {
        fn fallible(fail: bool) -> crate::Result<u32> {
            if fail {
                Err(AllocError)?;
            }
            Ok(7)
        }

        assert_eq!(fallible(false).unwrap(), 7);
        assert!(
            fallible(true)
                .unwrap_err()
                .is(STATUS_INSUFFICIENT_RESOURCES)
        );
    }
}
