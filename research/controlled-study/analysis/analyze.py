#!/usr/bin/env python3
"""Deterministic analysis for the controlled Vinglish Zero study.

This script consumes recorded generation data only. It does not generate code,
run benchmark implementations, or infer missing provenance.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import random
import statistics
import sys
from collections import defaultdict
from pathlib import Path
from typing import Iterable


ARMS = ("baseline", "vinglish_zero")
BENCHMARKS = (
    "chess_engine",
    "json_parser",
    "markdown_parser",
    "regex_engine",
    "sql_database_engine",
)
LEVELS = ("L1", "L2", "L3", "L4", "L5")
REQUIRED_PROVENANCE = (
    "artifact_path",
    "artifact_sha256",
    "prompt_sha256",
    "system_prompt_sha256",
    "evaluation_prompt_sha256",
    "model_id",
    "model_snapshot",
    "temperature",
    "top_p",
    "seed_policy",
    "hardware_id",
    "os",
    "python_version",
    "evaluator_revision",
    "started_at_utc",
    "finished_at_utc",
    "generation_credits",
    "generation_wall_seconds",
    "retry_count",
)
METRICS = (
    "generation_credits",
    "generation_wall_seconds",
    "retry_count",
    "execution_duration_seconds",
    "evaluation_duration_seconds",
    "generation_token_usage",
    "documentation_coverage_percent",
    "cyclomatic_complexity",
    "loc",
    "module_count",
    "class_count",
    "function_count",
    "performance_seconds",
    "memory_bytes",
    "peak_allocation_bytes",
    "cpu_seconds",
    "throughput",
    "startup_seconds",
    "security_findings",
    "architecture_score",
    "maintainability_score",
    "compilation_success_rate",
    "maintained_test_pass_rate",
    "independent_test_pass_rate",
    "randomized_test_pass_rate",
    "stress_test_pass_rate",
    "feature_completeness_rate",
)
RATIOS = {
    "maintained_test_pass_rate": ("maintained_tests_passed", "maintained_tests_total"),
    "independent_test_pass_rate": ("independent_tests_passed", "independent_tests_total"),
    "randomized_test_pass_rate": ("randomized_tests_passed", "randomized_tests_total"),
    "stress_test_pass_rate": ("stress_tests_passed", "stress_tests_total"),
    "feature_completeness_rate": ("feature_points", "feature_total"),
}


class StudyError(ValueError):
    """Raised when a dataset cannot support the preregistered analysis."""


def number(row: dict[str, str], field: str) -> float | None:
    value = row.get(field, "").strip()
    if not value or value in {"NOT_AVAILABLE", "FAILED"}:
        return None
    try:
        return float(value)
    except ValueError as error:
        raise StudyError(f"generation {row.get('generation_id', '?')}: {field} is not numeric") from error


def metric_value(row: dict[str, str], metric: str) -> float | None:
    """Derive preregistered binary and rate outcomes without imputing missing values."""

    if metric == "compilation_success_rate":
        value = row.get("compilation_success", "").strip().lower()
        if not value or value in {"not_available", "failed"}:
            return None
        if value not in {"true", "false"}:
            raise StudyError(f"generation {row.get('generation_id', '?')}: compilation_success is not boolean")
        return 1.0 if value == "true" else 0.0
    if metric in RATIOS:
        numerator, denominator = RATIOS[metric]
        passed, total = number(row, numerator), number(row, denominator)
        if passed is None or total is None or total <= 0:
            return None
        return passed / total
    return number(row, metric)


def read_rows(path: Path) -> list[dict[str, str]]:
    try:
        with path.open(newline="", encoding="utf-8") as handle:
            rows = list(csv.DictReader(handle))
    except OSError as error:
        raise StudyError(f"cannot read {path}: {error}") from error
    if not rows:
        raise StudyError("no observations recorded; no statistical result can be produced")
    required = {"study_id", "benchmark", "level", "arm", "replicate", "generation_id"}
    missing = required - set(rows[0])
    if missing:
        raise StudyError(f"runs file is missing columns: {', '.join(sorted(missing))}")
    return rows


def validate_manifest(path: Path, rows: list[dict[str, str]]) -> dict[str, object]:
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise StudyError(f"cannot read {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise StudyError(f"cannot decode {path}: {error}") from error
    if manifest.get("status") != "ready":
        raise StudyError(f"study manifest is not ready: {manifest.get('status', 'missing status')}")
    artifacts = manifest.get("required_artifacts")
    if not isinstance(artifacts, dict) or any(not value for value in artifacts.values()):
        raise StudyError("study manifest has incomplete immutable provenance")
    study_id = manifest.get("study_id")
    if not study_id or any(row["study_id"] != study_id for row in rows):
        raise StudyError("run rows do not match the immutable study manifest")
    return manifest


def validate(rows: list[dict[str, str]], require_complete: bool, manifest: dict[str, object] | None = None) -> None:
    """Validate rows against the frozen manifest, retaining legacy defaults for tests."""

    manifest = manifest or {}
    benchmarks = tuple(manifest.get("benchmarks", BENCHMARKS))
    levels = tuple(manifest.get("levels", LEVELS))
    arms = tuple(manifest.get("arms", ARMS))
    replications = manifest.get("replications_per_cell", 10)
    if not isinstance(replications, int) or replications <= 0:
        raise StudyError("manifest replications_per_cell must be a positive integer")
    seen: set[tuple[str, str, str, str]] = set()
    for row in rows:
        identity = (row["benchmark"], row["level"], row["arm"], row["replicate"])
        if row["benchmark"] not in benchmarks:
            raise StudyError(f"unknown benchmark: {row['benchmark']}")
        if row["level"] not in levels:
            raise StudyError(f"unknown level: {row['level']}")
        if row["arm"] not in arms:
            raise StudyError(f"unknown arm: {row['arm']}")
        if identity in seen:
            raise StudyError(f"duplicate study cell: {'/'.join(identity)}")
        seen.add(identity)
        missing = [field for field in REQUIRED_PROVENANCE if not row.get(field, "").strip()]
        if missing:
            raise StudyError(
                f"generation {row['generation_id']} has missing provenance: {', '.join(missing)}"
            )
    if require_complete:
        expected = {
            (benchmark, level, arm, str(replicate))
            for benchmark in benchmarks
            for level in levels
            for arm in arms
            for replicate in range(1, replications + 1)
        }
        missing = expected - seen
        if missing:
            raise StudyError(f"incomplete factorial cohort: {len(missing)} cells are missing")


def percentile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    if len(ordered) == 1:
        return ordered[0]
    position = (len(ordered) - 1) * fraction
    lower = math.floor(position)
    upper = math.ceil(position)
    return ordered[lower] + (ordered[upper] - ordered[lower]) * (position - lower)


def seeded_rng(*parts: str) -> random.Random:
    digest = hashlib.sha256("|".join(parts).encode("utf-8")).digest()
    return random.Random(int.from_bytes(digest[:8], "big"))


def bootstrap_mean_ci(values: list[float], label: str, samples: int = 10_000) -> list[float] | None:
    if len(values) < 2:
        return None
    rng = seeded_rng("bootstrap", label)
    means = [statistics.fmean(rng.choices(values, k=len(values))) for _ in range(samples)]
    return [percentile(means, 0.025), percentile(means, 0.975)]


def describe(values: list[float], label: str) -> dict[str, object]:
    result: dict[str, object] = {
        "n": len(values),
        "mean": statistics.fmean(values),
        "median": statistics.median(values),
        "minimum": min(values),
        "maximum": max(values),
        "mean_ci_95": bootstrap_mean_ci(values, label),
    }
    if len(values) >= 2:
        variance = statistics.variance(values)
        result["variance"] = variance
        result["standard_deviation"] = statistics.stdev(values)
        result["coefficient_of_variation"] = None if result["mean"] == 0 else math.sqrt(variance) / result["mean"]
        lower, upper = percentile(values, 0.25), percentile(values, 0.75)
        iqr = upper - lower
        result["tukey_outlier_count"] = sum(value < lower - 1.5 * iqr or value > upper + 1.5 * iqr for value in values)
    else:
        result.update({
            "variance": None,
            "standard_deviation": None,
            "coefficient_of_variation": None,
            "tukey_outlier_count": None,
        })
    return result


def cliffs_delta(left: list[float], right: list[float]) -> float | None:
    if not left or not right:
        return None
    more = sum(a > b for a in left for b in right)
    less = sum(a < b for a in left for b in right)
    return (more - less) / (len(left) * len(right))


def randomization_p_value(baseline: list[float], vz: list[float], label: str, samples: int = 20_000) -> float | None:
    if len(baseline) < 2 or len(vz) < 2:
        return None
    observed = statistics.fmean(vz) - statistics.fmean(baseline)
    pooled = baseline + vz
    size = len(baseline)
    rng = seeded_rng("randomization", label)
    extreme = 0
    for _ in range(samples):
        shuffled = pooled[:]
        rng.shuffle(shuffled)
        difference = statistics.fmean(shuffled[size:]) - statistics.fmean(shuffled[:size])
        extreme += abs(difference) >= abs(observed)
    return (extreme + 1) / (samples + 1)


def bootstrap_difference_ci(baseline: list[float], vz: list[float], label: str, samples: int = 10_000) -> list[float] | None:
    if len(baseline) < 2 or len(vz) < 2:
        return None
    rng = seeded_rng("difference", label)
    differences = [
        statistics.fmean(rng.choices(vz, k=len(vz))) - statistics.fmean(rng.choices(baseline, k=len(baseline)))
        for _ in range(samples)
    ]
    return [percentile(differences, 0.025), percentile(differences, 0.975)]


def analyze(rows: list[dict[str, str]]) -> tuple[dict[str, object], dict[str, object]]:
    groups: dict[tuple[str, str, str], list[dict[str, str]]] = defaultdict(list)
    for row in rows:
        groups[(row["benchmark"], row["level"], row["arm"])].append(row)
    summary: dict[str, object] = {}
    comparisons: dict[str, object] = {}
    cells = sorted({(benchmark, level) for benchmark, level, _ in groups})
    for benchmark, level in cells:
            key = f"{benchmark}/{level}"
            baseline_rows = groups.get((benchmark, level, "baseline"), [])
            vz_rows = groups.get((benchmark, level, "vinglish_zero"), [])
            if not baseline_rows and not vz_rows:
                continue
            summary[key] = {}
            comparisons[key] = {}
            for metric in METRICS:
                baseline = [value for row in baseline_rows if (value := metric_value(row, metric)) is not None]
                vz = [value for row in vz_rows if (value := metric_value(row, metric)) is not None]
                if baseline:
                    summary[key][f"baseline/{metric}"] = describe(baseline, f"{key}/baseline/{metric}")
                if vz:
                    summary[key][f"vinglish_zero/{metric}"] = describe(vz, f"{key}/vinglish_zero/{metric}")
                if baseline and vz:
                    comparisons[key][metric] = {
                        "mean_difference_vz_minus_baseline": statistics.fmean(vz) - statistics.fmean(baseline),
                        "mean_difference_ci_95": bootstrap_difference_ci(baseline, vz, f"{key}/{metric}"),
                        "randomization_p_value_two_sided": randomization_p_value(baseline, vz, f"{key}/{metric}"),
                        "cliffs_delta_vz_minus_baseline": cliffs_delta(vz, baseline),
                    }
    return summary, comparisons


def pearson(left: list[float], right: list[float]) -> float | None:
    """Return Pearson correlation only when both observed vectors have variance."""

    if len(left) < 2 or len(left) != len(right):
        return None
    left_mean, right_mean = statistics.fmean(left), statistics.fmean(right)
    numerator = sum((a - left_mean) * (b - right_mean) for a, b in zip(left, right))
    left_scale = math.sqrt(sum((a - left_mean) ** 2 for a in left))
    right_scale = math.sqrt(sum((b - right_mean) ** 2 for b in right))
    return None if left_scale == 0 or right_scale == 0 else numerator / (left_scale * right_scale)


def advanced_analysis(rows: list[dict[str, str]], failures: list[dict[str, str]]) -> dict[str, object]:
    """Produce deterministic correlations, level trends, variance, and failure frequencies."""

    numeric = {
        metric: [(row, value) for row in rows if (value := metric_value(row, metric)) is not None]
        for metric in METRICS
    }
    correlations: dict[str, dict[str, float | None]] = {}
    for left in METRICS:
        correlations[left] = {}
        left_values = {row["generation_id"]: value for row, value in numeric[left]}
        for right in METRICS:
            right_values = {row["generation_id"]: value for row, value in numeric[right]}
            shared = sorted(left_values.keys() & right_values.keys())
            correlations[left][right] = pearson([left_values[key] for key in shared], [right_values[key] for key in shared])
    trends: dict[str, object] = {}
    for metric, observations in numeric.items():
        groups: dict[tuple[str, str, str], list[float]] = defaultdict(list)
        for row, value in observations:
            groups[(row["benchmark"], row["arm"], row["level"])].append(value)
        for (benchmark, arm, level), values in sorted(groups.items()):
            trends.setdefault(f"{benchmark}/{arm}", {}).setdefault(metric, {})[level] = statistics.fmean(values)
    variance: dict[str, object] = {}
    for metric, observations in numeric.items():
        values = [value for _, value in observations]
        if len(values) < 2:
            variance[metric] = {"n": len(values), "total": None, "between_cell": None, "within_cell": None}
            continue
        mean = statistics.fmean(values)
        cells: dict[tuple[str, str, str], list[float]] = defaultdict(list)
        for row, value in observations:
            cells[(row["benchmark"], row["level"], row["arm"])].append(value)
        denominator = len(values) - 1
        between = sum(len(group) * (statistics.fmean(group) - mean) ** 2 for group in cells.values()) / denominator
        within = sum(sum((value - statistics.fmean(group)) ** 2 for value in group) for group in cells.values()) / denominator
        variance[metric] = {"n": len(values), "total": statistics.variance(values), "between_cell": between, "within_cell": within}
    frequencies: dict[str, int] = defaultdict(int)
    for failure in failures:
        frequencies[failure.get("category", "unknown")] += 1
    return {"correlations": correlations, "trends": trends, "variance_decomposition": variance, "failure_frequency": dict(sorted(frequencies.items()))}


def read_failures(path: Path) -> list[dict[str, str]]:
    try:
        with path.open(newline="", encoding="utf-8") as handle:
            rows = list(csv.DictReader(handle))
    except OSError as error:
        raise StudyError(f"cannot read {path}: {error}") from error
    if rows and "category" not in rows[0]:
        raise StudyError("failures file is missing column: category")
    return rows


def markdown(summary: dict[str, object], comparisons: dict[str, object]) -> str:
    lines = [
        "# Controlled Study Results",
        "",
        "This report is generated from recorded raw data. It does not assert causation.",
        "",
        "## Comparisons",
        "",
    ]
    if not comparisons:
        lines.append("No complete benchmark-level comparisons are available.")
        return "\n".join(lines) + "\n"
    for cell, metrics in comparisons.items():
        lines.extend([f"### {cell}", "", "| Metric | VZ - Baseline mean | 95% bootstrap CI | Randomization p | Cliff's delta |", "| --- | ---: | --- | ---: | ---: |"])
        for metric, result in metrics.items():
            interval = result["mean_difference_ci_95"]
            rendered_interval = "unavailable" if interval is None else f"[{interval[0]:.6g}, {interval[1]:.6g}]"
            p_value = result["randomization_p_value_two_sided"]
            delta = result["cliffs_delta_vz_minus_baseline"]
            lines.append(
                f"| {metric} | {result['mean_difference_vz_minus_baseline']:.6g} | {rendered_interval} | "
                f"{'unavailable' if p_value is None else f'{p_value:.6g}'} | "
                f"{'unavailable' if delta is None else f'{delta:.6g}'} |"
            )
        lines.append("")
    return "\n".join(lines)


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--runs", type=Path, required=True)
    parser.add_argument("--failures", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--require-complete", action="store_true")
    arguments = parser.parse_args()
    try:
        rows = read_rows(arguments.runs)
        manifest = validate_manifest(arguments.manifest, rows)
        validate(rows, arguments.require_complete, manifest)
        failures = read_failures(arguments.failures)
        summary, comparisons = analyze(rows)
        advanced = advanced_analysis(rows, failures)
        arguments.output.mkdir(parents=True, exist_ok=True)
        write_json(arguments.output / "summary.json", summary)
        write_json(arguments.output / "comparisons.json", comparisons)
        write_json(arguments.output / "advanced-analysis.json", advanced)
        (arguments.output / "report.md").write_text(markdown(summary, comparisons), encoding="utf-8")
    except StudyError as error:
        print(f"analysis blocked: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
