# ntresult

Lightweight `NTSTATUS`-based error handling for Windows kernel-mode Rust code.

`ntresult` provides a minimal and idiomatic interface for working with Windows `NTSTATUS` values in Rust, designed specifically for `#![no_std]` and kernel-mode environments. It bridges native Windows status codes with Rust's `Result` type without introducing unnecessary abstraction or overhead.

---

## Highlights

-  `#![no_std]` support (optional `alloc`)
-  Builds on **stable** Rust (nightly only for the opt-in `allocator-api` feature)
-  Zero-cost abstraction over `NTSTATUS`
-  Transparent `Error` and `Status` wrappers
-  Idiomatic `Result`-based API
-  Explicit handling of expected status values
-  Severity, facility and code inspection
-  Shorthand macros for the common conversions
-  No memory allocations
-  No `unsafe` code

---

## Feature flags

All features are additive: each one only adds a `From` conversion into `Error`.
Enabling a feature never changes the behaviour of an existing conversion.

| Feature         | Default | Toolchain   | Conversion added                      |
| --------------- | ------- | ----------- | ------------------------------------- |
| `alloc`         | no      | stable      | `alloc::collections::TryReserveError` |
| `allocator-api` | no      | **nightly** | `core::alloc::AllocError`             |

Conversions that need nothing beyond `core` — `core::num::TryFromIntError` and
`core::net::AddrParseError` — are always available and are not gated behind a feature.

**There are no default features.** Enabling `alloc` links the `alloc` crate, which
makes rustc require a `#[global_allocator]` from the final artifact — a driver, a
`staticlib` — *even if nothing ever allocates*. A driver that only wants the
`NTSTATUS` wrapper should not be made to supply one, so `alloc` is opt-in.

`allocator-api` enables the unstable `allocator_api` language feature and therefore
requires a nightly compiler. Everything else builds on stable Rust.

`allocator-api` does **not** imply `alloc`: `AllocError` is defined in `core`, so a
driver using a custom `Allocator` without a global allocator can still use it.

```toml
# stable, no allocator required
ntresult = "0.1"

# stable, plus the TryReserveError conversion
# (needs a #[global_allocator] in the final artifact)
ntresult = { version = "0.1", features = ["alloc"] }

# nightly, everything
ntresult = { version = "0.1", features = ["alloc", "allocator-api"] }
```

---

## Core Types

### `Result<T>`

```rust,ignore
pub type Result<T, E = Error> = core::result::Result<T, E>;
```

### `Error`

```rust,ignore
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Error(pub(crate) NTSTATUS);

impl core::error::Error for Error {}
```

