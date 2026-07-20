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

Unchanged blobs are byte-compared and never rewritten. Changed blobs and their
delta records are written through a flushed sibling temporary file and renamed
only after the full content exists, so an interrupted write cannot expose a
truncated cache artifact. On Windows, where replacement rename semantics differ,
the fallback can briefly produce a cache miss but never publishes partial data.
Cache invalidation is limited to a changed semantic hash or a cache, Semantic
IR, reasoning engine, or hypothesis-registry version change.

Malformed cache blobs are disposable. Repository queries preserve valid entries,
report invalid blobs in `skipped`, and reanalyze the corresponding source as
needed. `vz stats` (or `vz cache stats`) reads only the persistent cache and
reports compatible, incompatible, and invalid blob counts without invoking a
frontend.

## Query Integration

`vz query` reads valid cache entries before selecting a frontend. This lets warm
repository queries operate directly on persisted semantic reports. Extension
points without an operational frontend are excluded from source discovery until
they can emit Semantic IR.

`vz profile` uses a fresh isolated cache for its cold run and reuses that exact
cache for the warm run. This makes the reported cold/warm comparison independent
of existing local cache state and leaves the persistent query cache untouched.
