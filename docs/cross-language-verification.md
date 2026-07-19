# Cross-Language Semantic Verification

The verification suite is a regression boundary for the adapter architecture.
It proves that independently produced `SemanticGraph` values activate the
expected deterministic hypothesis for a named function, without comparing
source text, ASTs, node IDs, or language-specific frontend structures.

Run the suite from the repository root:

```bash
cargo run -p vz-cli -- verify
cargo run -p vz-cli -- validate
```

It writes:

```text
verification/cross-language-report.json
verification/cross-language-report.md
verification/validation-report.md
```

## Method

For each corpus pattern, the runner selects the adapter from the source file,
constructs a `SemanticGraph`, runs the existing `ReasoningEngine`, and records:

- graph node and function counts
- the required active intent and its confidence
- the actual deterministic primary intent and semantic pipeline

The verifier fails when the expected hypothesis is not active for the target
function in any executable frontend. `vz validate` additionally compares the
actual primary intent and a minimum confidence against the checked-in golden
contract in `verification/expected/semantic-corpus.json`.

## Current Corpus

The checked-in core corpus contains 31 contracts across Python, Java, C, and
the compiler-owned Vinglish semantic-export fixture. It includes accumulator,
counter, maximum, minimum, average, product, reducer, mapper, filter,
validator, linear search, binary search, and the existing collection-pattern
registry: sum, count-if, any, all, find-first, find-last, reverse traversal,
prefix sum, frequency counter, histogram, max-by, min-by, group-by, partition,
unique, distinct, zip, flatten, and contains.

The composition corpus additionally verifies the ordered
`Filter -> Map -> Reduce` pipeline across all four frontends. Its report lists
the ordered pipeline separately from the expected active intent.

`vz validate` also exercises the two existing semantic diagnostic contracts,
six representative query expressions, the cold/warm/invalidation/delta cache
contract, and a deterministic 100/500/1000-function stress corpus. It invokes
the existing benchmark and profile commands and writes one local summary.
It applies the checked-in, deliberately broad release budget in
`verification/expected/performance-budget.json`; exceeding that budget fails
CI and flags a material regression without treating host-specific timings as a
portable microbenchmark.

Vinglish is represented by its stable JSON transport fixture because a Vinglish
compiler executable is not required for the verification suite. This keeps the
test deterministic and preserves the compiler/Zero transport boundary.

Rust, JavaScript, and TypeScript are registered adapter extension points, not
operational source frontends in this build. Validation asserts their stable,
user-facing unavailable-frontend errors. They are intentionally not counted as
cross-language SemanticGraph producers.

## Adding A Pattern

1. Add equivalent source files under `examples/python`, `examples/java`,
   `examples/c`, and `examples/vinglish` (or a compiler-owned Vinglish export
   fixture when source compilation is intentionally outside the suite).
2. Add the four paths to `CORPUS` in
   `crates/cli/src/verification.rs`.
3. Add a golden primary intent and confidence threshold to
   `verification/expected/semantic-corpus.json`.
4. Run `cargo run -p vz-cli -- validate` and inspect the generated report.
5. Specify the target function and expected hypothesis. The test fails if that
   hypothesis is not active for any executable frontend.

## Adding A Language

Implement the existing `SourceAdapter` contract, register it in the CLI
composition module, then add its equivalent source path to every corpus entry.
No semantic engine, reasoning, diagnostic, or Semantic IR change is required.
