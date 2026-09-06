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
