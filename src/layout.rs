//! Bit layout of an `NTSTATUS` value.
//!
//! ```text
//!  31 30 | 29 | 28 | 27 ------ 16 | 15 ------- 0
//!   Sev  | C  | R  |   Facility   |     Code
//! ```

use windows_sys::Win32::Foundation::NTSTATUS;

/// A single field within an `NTSTATUS`, as a bit offset and width.
pub(crate) struct Field {
    shift: u32,
    mask: u32,
}

impl Field {
    /// Extract this field from `status`.
    #[inline]
    #[allow(clippy::cast_sign_loss, reason = "the raw bit pattern is wanted")]
    pub(crate) const fn get(&self, status: NTSTATUS) -> u32 {
        // `NTSTATUS` is signed, so cast before shifting: `>>` on a negative
        // value sign-extends and would corrupt every field above bit 15.
        ((status as u32) >> self.shift) & self.mask
    }
}

/// Reinterpret a raw 32-bit pattern as an `NTSTATUS`.
///
/// Status codes are written as unsigned hex by convention, but `NTSTATUS` is
/// signed, so every value with the top bit set has to wrap.
#[inline]
#[allow(clippy::cast_possible_wrap, reason = "wrapping is the intent")]
pub(crate) const fn from_bits(bits: u32) -> NTSTATUS {
    bits as i32
}

pub(crate) const SEVERITY: Field = Field {
    shift: 30,
    mask: 0b11,
};

pub(crate) const CUSTOMER: Field = Field {
    shift: 29,
    mask: 0b1,
};

pub(crate) const FACILITY: Field = Field {
    shift: 16,
    mask: 0xFFF,
};

pub(crate) const CODE: Field = Field {
    shift: 0,
    mask: 0xFFFF,
};

/// Generate the `NTSTATUS` field accessors for a `#[repr(transparent)]` newtype
/// over an `NTSTATUS`, so `Error` and `Status` share one definition.
///
/// `$ctor` is the constructor path as a string, used only to build doc examples.
macro_rules! status_accessors {
    ($ty:ident, $ctor:literal) => {
        impl $ty {
            #[doc = "Retrieve the severity class encoded in the status."]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = "```"]
            #[doc = concat!("use kerror::{", stringify!($ty), ", Severity};")]
            #[doc = "use windows_sys::Win32::Foundation::STATUS_ACCESS_DENIED;"]
            #[doc = ""]
            #[doc = concat!("let value = ", $ctor, "(STATUS_ACCESS_DENIED);")]
            #[doc = "assert_eq!(value.severity(), Severity::Error);"]
            #[doc = "```"]
            #[must_use]
            #[inline]
            pub const fn severity(self) -> crate::Severity {
                crate::Severity::from_ntstatus(self.0)
            }

            #[doc = "Retrieve the facility code (bits 16-27), identifying the"]
            #[doc = "subsystem the status originates from."]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = "```"]
            #[doc = concat!("use kerror::", stringify!($ty), ";")]
            #[doc = "use windows_sys::Win32::Foundation::STATUS_ACPI_INVALID_DATA;"]
            #[doc = ""]
            #[doc = "// STATUS_ACPI_INVALID_DATA is 0xC014000F"]
            #[doc = concat!("let value = ", $ctor, "(STATUS_ACPI_INVALID_DATA);")]
            #[doc = "assert_eq!(value.facility(), 0x014);"]
            #[doc = "```"]
            #[must_use]
            #[inline]
            pub const fn facility(self) -> u32 {
                crate::layout::FACILITY.get(self.0)
            }

            #[doc = "Retrieve the status code (bits 0-15), the facility-specific"]
            #[doc = "identifier."]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = "```"]
            #[doc = concat!("use kerror::", stringify!($ty), ";")]
            #[doc = "use windows_sys::Win32::Foundation::STATUS_ACCESS_DENIED;"]
            #[doc = ""]
            #[doc = "// STATUS_ACCESS_DENIED is 0xC0000022"]
            #[doc = concat!("let value = ", $ctor, "(STATUS_ACCESS_DENIED);")]
            #[doc = "assert_eq!(value.code(), 0x0022);"]
            #[doc = "```"]
            #[must_use]
            #[inline]
            pub const fn code(self) -> u32 {
                crate::layout::CODE.get(self.0)
            }

            #[doc = "Check whether the status is customer-defined (bit 29) rather"]
            #[doc = "than defined by Microsoft."]
            #[doc = ""]
            #[doc = "Third parties set this bit when minting their own status codes,"]
            #[doc = "which guarantees the value can never collide with a current or"]
            #[doc = "future Microsoft-defined code. Customer-defined values are"]
            #[doc = "recognisable by their leading nibble: `0x2` success, `0x6`"]
            #[doc = "informational, `0xA` warning, `0xE` error."]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = "```"]
            #[doc = concat!("use kerror::", stringify!($ty), ";")]
            #[doc = "use windows_sys::Win32::Foundation::STATUS_ACCESS_DENIED;"]
            #[doc = ""]
            #[doc = concat!("assert!(!", $ctor, "(STATUS_ACCESS_DENIED).is_customer());")]
            #[doc = concat!("assert!(", stringify!($ty), "::from_bits(0xE000_0001).is_customer());")]
            #[doc = "```"]
            #[must_use]
            #[inline]
            pub const fn is_customer(self) -> bool {
                crate::layout::CUSTOMER.get(self.0) != 0
            }

            #[doc = "Check whether the severity class is [`Severity::Success`]."]
            #[doc = ""]
            #[doc = "# Warning"]
            #[doc = ""]
            #[doc = "This reports the severity field, **not** whether the value equals"]
            #[doc = "`STATUS_SUCCESS`. The `Success` severity class also contains codes"]
            #[doc = "such as `STATUS_PENDING` (`0x00000103`) and `STATUS_TIMEOUT`"]
            #[doc = "(`0x00000102`), which this crate does not treat as success."]
            #[doc = ""]
            #[doc = "[`Severity::Success`]: crate::Severity::Success"]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = "```"]
            #[doc = concat!("use kerror::", stringify!($ty), ";")]
            #[doc = "use windows_sys::Win32::Foundation::{STATUS_ACCESS_DENIED, STATUS_PENDING};"]
            #[doc = ""]
            #[doc = concat!("assert!(!", $ctor, "(STATUS_ACCESS_DENIED).is_success());")]
            #[doc = concat!("assert!(", $ctor, "(STATUS_PENDING).is_success());")]
            #[doc = "```"]
            #[must_use]
            #[inline]
            pub const fn is_success(self) -> bool {
                matches!(self.severity(), crate::Severity::Success)
            }

            #[doc = "Check whether the severity class is [`Severity::Information`]."]
            #[doc = ""]
            #[doc = "[`Severity::Information`]: crate::Severity::Information"]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = "```"]
            #[doc = concat!("use kerror::", stringify!($ty), ";")]
            #[doc = "use windows_sys::Win32::Foundation::STATUS_OBJECT_NAME_EXISTS;"]
            #[doc = ""]
            #[doc = concat!("assert!(", $ctor, "(STATUS_OBJECT_NAME_EXISTS).is_information());")]
            #[doc = "```"]
            #[must_use]
            #[inline]
            pub const fn is_information(self) -> bool {
                matches!(self.severity(), crate::Severity::Information)
            }

            #[doc = "Check whether the severity class is [`Severity::Warning`]."]
            #[doc = ""]
            #[doc = "[`Severity::Warning`]: crate::Severity::Warning"]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = "```"]
            #[doc = concat!("use kerror::", stringify!($ty), ";")]
            #[doc = "use windows_sys::Win32::Foundation::STATUS_BUFFER_OVERFLOW;"]
            #[doc = ""]
            #[doc = concat!("assert!(", $ctor, "(STATUS_BUFFER_OVERFLOW).is_warning());")]
            #[doc = "```"]
            #[must_use]
            #[inline]
            pub const fn is_warning(self) -> bool {
                matches!(self.severity(), crate::Severity::Warning)
            }

            #[doc = "Check whether the severity class is [`Severity::Error`]."]
            #[doc = ""]
            #[doc = "[`Severity::Error`]: crate::Severity::Error"]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = "```"]
            #[doc = concat!("use kerror::", stringify!($ty), ";")]
            #[doc = "use windows_sys::Win32::Foundation::STATUS_ACCESS_DENIED;"]
            #[doc = ""]
            #[doc = concat!("assert!(", $ctor, "(STATUS_ACCESS_DENIED).is_error());")]
            #[doc = "```"]
            #[must_use]
            #[inline]
            pub const fn is_error(self) -> bool {
                matches!(self.severity(), crate::Severity::Error)
            }
        }
    };
}

