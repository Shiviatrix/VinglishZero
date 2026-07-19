# Raw Data Contract

`runs.csv` and `failures.csv` are checked-in schemas, not data. Do not replace
blank fields with estimates. Store generated source, evaluator logs, and prompt
bundles under repository-relative artifact paths, then record their SHA-256
values here.

`study-manifest.json` remains blocked until the exact benchmark specifications
and prompt bundle are frozen. The analyzer refuses confirmatory output while
required provenance is incomplete.

The execution engine writes raw rows to its selected output directory rather
than mutating these checked-in blank schemas. See
[`specs/execution-manifest.md`](../specs/execution-manifest.md) for the frozen
command, metrics, and checkpoint contract.
