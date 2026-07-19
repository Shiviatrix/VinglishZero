# Execution Manifest Contract

`run_study.py` executes only a manifest whose `status` is `ready`. It does not
connect to a model service itself and it does not accept an ad-hoc prompt at
runtime. This keeps the experiment runner reusable while making the actual
generator, test harness, and evaluator part of the frozen experimental input.

## Required Inputs

The manifest must retain the schedule already declared by the protocol:

```json
{
  "study_id": "new-immutable-study-id",
  "status": "ready",
  "benchmarks": ["chess_engine", "json_parser", "markdown_parser", "regex_engine", "sql_database_engine"],
  "levels": ["L1", "L2", "L3", "L4", "L5"],
  "arms": ["baseline", "vinglish_zero"],
  "replications_per_cell": 10
}
```

`required_artifacts` must contain the protocol fields and a `files` object.
Each `files` key is a path relative to the manifest and each value is the
`sha256:<hex>` digest of that exact file. The runner verifies these digests
before scheduling its first generation. A changed prompt, specification, lock
file, evaluator, or hardware manifest therefore blocks the cohort rather than
silently mixing conditions.

Create that immutable mapping from an explicit `draft` manifest with `bindings`
instead of typing digests manually:

```bash
python3 research/controlled-study/freeze_manifest.py \
  --draft research/controlled-study/specs/my-study-draft.json \
  --output research/controlled-study/specs/my-study-ready.json
```

The draft must bind each required artifact name to a relative file. The utility
hashes every bound file, writes a new `ready` manifest, and verifies the result.
It never changes an already ready manifest.

`provenance` must declare `model_id`, `model_snapshot`, `temperature`, `top_p`,
`seed_policy`, `tool_budget`, `git_commit`, and `hardware_id`.

## Commands

The required command names are `generate`, `compile`, `maintained_tests`,
`independent_tests`, `randomized_tests`, `stress_tests`, `feature_evaluation`,
`security_evaluation`, and `performance_evaluation`. Each command is an argv
array and a positive timeout:

```json
{
  "generate": {
    "argv": ["/absolute/path/to/frozen-generator", "--output", "{artifact_dir}"],
    "timeout_seconds": 1800
  }
}
```

Commands are never passed through a shell. The permitted placeholders are
`{study_id}`, `{benchmark}`, `{level}`, `{arm}`, `{replicate}`, `{generation_id}`,
`{run_dir}`, and `{artifact_dir}`. The generator must create the artifact in
`{artifact_dir}` and emit a JSON object containing `generation_credits` and
`retry_count`. Test commands emit `{"passed": N, "total": N}`. The feature
evaluator uses the same shape; the security evaluator emits `{"findings": N}`;
and the performance evaluator emits `runtime_seconds`, `memory_bytes`, and
`startup_seconds`.

All stdout, stderr, argv, duration, exit status, artifact hash, metrics, and
automatically classified failures are stored per run. A completed observation
is written atomically before it appears in the CSV exports, so a resumed study
skips terminal runs and continues only missing cells.

`runs.csv` also records the study and benchmark versions, git commit, Rust and
Python versions, token and tool usage, execution and evaluation duration, CPU,
allocation, throughput, and evaluator-reported memory metrics.

## Execution

```bash
python3 research/controlled-study/run_study.py \
  --manifest research/controlled-study/data/study-manifest.json \
  --workspace /absolute/path/to/frozen-harness \
  --output research/controlled-study/results/execution
```

Use `--max-runs N` to deliberately checkpoint after `N` terminal observations.
Reissue the exact command to resume. After the full factorial schedule is
complete, the runner invokes the preregistered analyzer and emits Markdown,
HTML, PDF, JSON, CSV, and SVG publication artifacts under the selected output
directory.

## Locking And Recovery

The runner takes one cross-process lock per `generation_id` before checking or
executing that cell, plus a study-wide export lock while validating or updating
CSV exports. The locks are backed by the operating system and release when a
process crashes; stale lock files therefore do not prevent recovery.

On first use, the output directory records the SHA-256 of the complete ready
manifest. Subsequent executions must present the same bytes, preventing a
resumed cohort from silently changing its frozen commands, schedule, or prompt
configuration.

`observation.json` is canonical and is published with a SHA-256 sidecar. The
runner validates its complete schema, including terminal state and every CSV
field, before publishing it. Derived CSV files are never silently repaired. If
their bytes differ from the verified observations, execution stops and preserves
the mismatched file. After investigation, run:

```bash
python3 research/controlled-study/run_study.py \
  --manifest research/controlled-study/specs/my-study-ready.json \
  --output research/controlled-study/results/execution \
  --repair-exports
```

The command keeps a hash-named forensic copy of each repaired export.
