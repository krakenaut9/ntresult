//! Conversions from address parsing error types.
//!
//! | Error type | `NTSTATUS` |
//! |---|---|
//! | `core::net::AddrParseError` | `STATUS_INVALID_ADDRESS` |

use crate::Error;
use core::net::AddrParseError;
use windows_sys::Win32::Foundation::STATUS_INVALID_ADDRESS;

/// Convert [`AddrParseError`] into `Error(STATUS_INVALID_ADDRESS)`.
///
/// # Examples
/// ```
/// use core::net::IpAddr;
/// use windows_sys::Win32::Foundation::STATUS_INVALID_ADDRESS;
///
/// fn parse(addr: &str) -> ntresult::Result<IpAddr> {
///     Ok(addr.parse::<IpAddr>()?)
/// }
///
/// assert!(parse("127.0.0.1").is_ok());
///
/// let err = parse("not an address").unwrap_err();
/// assert!(err.is(STATUS_INVALID_ADDRESS));
/// ```
impl From<AddrParseError> for Error {
    #[inline]
    fn from(_: AddrParseError) -> Self {
        Self(STATUS_INVALID_ADDRESS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn addr_parse_error_maps_to_invalid_address() {
        let err: AddrParseError = "not an address".parse::<IpAddr>().unwrap_err();

        assert_eq!(Error::from(err).ntstatus(), STATUS_INVALID_ADDRESS);
    }

    #[test]
    fn propagates_through_question_mark() {
        fn parse(addr: &str) -> crate::Result<IpAddr> {
            Ok(addr.parse::<IpAddr>()?)
        }

        assert_eq!(parse("127.0.0.1").unwrap(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(parse("::1").unwrap(), IpAddr::V6(Ipv6Addr::LOCALHOST));
        assert!(
            parse("not an address")
                .unwrap_err()
                .is(STATUS_INVALID_ADDRESS)
        );
        assert!(parse("999.0.0.1").unwrap_err().is(STATUS_INVALID_ADDRESS));
    }
}
