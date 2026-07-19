# Incremental Semantic Execution

The incremental layer is additive to the deterministic pipeline. A frontend may
submit each completed `FunctionFacts` value to `ReasoningEngine::analyze_function`
immediately. `analyze_functions_parallel` uses a bounded worker pool and restores
input ordering before report construction, so concurrency cannot affect output.

```text
Function Semantic IR -> facts -> reasoning -> IntentReport -> semantic cache
```

## Persistent Cache

Cache entries live under `.vinglish-zero/cache/`, one binary blob per stable
`source::ordinal::function-name` identifier. An entry contains:

- source fingerprint and stable semantic hash
- Semantic IR, reasoning engine, hypothesis registry, and cache versions
- immutable function facts
- `FunctionIntentReport`, including primary intent and semantic pipeline
- diagnostics and query metadata

An unchanged source fingerprint loads the cache directly. When a source file
changes, it is lowered normally, but only functions with a changed semantic hash
are reanalyzed. Functions whose semantic state is unchanged reuse their stored
report and update only their source fingerprint.

## Binary Format And Deltas

Each blob has a deterministic `VZSC` magic header, little-endian cache version,
payload length, and canonical serialized semantic payload. The cache stores a
binary prefix/suffix delta beside a changed blob. A delta records prefix length,
removed length, suffix length, and replacement bytes. Applying it reconstructs
the next semantic blob exactly; it never operates on source or AST data.

Unchanged blobs are byte-compared and never rewritten. Cache invalidation is
limited to a changed semantic hash or a cache, Semantic IR, reasoning engine,
or hypothesis-registry version change.

## Query Integration

`vz query` reads valid cache entries before selecting a frontend. This lets warm
repository queries operate directly on persisted semantic reports. Unavailable
frontends do not block a query when their cache entry remains valid.
