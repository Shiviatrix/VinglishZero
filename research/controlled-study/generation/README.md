# Generation Harness

This harness prepares real, immutable inputs for the controlled-study runner.
It never generates source itself. A study owner supplies a real external
generator command, benchmark specification, prompts, arm-specific instructions,
and evaluator commands.

## Prepare And Freeze

Create a definition JSON with the fields exercised by
[`test_generation.py`](test_generation.py). All prompt and specification paths
may be relative to the definition file. Then run:

```bash
python3 research/controlled-study/generation/prepare_bundle.py \
  --definition /absolute/path/study-definition.json \
  --output /absolute/path/frozen-bundle \
  --freeze-output /absolute/path/frozen-bundle/study-ready.json
```

Preparation copies the supplied files, computes their digests, captures the
host and Git environment, validates all provider/evaluator executables, runs
declared preflight commands, creates an immutable prompt bundle, and writes a
draft directly consumable by `freeze_manifest.py`. With `--freeze-output`, it
also produces and validates the ready manifest.

`--freeze-output` must be inside `--output`: the ready manifest references
hash-bound, relative artifacts in that directory. This makes the bundle
portable as one unit and prevents a manifest from silently resolving artifacts
from another location.

The definition supplies `benchmark_spec`, the four prompt files,
`arm_instructions`, generator configuration, and every evaluator argv. The
generator configuration must include `model_id`, `model_snapshot`,
`temperature`, `top_p`, `seed_policy`, `max_tokens`, `tool_configuration`,
`retry_policy.max_attempts`, `timeout_seconds`, `generation_budget`, and a
`provider_command`. Preparation rejects an incomplete definition, missing
files, unavailable executables, failed preflight commands, or non-portable
manifest placement.

## Provider Contract

`generator.provider_command` is a shell-free argv template. It receives
`{request_path}`, `{response_path}`, `{artifact_dir}`, `{session_dir}`,
`{benchmark}`, `{level}`, `{arm}`, and `{session_id}`. The real provider must
write a JSON response at `{response_path}` containing:

```json
{
  "generation_credits": 0,
  "token_usage": 0,
  "tool_usage": []
}
```

The wrapper records every request, response, attempt log, duration, retry,
environment snapshot, and hash under the execution run directory. A provider
failure is returned to the existing execution engine as a non-zero command;
the engine records its terminal `FAILED` observation.

The provider is deliberately not implemented by Vinglish Zero. It is the
study owner's real model/client integration and must record actual generation
credits, token usage, and tool usage. Test fixtures use a local fake provider
only to validate the contract; they are not experimental results.

## Regression Tests

Run the isolated generation-harness tests from the controlled-study directory:

```bash
cd research/controlled-study
python3 -m unittest generation/test_generation.py -v
```
