# Cross-Language Semantic Verification

The verification suite is a regression boundary for the adapter architecture.
It proves that independently produced `SemanticGraph` values activate the
expected deterministic hypothesis for a named function, without comparing
source text, ASTs, node IDs, or language-specific frontend structures.

Run the suite from the repository root:

```bash
cargo run -p vz-cli -- verify
```

It writes:

```text
verification/cross-language-report.json
verification/cross-language-report.md
```

## Method

For each corpus pattern, the runner selects the adapter from the source file,
constructs a `SemanticGraph`, runs the existing `ReasoningEngine`, and records:

- graph node and function counts
- the required active intent and confidence

The verifier fails when the expected hypothesis is not active for the target
function in any language. It intentionally does not compare arbitrary primary
intent ordering, because a function can have more than one valid active
hypothesis. This prevents syntax and AST shape from becoming a false
equivalence requirement.

## Current Corpus

The checked-in corpus verifies `accumulator` plus the collection-pattern
registry across Python, Java, C, and the compiler-owned Vinglish
semantic-export fixture: `sum`, `minimum`, `count_if`, `any`, `all`,
`find_first`, `find_last`, `reverse_traversal`, `prefix_sum`,
`frequency_counter`, `histogram`, `max_by`, `min_by`, `group_by`, `partition`,
`unique`, `distinct`, `zip`, `flatten`, and `contains`.

The composition corpus additionally verifies the ordered
`Filter -> Map -> Reduce` pipeline across all four frontends. Its report lists
the ordered pipeline separately from the expected active intent.

Vinglish is represented by its stable JSON transport fixture because a Vinglish
compiler executable is not required for the verification suite. This keeps the
test deterministic and preserves the compiler/Zero transport boundary.

## Adding A Pattern

1. Add equivalent source files under `examples/python`, `examples/java`,
   `examples/c`, and `examples/vinglish` (or a compiler-owned Vinglish export
   fixture when source compilation is intentionally outside the suite).
2. Add the four paths to `CORPUS` in
   `crates/cli/src/verification.rs`.
3. Run `cargo run -p vz-cli -- verify` and inspect the generated report.
4. Specify the target function and expected hypothesis. The test fails if that
   hypothesis is not active for any frontend.

## Adding A Language

Implement the existing `SourceAdapter` contract, register it in the CLI
composition module, then add its equivalent source path to every corpus entry.
No semantic engine, reasoning, diagnostic, or Semantic IR change is required.
