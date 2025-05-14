use hashbrown::TryReserveError;
use windows_sys::Win32::Foundation::STATUS_INSUFFICIENT_RESOURCES;

use crate::Error;

impl From<TryReserveError> for Error {
    /// Convert [`hashbrown::TryReserveError`] to appropriate `Error`
    fn from(_: TryReserveError) -> Self {
        Error(STATUS_INSUFFICIENT_RESOURCES)
    }
}
