use crate::Error;
use core::net::AddrParseError;
use windows_sys::Win32::Foundation::STATUS_INVALID_ADDRESS;

impl From<AddrParseError> for Error {
    /// Convert [`AddrParseError`] to appropriate `Error`
    fn from(_: AddrParseError) -> Self {
        Error(STATUS_INVALID_ADDRESS)
    }
}
