# Semantic Composition

Semantic composition is an additive deterministic layer over the existing
reasoning pipeline:

```text
Semantic IR -> facts -> evidence -> hypotheses -> constraints -> pipeline
```

It does not parse source, introduce a language branch, or rescore individual
hypotheses. A composition is emitted only after every named stage has become an
active hypothesis through its own constraint rules and the function name
contains the composition's stable stage tokens. These anchors prevent broad
loop hypotheses from creating unrelated pipelines.

## Report Contract

`FunctionIntentReport` retains its existing `hypotheses` collection and adds a
defaulted `primary_intent` field plus a `semantic_pipeline` array. The primary
intent uses the legacy active-hypothesis ordering, while each pipeline stage references an existing
hypothesis identifier and its already-computed integer confidence. Older report
consumers that ignore unknown fields retain their behavior; deserializers can
accept older reports because the new field defaults to an empty array.

The composition registry chooses one pipeline: the longest complete match,
then the lexicographically earliest identifier. This produces a stable ordered
pipeline and avoids emitting redundant subsets such as both `Filter -> Map`
and `Filter -> Map -> Reduce` for the same function.

## Registered Pipelines

- `Filter -> Map`
- `Map -> Reduce`
- `Filter -> Reduce`
- `Filter -> Map -> Reduce`
- `Histogram -> Maximum`
- `Group By -> Count`
- `Prefix Sum -> Search`
- `Partition -> Sort`
- `Flatten -> Filter`
- `Unique -> Sort`
- `Binary Search -> Validation`
- `Count If -> Average`
- `Sum -> Normalize`
- `Minimum -> Index`
- `Maximum -> Index`

`sort`, `normalize`, and `index` are deterministic stage hypotheses added for
these compositions. They use only existing evidence: nested loops, division,
numeric return types, loop/conditional control flow, return, and name tokens.
No Semantic IR change was required.

## Determinism

All stage membership, ordering, and confidence values derive from deterministic
constraint evaluation. There are no model weights, probabilities, embeddings,
or external calls.
