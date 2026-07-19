# Protocol: Controlled Generation Study

## Research Questions

1. Under fixed generation and evaluation conditions, do the baseline and
   Vinglish Zero arms differ on predeclared outcome measures?
2. Does any observed difference vary by benchmark domain or implementation
   complexity level?
3. Does either arm show materially different variance, failure frequency, or
   outlier behavior?

This protocol does not presume a favorable result for either arm.

## Units And Design

The unit of analysis is one independently generated implementation. Each cell
is identified by `(benchmark, level, arm, replicate)`. Arms are `baseline` and
`vinglish_zero`; replicates are integers 1 through 10.

The five domains are Chess Engine, JSON Parser, Markdown Parser, Regular
Expression Engine, and SQL Database Engine. Every domain has five predeclared
levels:

| Level | Meaning |
| --- | --- |
| L1 | Minimal implementation with only the explicitly listed core operation. |
| L2 | Basic feature-complete implementation for the frozen task specification. |
| L3 | Intermediate implementation matching the previously evaluated task scope. |
| L4 | Advanced implementation with the frozen extension requirements. |
| L5 | Production-style implementation with the frozen non-functional requirements. |

The original five-pair exercise corresponds only to the L3 scope. It cannot be
reused as a replication because its implementations and generation provenance
are already known.

## Required Controls

For every cell, retain identical values for:

- Frozen task specification and evaluation prompt
- System prompt and model snapshot/version
- Temperature, top-p, seed policy, token and tool budgets
- Hardware, operating system, Python version, and dependency lockfile
- Evaluation harness revision, test corpus, randomized seeds, and timeouts

The sole experimental treatment is whether the generator can use Vinglish
Zero. Any treatment-specific instruction must be explicit in the frozen prompt
bundle. If model or evaluator drift occurs, start a new study ID rather than
mixing runs.

## Required Provenance

Each row in `data/runs.csv` must include prompt hashes, model identity,
generation settings, hardware ID, evaluator revision, artifact hash, credits,
wall-clock generation time, and retry count. Values that are unavailable must
be recorded as missing; they must not be reconstructed from source size or
timestamps. A confirmatory run cannot be analyzed while required provenance is
missing.

The task specifications themselves are not currently present in this
repository. `data/study-manifest.json` therefore starts in a blocked state.
Populate it with immutable, repository-relative specifications and hashes before
generation. This is necessary to satisfy the requirement to reuse the exact
prior specifications.

## Outcomes

### Primary Outcomes

- Independent-test pass rate
- Compilation success
- Security finding count, with severity recorded separately
- Feature-completeness rate

### Secondary Outcomes

- Maintained, randomized, and stress-test pass rates
- Performance, memory, and startup time
- LOC, modules, classes, functions, cyclomatic complexity, and documentation
  coverage
- Generation credits, generation wall-clock time, and retry count
- Architecture and maintainability scores, only when a frozen rubric exists

No pooled all-domain correctness percentage or universal quality score is
permitted. Domain-specific outcome definitions and denominators must remain
visible in every table.

## Evaluation Procedure

1. Generate every implementation independently; do not reuse prior source.
2. Store the generated artifact under a stable repository-relative ID and hash
   it before evaluation.
3. Run the same maintained, independent, randomized, stress, security, and
   feature-completeness evaluators for both arms.
4. Capture evaluator stdout/stderr, exact command, seed, timeout, and revision.
5. Record each defect in `data/failures.csv` using the protocol categories.
6. Blind manual architecture, documentation, and maintainability scoring to arm
   whenever practical. Record rubric and rater identity or automated tool
   revision.

## Statistical Plan

All comparisons are stratified by benchmark and level. For each numeric outcome
and arm, report `n`, mean, median, sample standard deviation, variance,
minimum, maximum, coefficient of variation, and deterministic bootstrap 95%
confidence intervals for the mean.

For an arm comparison, report the Vinglish Zero minus baseline mean difference,
bootstrap 95% confidence interval, two-sided randomization-test p-value, and
Cliff's delta. Randomization tests and effect sizes are descriptive support,
not a substitute for the confidence interval. Binary outcomes are reported as
counts and proportions; any confidence interval method must be recorded with
the result.

The analysis uses fixed-seed resampling and a fixed number of permutations so
that output is reproducible from the raw CSV. It does not claim normality.
Where a cell has fewer than two observations, variance and inferential results
are reported as unavailable. Multiple comparisons must be labeled exploratory
unless a correction procedure was preregistered before collection.

Complexity scaling is analyzed within each arm and domain using level as an
ordered factor. The report must show the five level-specific distributions;
it must not infer a monotonic trend from aggregate totals alone.

## Failure Taxonomy

Use one or more of these categories for every observed defect:

`parser`, `semantic`, `algorithm`, `architecture`, `state_management`,
`concurrency`, `memory`, `security`, `optimization`, `transaction`, `api`,
`documentation`, and `testing`.

The evaluator must distinguish observed failures from review-only risks and
from unsupported features outside the frozen level specification.

## Validity And Stopping Rules

The report must discuss internal, construct, external, and evaluator validity,
plus prompt sensitivity, model nondeterminism, generation retries, selection
bias, and hardware variance. Stop and mark the cohort incomplete when a task
specification, model snapshot, evaluator revision, or control variable changes.

The analysis may conclude only what the observed controlled data supports. It
must distinguish association from causation and report evidence contradicting
every positive claim.
