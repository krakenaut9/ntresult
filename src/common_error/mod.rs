//! Conversions from common Rust error types into [`Error`](crate::Error).
//!
//! Each conversion maps a `core` or `alloc` error onto the `NTSTATUS` code that
//! best describes it, so a Rust-level failure can propagate through `?` and be
//! returned to the kernel as a status code.
//!
//! Every conversion sits behind its own feature flag:
//!
//! | Error type | `NTSTATUS` | Feature | Toolchain |
//! |---|---|---|---|
//! | `alloc::collections::TryReserveError` | `STATUS_INSUFFICIENT_RESOURCES` | `alloc` | stable |
//! | `core::num::TryFromIntError` | `STATUS_INTEGER_OVERFLOW` | `integer` | stable |
//! | `core::net::AddrParseError` | `STATUS_INVALID_ADDRESS` | `addr-parse` | stable |
//! | `core::alloc::AllocError` | `STATUS_INSUFFICIENT_RESOURCES` | `allocator-api` | **nightly** |
//!
//! Each submodule documents its own conversions and carries runnable examples.
//! Examples live on the individual `impl` blocks rather than here, so that every
//! example is compiled under exactly the feature that provides it.
//!
//! More types will be added in the future.

// `AllocError` comes from `core`, so this module is also needed when `alloc`
// itself is off but `allocator-api` is on.
#[cfg(any(feature = "alloc", feature = "allocator-api"))]
pub mod alloc;

#[cfg(feature = "addr-parse")]
pub mod addr_parse;

#[cfg(feature = "integer")]
pub mod integer;
