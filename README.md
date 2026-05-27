# kerror

Lightweight `NTSTATUS`-based error handling for Windows kernel-mode Rust code.

`kerror` provides a minimal and idiomatic interface for working with Windows `NTSTATUS` values in Rust, designed specifically for `#![no_std]` and kernel-mode environments. It bridges native Windows status codes with Rust's `Result` type without introducing unnecessary abstraction or overhead.

---

## Features

-  `#![no_std]` support (optional `alloc`)
-  Zero-cost abstraction over `NTSTATUS`
-  Transparent `Error` wrapper
-  Idiomatic `Result`-based API
-  Explicit handling of expected status values
-  No memory allocations

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

| kerror::Result<T>    | NTSTATUS         |
| -------------------- | ---------------- |
| Ok(T)                | STATUS_SUCCESS   |
| Err(Error(status))   | status           |

### Returning an arbitrary data type
```rust
pub fn byte_vec(len: usize) -> kerror::Result<Vec<u8>> {
    Ok(Vec::try_with_capacity(len)?)
}
```

### Returning expected non-success statuses

Some kernel APIs use non-success `NTSTATUS` values as valid outcomes (e.g. `STATUS_BUFFER_TOO_SMALL`).

```rust
use kerror::NtStatusResult;

fn driver_fn() -> kerror::Result<NTSTATUS> {
    Ok(STATUS_BUFFER_TOO_SMALL)
}

let status = driver_fn().ntstatus_res();
```
| kerror::Result<NTSTATUS>    | NTSTATUS         |
| --------------------------- | ---------------- |
| Ok(NTSTATUS)                | status           |
| Err(Error(NTSTATUS))        | status           |

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
The crate provides conversions from common Rust errors into kerror::Error.
Currently supported:

- core::net::AddrParseError
- alloc::alloc::AllocError
- alloc::collections::TryReserveError
- core::num::TryFromIntError

Each error is mapped to an appropriate NTSTATUS value.
This allows seamless propagation of Rust-level failures into kernel-compatible error codes.
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