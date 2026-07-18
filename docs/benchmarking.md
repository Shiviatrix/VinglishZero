# Benchmarking

Vinglish Zero includes a small deterministic benchmark harness for the current
pipeline.

Run it from the repository root:

```bash
cargo run -p vz-cli -- benchmark
```

The command reuses the existing source-adapter registry and the deterministic
reasoning engine. It measures wall-clock time at the architecture boundary:

- time to lower source into `SemanticGraph`
- time to run the reasoning engine on that graph
- time to parse and match a deterministic semantic query over that report
- total wall-clock time for the sampled pipeline run

The benchmark does not instrument internal parser phases separately. That keeps
the harness aligned with the current architecture, where the CLI sees the
adapter as a single frontend boundary.

Reports are written to:

```text
verification/benchmark-report.json
verification/benchmark-report.md
```

The JSON report is intended for machine comparison. The Markdown report is
intended for a quick human scan during release work.

The default benchmark repeats each sample three times and records the average.
Set `VZ_BENCHMARK_ITERATIONS` to change the measured iteration count:

```bash
VZ_BENCHMARK_ITERATIONS=5 cargo run -p vz-cli -- benchmark
```

The checked-in corpus includes the baseline accumulator and maximum examples,
plus every collection-pattern verification target across Python, Java, C, and
the Vinglish JSON transport fixture. Each benchmark entry selects the named
function for its pattern instead of relying on report order.

The corpus also includes the cross-language `Filter -> Map -> Reduce`
composition case, so the reported reasoning timing includes deterministic
pipeline selection.

For cache benchmarks, run the same `vz query` twice. The first run builds
semantic blobs; the second reports cache hits and avoids frontend work. The
structured query report includes cold/warm latency, analyzed-file count, cache
hits, and cache misses.
