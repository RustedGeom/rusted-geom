## Summary

<!-- Briefly describe what this PR does and why. -->

## Changes

<!-- List the key changes in this PR. -->

- 

## Related Issues

<!-- Link related issues: Fixes #123, Closes #456 -->

## Type of Change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that changes existing behavior)
- [ ] Performance improvement
- [ ] Refactoring (no functional changes)
- [ ] Documentation update
- [ ] Build / CI change

## Checklist

- [ ] `cargo test -p kernel` passes
- [ ] `cargo clippy -p kernel` has no new warnings
- [ ] TypeScript type check passes (`npm --prefix ./bindings/web run typecheck`)
- [ ] New public APIs are documented
- [ ] Benchmarks checked for regressions (if touching hot paths)
- [ ] WASM API reference updated (if adding/changing WASM exports)
