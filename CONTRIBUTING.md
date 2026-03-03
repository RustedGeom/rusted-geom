# Contributing to rusted-geom

Thank you for your interest in contributing to **rusted-geom**! This document covers everything you need to get started.

## Getting Started

1. **Fork** the repository and clone your fork.
2. Install the [prerequisites](README.md#prerequisites).
3. Run `pnpm install` to install JavaScript dependencies.
4. Build the WASM kernel: `./scripts/build_kernel_wasm.sh`

## Development Workflow

### Rust kernel changes

```bash
cargo test -p kernel           # run unit tests
cargo bench -p kernel           # run benchmarks
./scripts/build_kernel_wasm.sh  # rebuild WASM
```

### TypeScript binding changes

```bash
./scripts/stage_web_wasm.sh
cd bindings/web && npm run build
npm run typecheck
npm run test
```

### Showcase app changes

```bash
pnpm --dir showcase dev         # local dev server at http://localhost:3000
```

## Pull Request Guidelines

- **One concern per PR.** Keep pull requests focused on a single change.
- **Write tests.** New kernel operations should include Rust unit tests. WASM-facing APIs should have corresponding web runtime tests.
- **Run CI locally.** Before pushing, make sure `cargo test -p kernel` and the TypeScript type check pass.
- **Follow existing patterns.** Look at neighboring code for naming conventions, error handling, and module structure.
- **Benchmark-sensitive changes.** If your change affects hot paths (evaluation, tessellation, intersections), run `cargo bench -p kernel` and note any regressions.

## Commit Messages

Use concise, imperative-mood commit messages:

```
Add curve reverse operation
Fix surface tessellation edge case at seam
Refactor session store to use DashMap
```

For multi-line messages, include a blank line after the subject and explain *why* the change was made.

## Code Style

### Rust

- Follow standard `rustfmt` formatting (run `cargo fmt`).
- Use `clippy` to catch common issues: `cargo clippy -p kernel`.
- Prefer returning `Result` types over panicking.
- Document public API items with `///` doc comments.

### TypeScript

- Follow the existing ESM module patterns in `bindings/web/`.
- Types should be explicit — avoid `any`.

## Adding a New Kernel Operation

1. Implement the core logic in `crates/kernel/src/kernel_impl/`.
2. Add the `#[wasm_bindgen]` wrapper in the appropriate `crates/kernel/src/wasm/` module.
3. Write Rust tests in the same file or a dedicated test module.
4. Update the [WASM API Reference](docs/reference/kernel-wasm-api.md) if adding a public API.
5. Add a web runtime test in `bindings/web/`.

## Reporting Issues

- Use the [issue templates](.github/ISSUE_TEMPLATE/) to file bugs, request features, or report performance regressions.
- Include reproduction steps and environment details for bug reports.
- Check existing issues before filing a new one.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
