use nt_string::NtStringError;
use windows_sys::Win32::Foundation::{
    STATUS_BUFFER_OVERFLOW, STATUS_INSUFFICIENT_RESOURCES, STATUS_INVALID_PARAMETER,
};

use crate::Error;

impl From<NtStringError> for Error {
    /// Convert [`NtStringError`] to appropriate `Error`
    fn from(error: NtStringError) -> Self {
        match error {
            NtStringError::InsufficientResources => Error(STATUS_INSUFFICIENT_RESOURCES),
            NtStringError::BufferSizeExceedsU16 => Error(STATUS_BUFFER_OVERFLOW),
            _ => Error(STATUS_INVALID_PARAMETER),
        }
    }
}
