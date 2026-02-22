# Binding + Codegen TODOs

## Completed
- [x] Create `kernel-abi-meta` proc-macro crate with `#[rgm_export]` and `#[rgm_ffi_type]`.
- [x] Implement session-scoped C ABI in `kernel-ffi`.
- [x] Add `rgm_nurbs_interpolate_fit_points` constructor with exact fit-point policy defaults.
- [x] Add error APIs `rgm_last_error_code` and `rgm_last_error_message`.
- [x] Add M1 evaluation/intersection ABI stubs.
- [x] Build metadata-driven `abi-gen` CLI and emit `target/abi/rgm_abi.json`.
- [x] Generate `include/rusted_geom.h`.
- [x] Generate `.NET` bindings (`LibraryImport`, model structs, safe handles, wrapper API).
- [x] Generate web TypeScript facade and function catalog.
- [x] Add stale-generation CI checks.
- [x] Add ABI compatibility gate with semver-major enforcement and baseline manifest.
- [x] Add runtime safety tests for handle/session behavior.
- [x] Add generator tests for naming and hash behavior.
- [x] Apply CAD-kernel naming refactor across C ABI + generated .NET/TS APIs (hard break).
- [x] Add generated CurveHandle instance APIs with direct-return semantics for .NET/TS.

## Next
- [ ] Replace curve placeholder evaluation with full NURBS evaluator integration.
- [ ] Expand M1 constructor/evaluation/intersection parity integration tests for real .NET and WASM execution against native runtime.
- [ ] Add NuGet/npm packaging and publish workflow with RID-specific native assets.
- [ ] Add ABI/API diff reporting artifact in CI for easier review.
