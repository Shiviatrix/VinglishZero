# Verification Reports

This directory is the local output location for `vz verify`, `vz validate`,
`vz benchmark`, `vz profile`, and semantic query experiments.

Reports are intentionally ignored because timings, cache state, frontend
availability, and host hardware make them non-reproducible repository artifacts.
Regenerate them locally before release or performance review.

The exceptions are the files in `expected/`: `semantic-corpus.json` is the
golden contract for deterministic primary intent and minimum confidence, and
`performance-budget.json` is the broad release budget that detects material
regressions in CI.