pub(crate) use status_accessors;

#[cfg(test)]
mod tests {
    use super::*;

    /// Every bit of an `NTSTATUS` belongs to exactly one field, except bit 28,
    /// which is reserved and deliberately unmapped.
    #[test]
    fn fields_tile_the_word_without_overlap() {
        const RESERVED_BIT: u32 = 1 << 28;

        let fields = [&SEVERITY, &CUSTOMER, &FACILITY, &CODE];
        let mut covered: u32 = 0;

        for field in fields {
            let occupied = field.mask << field.shift;
            assert_eq!(covered & occupied, 0, "fields overlap at {occupied:#034b}");
            covered |= occupied;
        }

        assert_eq!(covered, !RESERVED_BIT);
    }

    #[test]
    fn get_is_unsigned_and_masked() {
        // All-ones status: every field must saturate, none may bleed together.
        let all_ones = from_bits(u32::MAX);

        assert_eq!(SEVERITY.get(all_ones), 0b11);
        assert_eq!(CUSTOMER.get(all_ones), 0b1);
        assert_eq!(FACILITY.get(all_ones), 0xFFF);
        assert_eq!(CODE.get(all_ones), 0xFFFF);
    }

    #[test]
    fn fields_are_independent() {
        // 0xC0230001: severity 0b11, customer 0, facility 0x023, code 0x0001.
        let status = from_bits(0xC023_0001);

        assert_eq!(SEVERITY.get(status), 0b11);
        assert_eq!(CUSTOMER.get(status), 0);
        assert_eq!(FACILITY.get(status), 0x023);
        assert_eq!(CODE.get(status), 0x0001);
    }

    #[test]
    fn get_is_const_evaluable() {
        const FACILITY_BITS: u32 = FACILITY.get(from_bits(0xC023_0001));
        assert_eq!(FACILITY_BITS, 0x023);
    }

    #[test]
    fn from_bits_round_trips_through_the_sign_boundary() {
        assert_eq!(from_bits(0), 0);
        assert_eq!(from_bits(0x7FFF_FFFF), i32::MAX);
        assert_eq!(from_bits(0x8000_0000), i32::MIN);
        assert_eq!(from_bits(u32::MAX), -1);
    }
}
