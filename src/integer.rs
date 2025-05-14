use core::num::TryFromIntError;
use windows_sys::Win32::Foundation::STATUS_INTEGER_OVERFLOW;

use crate::Error;

impl From<TryFromIntError> for Error {
    /// Convert [`TryFromIntError`] to `Error(STATUS_INTEGER_OVERFLOW)`
    fn from(_: TryFromIntError) -> Self {
        Error(STATUS_INTEGER_OVERFLOW)
    }
}
