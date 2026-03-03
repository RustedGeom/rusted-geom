# .NET Binding Readiness (C ABI Contract)

This document defines the contract for a future `.NET` binding layer on top of
the Rust kernel C ABI.

## Scope

- Native layer: exported C ABI functions (`rgm_*`) from the kernel crate.
- Managed layer: C# P/Invoke wrapper and safe ownership abstractions.
- Out of scope: high-level C# API shape and domain naming ergonomics.

## Native/Managed Boundary Rules

- Bind only C ABI symbols (`rgm_*`), never Rust-mangled symbols.
- Bind only stable layout types (`#[repr(C)]`, `#[repr(transparent)]`, `#[repr(i32)]`).
- Treat handles as opaque `u64` values (`0` is invalid).
- Keep session/object lifecycle pairing explicit:
  - `rgm_kernel_create` / `rgm_kernel_destroy`
  - `rgm_object_release`
- Prefer caller-owned buffers with pointer+capacity+out_count patterns.

## Error and Diagnostics Model

- Every call checks `RgmStatus`.
- On non-`Ok`, fetch diagnostics from:
  - `rgm_last_error_code`
  - `rgm_last_error_message` (two-pass buffer sizing)
- Managed wrapper maps status + message into typed exceptions.

## Marshaling Conventions

- Scalars: `f64 -> double`, `u32 -> uint`, `usize -> nuint`.
- Booleans: marshal explicitly and consistently (1-byte bool semantics).
- Arrays: pass pinned spans/arrays as pointer + count.
- Strings: UTF-8 buffers with explicit size and written-count handling.
- Use query-then-copy for variable-length outputs.

## ABI Compatibility Rules for Managed Bindings

Treat these as breaking for managed consumers:

- enum discriminant changes
- `#[repr(C)]` field order/type changes
- handle/value width changes
- ownership or allocation/free pairing changes

Additive API growth is allowed when existing symbols remain unchanged.

## Initial .NET Validation Matrix

- Session create/destroy roundtrip
- Object create/release for each exposed domain
- Error path retrieval (`last_error_code`, `last_error_message`)
- Variable-length buffer copy paths
- Multi-session isolation behavior