`Debug` and `Display` are hand-written rather than derived — see
[Formatting](#formatting).

### `Status`

```rust,ignore
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Status(NTSTATUS);
```

The same data as `Error`, with the opposite intent: a status that is an expected
outcome rather than a failure. It exposes an identical set of accessors.

## Usage

### Converting NTSTATUS → Result

```rust
use ntresult::IntoResult;
use windows_sys::Win32::Foundation::{NTSTATUS, STATUS_SUCCESS};

// Stand-in for a kernel API returning a raw status.
fn some_kernel_call() -> NTSTATUS {
    STATUS_SUCCESS
}

fn my_func() -> ntresult::Result<()> {
    some_kernel_call().into_result()
}

assert!(my_func().is_ok());
```

| NTSTATUS       | ntresult::Result<()> |
| -------------- | -------------------- |
| STATUS_SUCCESS | Ok(())               |
| any other code | Err(Error(NTSTATUS)) |

`ntresult` intentionally treats all non-`STATUS_SUCCESS` values as errors.
This preserves strict semantics and avoids ambiguity.


### Returning NTSTATUS from Result
```rust
use ntresult::NtStatus;
use windows_sys::Win32::Foundation::STATUS_SUCCESS;

fn driver_fn() -> ntresult::Result<()> {
    Ok(())
}

assert_eq!(driver_fn().ntstatus(), STATUS_SUCCESS);
```

| ntresult::Result<()> | NTSTATUS       |
| -------------------- | -------------- |
| Ok(())               | STATUS_SUCCESS |
| Err(Error(status))   | status         |

### Returning an arbitrary data type

When the `Ok` payload is data rather than a status, use `ntstatus_or_success()`.
The name states that the payload is discarded.

```rust
use ntresult::NtStatusOrSuccess;
use windows_sys::Win32::Foundation::STATUS_SUCCESS;

fn read_register() -> ntresult::Result<u32> {
    Ok(0x1234)
}

// The payload is discarded; only success or the error's status remains.
assert_eq!(read_register().ntstatus_or_success(), STATUS_SUCCESS);
```

| ntresult::Result<T> | NTSTATUS       |
| ------------------- | -------------- |
| Ok(T)               | STATUS_SUCCESS |
| Err(Error(status))  | status         |

### Returning expected non-success statuses

Some kernel APIs use non-success `NTSTATUS` values as valid outcomes (e.g. `STATUS_BUFFER_TOO_SMALL`).

Return them as `Ok` in a `StatusResult`, and they survive extraction:

```rust
use ntresult::{NtStatus, Status, StatusResult};
use windows_sys::Win32::Foundation::STATUS_BUFFER_TOO_SMALL;

fn driver_fn() -> StatusResult {
    Ok(Status::from_ntstatus(STATUS_BUFFER_TOO_SMALL))
}

// The carried status survives; it is not collapsed to STATUS_SUCCESS.
assert_eq!(driver_fn().ntstatus(), STATUS_BUFFER_TOO_SMALL);
```

| StatusResult (= Result<Status>) | NTSTATUS |
| ------------------------------- | -------- |
| Ok(Status(status))              | status   |
| Err(Error(status))              | status   |

`Status` is a newtype rather than a bare `NTSTATUS` because `NTSTATUS` is an
alias for `i32`: without it, a `Result<i32>` carrying a byte count would be
indistinguishable from one carrying a status code.

---

## Inspecting a status

An `NTSTATUS` is not opaque — it packs four fields:

```text
 31 30 | 29 | 28 | 27 ------ 16 | 15 ------- 0
  Sev  | C  | R  |   Facility   |     Code
```

`Error` and `Status` expose the same accessors for all of them.

```rust
use ntresult::{Error, Severity};
use windows_sys::Win32::Foundation::STATUS_ACPI_INVALID_DATA;

// STATUS_ACPI_INVALID_DATA is 0xC014000F
let err = Error::from_ntstatus(STATUS_ACPI_INVALID_DATA);

assert_eq!(err.severity(), Severity::Error);
assert_eq!(err.facility(), 0x014);
assert_eq!(err.code(), 0x000F);
assert!(err.is_error());
assert!(!err.is_customer());
```

`Severity` is ordered, so `err.severity() >= Severity::Warning` works.

Beware `is_success()`: it reports the *severity field*, not whether the status
equals `STATUS_SUCCESS`. `STATUS_PENDING` and `STATUS_TIMEOUT` both have Success
severity while still being errors as far as `into_result()` is concerned.

### Customer-defined status codes

Set bit 29 to mint your own codes — Microsoft guarantees it will never define a
status with that bit set, so yours can never collide. Customer values are
recognisable by their leading nibble: `0x2` success, `0x6` informational,
`0xA` warning, `0xE` error.

```rust
use ntresult::Error;

// `from_bits` takes the unsigned form, so no `as i32` cast is needed.
let mine = Error::from_bits(0xE000_0001);

assert!(mine.is_customer());
assert!(mine.is_error());
assert_eq!(mine.code(), 0x0001);
```

---

## Error Conversion

To turn an `NTSTATUS` straight into an `Error`, use the `IntoError` trait.

```rust
use ntresult::IntoError;
use windows_sys::Win32::Foundation::STATUS_INVALID_PARAMETER;

fn check_len(len: usize) -> ntresult::Result<()> {
    if len == 0 {
        return Err(STATUS_INVALID_PARAMETER.into_error());
    }

    Ok(())
}

assert!(check_len(0).is_err());
assert!(check_len(1).is_ok());
```

### Common error types
The `common_error` module provides conversions from common Rust errors into
`ntresult::Error`, so a Rust-level failure can propagate through `?` and be
returned to the kernel as a status code. Each error maps to the `NTSTATUS`
that best describes it:

| Error type                            | `NTSTATUS`                      | Feature         |
| ------------------------------------- | ------------------------------- | --------------- |
| `core::num::TryFromIntError`          | `STATUS_INTEGER_OVERFLOW`       | always          |
| `core::net::AddrParseError`           | `STATUS_INVALID_ADDRESS`        | always          |
| `alloc::collections::TryReserveError` | `STATUS_INSUFFICIENT_RESOURCES` | `alloc`         |
| `core::alloc::AllocError`             | `STATUS_INSUFFICIENT_RESOURCES` | `allocator-api` |

More types will be added in the future.

### Formatting

`Error` and `Status` both implement `Display` and `Debug`, rendering the status
as `0x` plus eight uppercase hexadecimal digits.

```rust
use ntresult::Error;
use windows_sys::Win32::Foundation::STATUS_ACCESS_DENIED;

let err = Error::from_ntstatus(STATUS_ACCESS_DENIED);

assert_eq!(err.to_string(), "0xC0000022");            // Display
assert_eq!(format!("{err:?}"), "Error(0xC0000022)");  // Debug
```

`Debug` matters most in `unwrap()` panics and `assert_eq!` output, where a
derived implementation would print `Error(-1073741790)` instead.

---

## Macros

Shorthands for the conversions above. Each has a `ret` variant that returns
immediately, which is what makes them worth having in driver code full of early
exits.

| Macro         | Expands to                                                   |
| ------------- | ------------------------------------------------------------ |
| `krok!(v)`    | `Ok(v)`                                                      |
| `krerr!(s)`   | `Err(Error::from_ntstatus(s))`                               |
| `kres!(s)`    | `s.into_result()` — `Ok(())` on `STATUS_SUCCESS`, else `Err` |
| `krokret!(v)` | `return Ok(v)`                                               |
| `krerret!(s)` | `return Err(Error::from_ntstatus(s))`                        |
| `kresret!(s)` | `return s.into_result()`                                     |

```rust
use ntresult::{kres, krerret};
use windows_sys::Win32::Foundation::{
    NTSTATUS, STATUS_INVALID_PARAMETER, STATUS_SUCCESS,
};

// Stand-in for a kernel API returning a raw status.
fn kernel_call() -> NTSTATUS {
    STATUS_SUCCESS
}

fn init(len: usize) -> ntresult::Result<()> {
    if len == 0 {
        // return Err(Error::from_ntstatus(...)) in one step
        krerret!(STATUS_INVALID_PARAMETER);
    }

    // NTSTATUS -> Result, then propagate with `?`
    kres!(kernel_call())?;

    Ok(())
}

assert!(init(0).is_err());
assert!(init(4).is_ok());
```

`krok!` and `krokret!` perform no conversion — they are plain `Ok(..)` and
`return Ok(..)`, provided so the family reads consistently.

---

## Minimum supported Rust version

**1.85**, set by the 2024 edition. The `allocator-api` feature additionally
requires a nightly compiler; everything else builds on stable.

The MSRV is treated as a compatibility promise: raising it is a breaking change
and will come with a minor version bump before 1.0.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
