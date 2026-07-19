# Contributing

Thank you for contributing to Vinglish Zero.

## Principles

- Keep all reasoning deterministic and reproducible.
- Keep adapters language-specific and everything after Semantic IR language-agnostic.
- Do not couple Vinglish Zero to compiler crates or HIR.
- Preserve the versioned Vinglish JSON transport boundary.
- Prefer focused changes with regression coverage and documentation updates.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --workspace --offline -- -D warnings
cargo test --workspace --offline
cargo run -p vz-cli --offline -- verify
cargo run -p vz-cli --offline -- validate
cargo run -p vz-cli --offline -- benchmark
```

Run `vz profile` when a change affects execution, caching, or query behavior.
Do not commit `.vinglish-zero/`, `target/`, or locally generated cache blobs.

## Adapter Contributions

Implement the `SourceAdapter` contract, lower only to shared Semantic IR, and
add representative fixtures plus cross-language verification. Do not add
language checks to reasoning or diagnostics.

## Pull Requests

Explain the problem, design constraints, validation performed, and any impact
on transport compatibility, determinism, or benchmarks.
