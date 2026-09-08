# kerror

Lightweight `NTSTATUS`-based error handling for Windows kernel-mode Rust code.

`kerror` provides a minimal and idiomatic interface for working with Windows `NTSTATUS` values in Rust, designed specifically for `#![no_std]` and kernel-mode environments. It bridges native Windows status codes with Rust's `Result` type without introducing unnecessary abstraction or overhead.

---

## Highlights

-  `#![no_std]` support (optional `alloc`)
-  Builds on **stable** Rust (nightly only for the opt-in `allocator-api` feature)
-  Zero-cost abstraction over `NTSTATUS`
-  Transparent `Error` wrapper
-  Idiomatic `Result`-based API
-  Explicit handling of expected status values
-  No memory allocations

---

## Feature flags

All features are additive: each one only adds a `From` conversion into `Error`.
Enabling a feature never changes the behaviour of an existing conversion.

| Feature | Default | Toolchain | Conversion added |
| --------------- | ------- | ------------ | -------------------------------------- |
| `alloc`         | no      | stable       | `alloc::collections::TryReserveError`  |
| `allocator-api` | no      | **nightly**  | `core::alloc::AllocError`              |

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
kerror = "0.3"

# stable, plus the TryReserveError conversion
# (needs a #[global_allocator] in the final artifact)
kerror = { version = "0.3", features = ["alloc"] }

# nightly, everything
kerror = { version = "0.3", features = ["alloc", "allocator-api"] }
```

---

## Core Types

### `Result<T>`

```rust
pub type Result<T, E = Error> = core::result::Result<T, E>;
```

### `Error`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Error(pub(crate) NTSTATUS);

impl core::error::Error for Error {}
```

## Usage

### Converting NTSTATUS → Result

```rust
use kerror::IntoResult;

fn some_kernel_call() -> NTSTATUS {
    // Returns any NTSTATUS
}

fn my_func() -> kerror::Result<()> {
    some_kernel_call().into_result()
}
```

| NTSTATUS       | kerror::Result<()>   |
| -------------- | -------------------- |
| STATUS_SUCCESS | Ok(())               |
| any other code | Err(Error(NTSTATUS)) |

`kerror` intentionally treats all non-`STATUS_SUCCESS` values as errors.
This preserves strict semantics and avoids ambiguity.


### Returning NTSTATUS from Result
```rust
use kerror::NtStatus;

fn driver_fn() -> kerror::Result<()> {
    Ok(())
}

let status = driver_fn().ntstatus();
```

| kerror::Result<()>   | NTSTATUS         |
| -------------------- | ---------------- |
| Ok(())               | STATUS_SUCCESS   |
| Err(Error(status))   | status           |

### Returning an arbitrary data type

When the `Ok` payload is data rather than a status, use `ntstatus_or_success()`.
The name states that the payload is discarded.

```rust
use kerror::NtStatusOrSuccess;

pub fn byte_vec(len: usize) -> kerror::Result<Vec<u8>> {
    let mut buf = Vec::new();
    buf.try_reserve(len)?;
    Ok(buf)
}

let status = byte_vec(16).ntstatus_or_success();
```

| kerror::Result<T>    | NTSTATUS         |
| -------------------- | ---------------- |
| Ok(T)                | STATUS_SUCCESS   |
| Err(Error(status))   | status           |

### Returning expected non-success statuses

Some kernel APIs use non-success `NTSTATUS` values as valid outcomes (e.g. `STATUS_BUFFER_TOO_SMALL`).

Return them as `Ok` in a `StatusResult`, and they survive extraction:

```rust
use kerror::{NtStatus, Status, StatusResult};

fn driver_fn() -> StatusResult {
    Ok(Status::new(STATUS_BUFFER_TOO_SMALL))
}

let status = driver_fn().ntstatus();   // STATUS_BUFFER_TOO_SMALL, not SUCCESS
```

| StatusResult (= Result<Status>) | NTSTATUS         |
| ------------------------------- | ---------------- |
| Ok(Status(status))              | status           |
| Err(Error(status))              | status           |

`Status` is a newtype rather than a bare `NTSTATUS` because `NTSTATUS` is an
alias for `i32`: without it, a `Result<i32>` carrying a byte count would be
indistinguishable from one carrying a status code.

---

## Error Conversion
In case you want to directly convert an NTSTATUS value into `Error` you can use the `kerror::IntoError` trait.

```rust
use kerror::IntoError;

pub fn check_len(len: usize) -> kerror::Result<()> {
    if len == 0 {
        return Err(STATUS_INVALID_PARAMETER.into_error());
    }

    Ok(())
}
```

### Common error types
The crate provides conversions from common Rust errors into `kerror::Error`, so a
Rust-level failure can propagate through `?` and be returned to the kernel as a
status code. Each error maps to the `NTSTATUS` that best describes it:

| Error type                            | `NTSTATUS`                      | Feature         |
| ------------------------------------- | ------------------------------- | --------------- |
| `core::num::TryFromIntError`          | `STATUS_INTEGER_OVERFLOW`       | always          |
| `core::net::AddrParseError`           | `STATUS_INVALID_ADDRESS`        | always          |
| `alloc::collections::TryReserveError` | `STATUS_INSUFFICIENT_RESOURCES` | `alloc`         |
| `core::alloc::AllocError`             | `STATUS_INSUFFICIENT_RESOURCES` | `allocator-api` |

More types will be added in the future.

### Formatting
`kerror::Error` implements the `Display` trait:
```rust
println!("{}", err);
```
Output:
```
0xC0000005
```