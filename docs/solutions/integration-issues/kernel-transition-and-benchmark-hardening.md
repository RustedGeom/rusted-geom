---
title: "Kernel Path Transition and Benchmark Hardening"
category: "integration-issues"
date: "2026-03-03"
status: "completed"
tags:
  - rust
  - wasm
  - benchmarks
  - ci
  - bindings
---

## Problem

The kernel crate identity and structure were patchy:

- crate path still used `crates/kernel` migration-incompatible legacy naming despite broader kernel scope
- benchmark naming included placeholder workloads that were not real benchmarks
- CI had no benchmark smoke/full strategy
- formal ABI/WASM reference docs were incomplete for future managed bindings

## Root Cause

- historical FFI-first naming remained as the crate identity
- benchmark stubs were added as placeholders and never promoted to real workloads
- documentation and CI policy evolved slower than the implementation surface

## Working Solution

1. Transition crate path from legacy naming to `crates/kernel`, then update package identity to `kernel`.
2. Replace placeholder `brep_bench` with real operations:
   - B-rep build from surfaces
   - validation
   - tessellation
   - native save/load roundtrip
3. Add real domain benchmarks:
   - `curve_bench`, `surface_bench`, `mesh_bench`, `face_bench`, `intersection_bench`, `landxml_bench`
   - shared deterministic fixtures in `benches/common/mod.rs`
4. Add CI benchmark workflows:
   - smoke benchmark job in `ci.yml`
   - scheduled/manual full benchmark workflow in `benchmarks-full.yml`
5. Add formal docs for ABI/WASM and `.NET` readiness:
   - `docs/reference/kernel-c-abi.md`
   - `docs/reference/kernel-wasm-api.md`
   - `docs/reference/landxml-support-matrix.md`
   - `docs/architecture/dotnet-binding-readiness.md`

## Validation

- `cargo build -p kernel` passes without warnings
- `cargo test -p kernel` passes
- `cargo bench -p kernel --no-run` passes with all benchmark targets
- benchmark smoke commands run successfully for bounds and curve benches
- wasm staging scripts work with the new crate path

## Prevention

- Keep benchmark policy strict: benchmark names must match real executed workloads.
- Do not merge placeholder benchmark stubs into active benchmark targets.
- Keep smoke/full benchmark workflows as part of CI maintenance.
- Preserve ABI contracts explicitly in docs before adding new language bindings.
- When changing crate paths, update scripts/workflows/bindings imports in one pass and re-validate end-to-end.
