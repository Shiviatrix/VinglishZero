# Cross-Benchmark Study

## Scope

This document summarizes five completed, independent comparisons of generated
Python systems labeled **baseline** and **Vinglish Zero-assisted**:

- Chess engine
- JSON parser
- Markdown parser
- Regular-expression engine
- SQL database engine

It is a descriptive research record, not a claim that Vinglish Zero caused an
outcome. The compared systems are not this Rust repository, and their
workload-specific correctness and performance results must not be interpreted
as benchmarks of `vz` itself. For the Vinglish Zero CLI benchmark harness, see
[Benchmarking](../benchmarking.md).

The comparisons are exploratory prior evidence, not input to the confirmatory
[controlled generation study](../../research/controlled-study/README.md).

## Generation Resources

The following generation credits and elapsed times were supplied after the
domain reports were completed. They are recorded verbatim as aggregate session
data; prompts, model versions, temperatures, retries, and session boundaries
were not recorded. Consequently, the values describe these five pairs only and
cannot establish a causal effect of semantic reasoning.

| Benchmark | Baseline credits | VZ credits | Credit delta | Baseline time | VZ time | Time delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Chess | 6 | 5 | -1 | 5:01 | 4:54 | -0:07 |
| JSON | 4 | 4 | 0 | 3:33 | 4:22 | +0:49 |
| Markdown | 5 | 3 | -2 | 3:03 | 3:15 | +0:12 |
| Regex | 8 | 2 | -6 | 4:46 | 3:19 | -1:27 |
| SQL | 5 | 6 | +1 | 5:11 | 3:53 | -1:18 |
| **Total** | **28** | **20** | **-8 (-28.6%)** | **21:34** | **19:43** | **-1:51 (-8.6%)** |

Negative deltas favor the Vinglish Zero-assisted run. Vinglish Zero used fewer
recorded credits in three tasks, the same amount in one, and more in one. It
finished sooner in Chess, Regex, and SQL, but took longer in JSON and Markdown.

## Domain Findings

| Domain | Independent evidence | Descriptive outcome |
| --- | --- | --- |
| Chess | Both passed rule/perft probes; baseline won 100/100 fixed depth-1 games; VZ was faster in one depth-3 search probe. | Baseline had the stronger measured match result; VZ was smaller and faster in one isolated search. |
| JSON | Both passed core, randomized, and large-input checks. VZ parsed and serialized faster; baseline started faster. | Functional parity for the evaluated corpus. |
| Markdown | Both passed 22/23 independent checks. Baseline failed nested emphasis; VZ accepted an unsafe `javascript:` URL. | Performance and size favored VZ; safety policy favored baseline. |
| Regex | VZ passed more independent behavioral assertions and compiled faster; baseline matched/searched faster with lower traced allocation. Both had semantic defects. | Trade-off; neither implementation is an overall winner. |
| SQL | Baseline passed all 28 independent scenarios; VZ passed 23 and led every measured runtime category. | Runtime favored VZ, while correctness, feature coverage, and defensive architecture favored baseline. |

## Aggregate Static Metrics

| Aggregate across the five comparisons | Baseline | VZ-assisted | Difference |
| --- | ---: | ---: | ---: |
| Non-blank LOC | 2,156 | 1,482 | VZ-assisted 31.3% smaller |
| Modules | 26 | 18 | VZ-assisted 30.8% fewer |
| Classes | 55 | 35 | VZ-assisted 36.4% fewer |
| Functions/methods | 182 | 128 | VZ-assisted 29.7% fewer |
| Documented definitions | 44/237 (18.6%) | 23/163 (14.1%) | Baseline +4.5 points |

VZ-assisted output was smaller in four of five comparisons; JSON was 5.1%
larger. Across JSON, Markdown, Regex, and SQL, where aggregate complexity was
reported consistently, VZ-assisted output totaled 530 versus baseline's 674,
or 21.4% lower. These are static proxies, not direct measures of
maintainability or quality.

## Interpretation

The study supports a narrow observation: in this five-pair sample,
VZ-assisted outputs were usually smaller, frequently faster on selected
operations, and associated with lower recorded credits and elapsed generation
time in aggregate. The same data also shows material trade-offs:

- Baseline won the fixed Chess match and SQL correctness corpus.
- Baseline had faster Regex matching/searching and lower allocation.
- VZ-assisted Markdown handling had a security-relevant URL-validation gap.
- Both Regex implementations had independent semantic defects.
- JSON showed no evaluated correctness winner.

No causal claim is justified. The data does **not** show that Vinglish Zero
improves generation efficiency, correctness, safety, maintainability, or
production readiness in general. It only documents the outcomes of these five
comparisons.

## Reproducibility And Limits

The underlying domain reports used Python 3.13.7 on a local macOS environment
and domain-specific harnesses. They are intentionally not included in this
repository because they contain separate comparison projects and local runtime
artifacts. This summary preserves the reported measurements and conclusions,
not a portable executable benchmark suite.

Key threats to validity:

- No paired repeated generation, randomization, semantic-reasoning ablation,
  or blinded evaluation.
- Incomplete generation provenance: prompts, model configuration, retry count,
  and exact credit semantics are unavailable.
- Heterogeneous correctness denominators and workload-specific microbenchmarks
  cannot be pooled into a meaningful global correctness percentage.
- LOC, AST complexity, startup time, RSS, and traced allocations measure
  different properties.
- Five dependency-free Python examples do not generalize to production systems
  or to Vinglish Zero's Rust adapter/reasoning architecture.

## Future Evaluation Protocol

Future studies should preregister tasks and success criteria; generate multiple
paired samples under identical prompts, models, temperatures, and budgets;
record credits, retries, token counts, and wall time; blind evaluation where
possible; use public raw artifacts; and repeat measurements on controlled
hardware. Security, fuzzing, property, mutation, load, and API-usability tests
should accompany domain-specific correctness benchmarks.
