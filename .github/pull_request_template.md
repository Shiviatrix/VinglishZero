## Summary

Describe the problem and the change.

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --offline -- -D warnings`
- [ ] `cargo test --workspace --offline`
- [ ] `vz verify`
- [ ] `vz validate`
- [ ] Documentation and benchmark impact reviewed where applicable

## Architecture

- [ ] No compiler/HIR coupling was introduced.
- [ ] Deterministic behavior and transport compatibility are preserved.
