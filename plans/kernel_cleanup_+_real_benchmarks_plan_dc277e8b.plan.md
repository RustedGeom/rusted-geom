---
name: Kernel Transition + Real Benchmarks Plan
overview: Transition from legacy FFI-oriented naming to kernel-oriented packaging, preserve substantial in-branch additions, enforce real benchmark coverage, and prepare a stable ABI surface for future .NET bindings.
todos:
  - id: crate-rename-transition
    content: Plan and execute transition from legacy kernel crate path to crates/kernel with compatibility window
    status: completed
  - id: lock-current-additions
    content: Stabilize and document already-added LandXML/session/wasm changes before broader refactors
    status: completed
  - id: benchmark-policy-enforcement
    content: Enforce benchmark naming policy and remove placeholder benchmark targets
    status: completed
  - id: real-domain-benchmarks
    content: Implement real Criterion benchmarks for all kernel geometry domains with shared fixtures
    status: completed
  - id: benchmark-ci-pipeline
    content: Integrate benchmark smoke/full workflows, artifacts, and regression thresholds in CI
    status: completed
  - id: naming-packaging-docs
    content: Complete naming/package cleanup and formal API docs with migration mapping and future binding guidance
    status: completed
  - id: dotnet-binding-readiness
    content: Define C ABI contract, memory/error conventions, and packaging strategy for future .NET interop
    status: completed
isProject: false
---

# Kernel Transition + Real Benchmarks Plan

## Updated Scope

- Include and preserve substantial additions already in branch, notably:
- `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/src/landxml](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/src/landxml)`
- `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/src/wasm/landxml.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/src/wasm/landxml.rs)`
- Session integration in `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/src/session/objects.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/src/session/objects.rs)` and `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/src/session/store.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/src/session/store.rs)`
- Transition crate naming from legacy FFI naming to `kernel` with a compatibility window.
- Clean naming and packaging in kernel modules (geometry-first layout, reduced FFI leakage).
- Formal API documentation for Rust and binding layers, including future `.NET` interop constraints.
- Real benchmark suite across all kernel geometry domains, enforced in CI.

## Crate Naming Transition

- Target canonical crate path: `crates/kernel`.
- Keep FFI as internal boundary namespace, not crate identity:
  - `src/api/c_abi`
  - `src/api/wasm`
- Keep C ABI symbol names stable (`rgm_*`) during transition unless a dedicated ABI versioning pass is approved.
- Migration approach:
  - introduce new crate path and workspace wiring
  - update internal imports/dependents/docs
  - remove transitional aliases after adoption.

## Hard Benchmark Policy

- Benchmark names must reflect actual executed workload.
- No placeholder benches that only `black_box` constants or literals.
- If a benchmark cannot be implemented with real workload yet, it must be:
  - excluded from bench targets, or
  - tracked as TODO and excluded from CI benchmark suites.

## Current Baseline to Account For

- `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/bounds_bench.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/bounds_bench.rs)` is a real benchmark target.
- `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/brep_bench.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/brep_bench.rs)` now runs real B-rep workloads (build/validate/tessellate/native roundtrip).
- LandXML parsing/binding additions already exist and should be benchmarked, not re-planned from scratch.

## Execution Phases

### Phase 1 — Stabilize Existing Additions

- Validate the current LandXML parser and wasm/session integration interfaces before package-level refactor.
- Ensure naming and exported API behavior of existing additions is captured in migration notes.
- Confirm current crate compiles and tests pass with current additions before rename.

Status: completed (warning-free `cargo build -p kernel`, warning-free `cargo test -p kernel`, and warning-free benchmark compilation via `cargo bench -p kernel --no-run`).

### Phase 2 — Crate Rename and Packaging Transition

- Move canonical workspace member from legacy crate path to `crates/kernel`.
- Update package metadata and public naming to remove `ffi` from developer-facing identities.
- Preserve existing functionality and ABI behavior while transitioning module/package names.
- Re-establish clear boundaries:
  - geometry/core logic
  - runtime/session
  - `api/c_abi`
  - `api/wasm`.

### Phase 3 — Benchmark Audit and Truthful Naming

- Audit all files in `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches)`.
- For each benchmark function, enforce one of:
  - real workload and truthful name, or
  - remove from active benchmark target.
- Replace placeholder `brep` benchmarks with real kernel calls and fixture-based operations.

### Phase 4 — Build Full Real Benchmark Matrix

- Implement/expand real Criterion benchmarks for all geometry domains:
  - curve, surface, mesh, face, intersection, brep, bounds, landxml.
- Add shared utilities in benchmark common module for:
  - session lifecycle
  - deterministic fixture generation/loading
  - tolerance and options setup
  - repeatable seed/control of stochastic paths
- Standardize outputs and dataset sizes to reduce CI variance.

### Phase 5 — CI Integration

- Add PR smoke benchmark workflow (reduced set, deterministic, fast).
- Add scheduled/manual full benchmark workflow (complete matrix with artifacts).
- Persist Criterion outputs for trend and regression tracking.
- Add explicit regression thresholds for critical operations.

### Phase 6 — Naming, Packaging, and API Documentation

- Reorganize modules toward geometry-oriented layout under the kernel crate.
- Reduce FFI leakage from crate root and keep curated public exports only.
- Produce formal docs:
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/README.md](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/README.md)`
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/docs/reference/kernel-c-abi.md](/Users/cesarecaoduro/GitHub/rusted-geom/docs/reference/kernel-c-abi.md)`
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/docs/reference/kernel-wasm-api.md](/Users/cesarecaoduro/GitHub/rusted-geom/docs/reference/kernel-wasm-api.md)`
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/docs/reference/landxml-support-matrix.md](/Users/cesarecaoduro/GitHub/rusted-geom/docs/reference/landxml-support-matrix.md)`

### Phase 7 — Future .NET Binding Readiness

- Define a stable C ABI interop contract for future `.NET` consumption:
  - opaque handles
  - ownership/freeing rules
  - error code + message retrieval pattern
  - two-pass copy/buffer conventions
  - UTF-8 string conventions.
- Add ABI compatibility/versioning policy and deprecation rules in docs.
- Document intended `.NET` packaging split (native artifacts vs managed wrapper) and test strategy.

Status: completed (added `docs/architecture/dotnet-binding-readiness.md` and extended `docs/architecture/abi-stability.md` with managed-binding constraints).

## Target Benchmark File Set

- During transition (current): `crates/kernel/benches/*`
- Final target (post-rename):
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/common/mod.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/common/mod.rs)`
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/bounds_bench.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/bounds_bench.rs)`
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/curve_bench.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/curve_bench.rs)`
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/surface_bench.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/surface_bench.rs)`
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/mesh_bench.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/mesh_bench.rs)`
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/face_bench.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/face_bench.rs)`
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/intersection_bench.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/intersection_bench.rs)`
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/brep_bench.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/brep_bench.rs)`
  - `[/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/landxml_bench.rs](/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel/benches/landxml_bench.rs)`

## /compound Capture Requirement

- After benchmark hardening and naming cleanup are completed and verified, add a single `docs/solutions/` entry documenting:
  - symptom (patchy naming + placeholder benchmarks)
  - root cause
  - applied fixes
  - prevention policy (truthful benchmark naming + CI enforcement + binding-boundary rules)

