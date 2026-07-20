# Semantic Query Engine

`vz query "<expression>"` searches source files handled by operational adapters
and Vinglish JSON transport fixtures in the current repository. Each file
follows the existing adapter -> Semantic IR -> reasoning path; querying never
parses a language or inspects source syntax itself.

## Query Language

- `AND` requires both terms. Adjacent terms are an implicit `AND`.
- `OR` accepts either term.
- `NOT` excludes a term.
- `THEN` requires an ordered semantic-pipeline relationship.

Examples:

```text
vz query "reducers"
vz query "maximum finder"
vz query "binary search"
vz query "filter THEN reduce"
vz query "NOT recursive"
```

Terms match deterministic report fields only: primary intent, ordered pipeline
stages, active hypotheses at or above the fixed 50% confidence floor, evidence
kinds, and normalized function-name tokens. Primary intents and pipeline stages
remain queryable regardless of confidence because the engine selected them as
the deterministic result for that function.
Aliases such as `reducers`, `maximum finder`, `max by`, and `binary search`
normalize to the corresponding stable hypothesis identifiers. There is no fuzzy
matching, embedding lookup, model inference, or external API.

## Ranking

Ranking uses fixed integer points: primary intent 100, pipeline stage 80,
active hypothesis 60, evidence 40, and function metadata 20. `THEN` receives
a fixed ordered-pipeline score. Ties sort by source path and function name.
Every query result includes matched fields and latency in nanoseconds.

Unavailable frontend extension points are ignored until they can emit Semantic
IR. JSON is considered only when it declares the Vinglish semantic transport
format, so unrelated repository configuration never becomes adapter noise.
Unreadable eligible files are reported in `skipped` rather than causing a query
failure.
