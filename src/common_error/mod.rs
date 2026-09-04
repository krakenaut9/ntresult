//! Conversions from common Rust error types into [`Error`](crate::Error).
//!
//! Each conversion maps a `core` or `alloc` error onto the `NTSTATUS` code that
//! best describes it, so a Rust-level failure can propagate through `?` and be
//! returned to the kernel as a status code.
//!
//! | Error type | `NTSTATUS` | Feature |
//! |---|---|---|
//! | `core::num::TryFromIntError` | `STATUS_INTEGER_OVERFLOW` | always |
//! | `core::net::AddrParseError` | `STATUS_INVALID_ADDRESS` | always |
//! | `alloc::collections::TryReserveError` | `STATUS_INSUFFICIENT_RESOURCES` | `alloc` |
//! | `core::alloc::AllocError` | `STATUS_INSUFFICIENT_RESOURCES` | `allocator-api` |
//!
//! Each submodule documents its own conversions and carries runnable examples.
//! Examples live on the individual `impl` blocks rather than here, so that every
//! example is compiled under exactly the feature that provides it.
//!
//! More types will be added in the future.

// `AllocError` lives in `core`, so this module is needed even without `alloc`.
// No `doc(cfg)` here: rustdoc would AND it into each impl's own badge.
#[cfg(any(feature = "alloc", feature = "allocator-api"))]
pub mod allocation;

pub mod addr_parse;

pub mod integer;
