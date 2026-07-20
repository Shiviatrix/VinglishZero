# Deterministic Reasoning Pipeline

## Collection Patterns

The collection-pattern registry includes `sum`, `minimum`, `count_if`, `any`,
`all`, `find_first`, `find_last`, `reverse_traversal`, `prefix_sum`,
`frequency_counter`, `histogram`, `max_by`, `min_by`, `group_by`, `partition`,
`unique`, `distinct`, `zip`, `flatten`, and `contains`.

Each hypothesis is a fixed set of structural requirements and integer support
weights. For example, `minimum` requires a loop, a numeric return, and a
comparison-controlled assignment that selects a smaller value. `flatten`
requires nested loops. `contains` requires looped conditional control flow and
a return, with boolean return type and loop-local return providing additional
support. Names are only supporting evidence; absence of a conventional name
does not make a structurally complete hypothesis impossible.

The new patterns use the existing Semantic IR nodes. The only additional fact
is whether the declared function return type is boolean, emitted as
`BooleanReturn` evidence. No adapter, source language, or compiler type is
visible to the rule evaluator.

Vinglish Zero reasons only over `SemanticGraph`. It does not receive source
text, adapter types, compiler types, network data, embeddings, probability
models, or machine-learning output.

```text
Semantic IR
  -> fact extraction
  -> evidence collection
  -> hypothesis registry
  -> constraint elimination
  -> integer confidence scoring
  -> semantic composition
  -> IntentReport
```

## Stages

1. `FactExtractor` walks Semantic IR and creates immutable `FactSet` values.
   Facts include control flow, mutation, calls, recursion, ownership, API use,
   boolean returns, and language-neutral type relationships.
2. Independent `EvidenceProvider` implementations translate facts into sorted
   observations. Providers do not name or score hypotheses.
3. `HypothesisRegistry` supplies declarative `RuleBasedHypothesis` values.
   New hypotheses can be registered without changing extraction, providers, or
   the constraint engine.
4. `ConstraintEngine` evaluates `Require`, `Support`, and `Reject` rules.
   Missing required evidence eliminates a hypothesis. Support and rejection use
   fixed integer weights and a bounded evidence-strength multiplier.
5. `IntentReport` is serialized with a fixed version and sorted output.
6. The optional composition layer selects the most specific ordered pipeline
   whose stages are all active hypotheses. See
   [semantic composition](semantic-composition.md).

`vz explain` renders this report directly. For every function it presents the
primary intent and confidence, any recognized pipeline, the observed evidence,
the strongest active alternatives, and a bounded summary of constraint
eliminations. It does not inspect source text after lowering.

## Determinism

The pipeline uses no random input. Semantic graph traversal uses the graph's
ordered node store; evidence is sorted by provider and kind; hypothesis
definitions are held in a sorted registry; reports sort active hypotheses by
confidence and then identifier. A hypothesis score is an integer clamped to
`0..100`.

`crates/reasoning/tests/deterministic_pipeline.rs` runs the same Semantic IR
twice and asserts byte-for-byte equal JSON reports. Its accumulator fixture
also demonstrates rule elimination: `average` is eliminated when looped
addition and a numeric return are present but division evidence is absent.
The same test suite constructs Semantic IR for every collection pattern and
asserts its expected active hypothesis without using an adapter.
