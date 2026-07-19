# Controlled Generation Study

This directory is the preregistered protocol and analysis surface for a
controlled empirical study of software generation with and without Vinglish
Zero. It is intentionally separate from `vz benchmark`, which measures this
repository's runtime pipeline.

No confirmatory samples are checked in. The prior five-pair comparison is
exploratory evidence only because it did not retain the generation provenance
required by this protocol. Do not use its outcomes as data in this study.

## Cohorts

| Cohort | Benchmarks | Levels | Arms | Replications | Required implementations |
| --- | ---: | ---: | ---: | ---: | ---: |
| Core replication | 5 | 1 (`L3`, the previously evaluated scope) | 2 | 10 | 100 |
| Complexity scaling | 5 | 5 (`L1` through `L5`) | 2 | 10 | 500 |

The protocol treats these as distinct cohorts because the requested 100 total
and the requested five-level design cannot both describe one full factorial
experiment.

## Before Generation

1. Create a frozen generation bundle with the
   [generation harness](generation/README.md). It copies all supplied inputs,
   validates the real provider/evaluator commands, captures environment
   provenance, and can emit the ready manifest without manual edits.
2. Review the generated ready manifest and keep the bundle immutable for the
   study run.
3. Execute the manifest with the reusable runner; it populates observations,
   `runs.csv`, and `failures.csv` from actual executions only.
4. Run the analyzer with `--require-complete`; it rejects missing cells,
   duplicate cells, missing provenance, and incomplete replication counts.

## Automated Execution

The reusable execution engine runs the complete frozen study, including
generation, compilation, all evaluation phases, collection, checkpointing,
analysis, and report rendering:

```bash
python3 research/controlled-study/run_study.py \
  --manifest research/controlled-study/data/study-manifest.json \
  --workspace /absolute/path/to/frozen-harness \
  --output research/controlled-study/results/execution
```

It refuses the checked-in blocked manifest. This is intentional: the missing
specifications, prompts, model settings, and evaluator definition must be
frozen before any result is scientifically valid. Read the
[execution manifest contract](specs/execution-manifest.md) before changing the
manifest to `ready`.

## Recovery Guarantees

Each run has an OS-managed, per-run lock and all export updates use a separate
study-wide lock. Locks are advisory file locks released automatically by the
operating system after a process crash; the lock files are retained as forensic
markers but cannot block recovery. A resumed runner reuses completed command
checkpoints and never executes a committed run again.

Observations are the canonical record and each has an adjacent SHA-256 sidecar.
`runs.csv` and `failures.csv` are validated byte-for-byte against the verified
observations at startup. A mismatch blocks execution and preserves the existing
CSV. Use `--repair-exports` only after review; it creates a hash-named backup
before rebuilding the derived exports.

The first execution also binds its output directory to the SHA-256 of the ready
manifest. A resumed study cannot silently switch prompts, commands, schedules,
or generation settings by using edited manifest bytes.

Every recoverable execution failure is committed with a terminal state:
`SUCCESS`, `FAILED`, `SKIPPED`, `ABORTED`, or `INTERRUPTED`. Metrics that cannot
be computed are explicitly recorded as `NOT_AVAILABLE`; blank metric fields are
not permitted.

## Analysis

```bash
python3 analysis/analyze.py \
  --manifest data/study-manifest.json \
  --runs data/runs.csv \
  --failures data/failures.csv \
  --output results \
  --require-complete
```

The dependency-free analyzer emits `summary.json`, `comparisons.json`, and
`report.md`. It uses deterministic bootstrap confidence intervals,
randomization tests, Cliff's delta, coefficient of variation, and Tukey
outlier counts. These statistics are reported only for populated cells and do
not transform heterogeneous benchmark outcomes into a single quality score.

Read [the protocol](protocol.md) before collecting data.

Run the analysis regression tests with:

```bash
python3 -m unittest analysis/test_analyze.py -v
```
