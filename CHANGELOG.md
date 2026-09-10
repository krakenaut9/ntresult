# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

While the version is below 1.0, a minor bump may contain breaking changes.

## [Unreleased]

## [0.1.0] - 2026-09-10

First public release.

### Added

- `Error`, a `#[repr(transparent)]` wrapper over `NTSTATUS` that implements
  `core::error::Error`.
- `Result<T, E = Error>`, so a signature reads `Result<()>`.
- `Status` and `StatusResult`, for returning a non-success status as a
  deliberate outcome rather than a failure. `Ok(Status::from_ntstatus(
  STATUS_BUFFER_TOO_SMALL))` keeps that status instead of collapsing it to
  `STATUS_SUCCESS`.
- `Severity`, ordered so that `severity() >= Severity::Warning` works.
- Status inspection on both `Error` and `Status`: `severity`, `facility`,
  `code`, `is`, `is_customer`, `is_success`, `is_information`, `is_warning`
  and `is_error`. All are `const fn`.
- Constructors `from_ntstatus` and `from_bits`, the latter taking the unsigned
  form status codes are conventionally written in, and the `ntstatus` accessor.
- `IntoResult` and `IntoError` for converting a raw `NTSTATUS`, and `NtStatus`
  and `NtStatusOrSuccess` for collapsing a `Result` back into one.
- Macros `ntok!`, `nterr!` and `ntres!`, plus the `ntok_ret!`, `nterr_ret!` and
  `ntres_ret!` variants that return immediately, and `ntbail!` as an alias for
  `nterr_ret!` under the name used by `anyhow`.
- `Display` and `Debug` rendering a status as `0x` and eight uppercase
  hexadecimal digits, so `unwrap` and `assert_eq!` output stays readable.
- Conversions into `Error` in `common_error`: `TryFromIntError` and
  `AddrParseError` unconditionally, `TryReserveError` behind the `alloc`
  feature, and `AllocError` behind `allocator-api`.

### Notes

- `#![no_std]` with no default features. Enabling `alloc` links the `alloc`
  crate, which makes rustc require a `#[global_allocator]` from the final
  artifact even if nothing allocates, so it is opt-in.
- `allocator-api` requires a nightly compiler; everything else builds on
  stable. It does not imply `alloc`.
- No `unsafe` code, enforced with `#![forbid(unsafe_code)]`.
- Minimum supported Rust version is 1.85.

[Unreleased]: https://github.com/krakenaut9/ntresult/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/krakenaut9/ntresult/releases/tag/v0.1.0
