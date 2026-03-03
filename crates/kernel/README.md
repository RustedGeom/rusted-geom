# kernel crate

`crates/kernel` contains the Rust geometry kernel used by both the C ABI and
the `wasm-bindgen` API surface.

## Layers

- `src/math`: low-level geometry and NURBS math routines.
- `src/elements`: geometry/topology data structures.
- `src/session`: session/object storage and lifecycle.
- `src/kernel_impl`: C ABI operation implementations and exported symbols.
- `src/wasm`: `wasm-bindgen` public API (`KernelSession`, handles, typed methods).
- `src/landxml`: LandXML parsing and sampling support.

## Build and Test

```bash
cargo build -p kernel
cargo test -p kernel
cargo bench -p kernel --no-run
```

## Benchmarks

Bench targets live under `benches/` and are real workload benchmarks:

- `bounds_bench`
- `curve_bench`
- `surface_bench`
- `mesh_bench`
- `face_bench`
- `intersection_bench`
- `brep_bench`
- `landxml_bench`

## ABI Boundaries

- C ABI symbols are `rgm_*` and return `RgmStatus`.
- Variable-length outputs use pointer + capacity + out_count patterns.
- Errors are session-scoped and retrievable via `rgm_last_error_*`.
