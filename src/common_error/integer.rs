//! Conversions from integer conversion error types.
//!
//! | Error type | `NTSTATUS` |
//! |---|---|
//! | `core::num::TryFromIntError` | `STATUS_INTEGER_OVERFLOW` |

use crate::Error;
use core::num::TryFromIntError;
use windows_sys::Win32::Foundation::STATUS_INTEGER_OVERFLOW;

/// Convert [`TryFromIntError`] into `Error(STATUS_INTEGER_OVERFLOW)`.
///
/// `TryFromIntError` covers both directions of a failed narrowing conversion --
/// a value too large for the target type and a negative value converted to an
/// unsigned type. Both map to `STATUS_INTEGER_OVERFLOW`.
///
/// # Examples
/// ```
/// use windows_sys::Win32::Foundation::STATUS_INTEGER_OVERFLOW;
///
/// fn narrow(value: u32) -> ntresult::Result<u8> {
///     Ok(u8::try_from(value)?)
/// }
///
/// assert_eq!(narrow(42).unwrap(), 42);
///
/// let err = narrow(256).unwrap_err();
/// assert!(err.is(STATUS_INTEGER_OVERFLOW));
/// ```
impl From<TryFromIntError> for Error {
    #[inline]
    fn from(_: TryFromIntError) -> Self {
        Self(STATUS_INTEGER_OVERFLOW)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_int_error_maps_to_integer_overflow() {
        let err: TryFromIntError = u8::try_from(256u32).unwrap_err();

        assert_eq!(Error::from(err).ntstatus(), STATUS_INTEGER_OVERFLOW);
    }

    #[test]
    fn negative_to_unsigned_maps_to_integer_overflow() {
        let err: TryFromIntError = u32::try_from(-1i32).unwrap_err();

        assert_eq!(Error::from(err).ntstatus(), STATUS_INTEGER_OVERFLOW);
    }

    #[test]
    fn propagates_through_question_mark() {
        fn narrow(value: i32) -> crate::Result<u8> {
            Ok(u8::try_from(value)?)
        }

        assert_eq!(narrow(42).unwrap(), 42);
        assert!(narrow(256).unwrap_err().is(STATUS_INTEGER_OVERFLOW));
        assert!(narrow(-1).unwrap_err().is(STATUS_INTEGER_OVERFLOW));
    }
}
