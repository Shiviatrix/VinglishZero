"""Regression tests for the controlled-study statistical gate."""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("analyze.py")
SPEC = importlib.util.spec_from_file_location("controlled_analyze", MODULE_PATH)
assert SPEC and SPEC.loader
analyze = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(analyze)


class ControlledStudyAnalysisTests(unittest.TestCase):
    def manifest(self, status: str = "ready") -> dict[str, object]:
        return {
            "study_id": "test-study",
            "status": status,
            "required_artifacts": {
                "task_specification_bundle": "sha256:task",
                "system_prompt": "sha256:system",
                "evaluation_prompt": "sha256:evaluation",
                "model_snapshot": "model-2026-07-19",
                "generation_settings": "sha256:settings",
                "hardware_manifest": "sha256:hardware",
                "evaluator_revision": "sha256:evaluator",
            },
        }

    def row(self, arm: str, replicate: int, loc: int) -> dict[str, str]:
        return {
            "study_id": "test-study",
            "benchmark": "json_parser",
            "level": "L3",
            "arm": arm,
            "replicate": str(replicate),
            "generation_id": f"{arm}-{replicate}",
            "artifact_path": f"artifacts/{arm}-{replicate}",
            "artifact_sha256": f"sha256:{arm}-{replicate}",
            "prompt_sha256": "sha256:prompt",
            "system_prompt_sha256": "sha256:system",
            "evaluation_prompt_sha256": "sha256:evaluation",
            "model_id": "test-model",
            "model_snapshot": "test-snapshot",
            "temperature": "0.2",
            "top_p": "1.0",
            "seed_policy": "independent",
            "tool_budget": "100",
            "hardware_id": "test-hardware",
            "os": "test-os",
            "python_version": "3.13",
            "evaluator_revision": "test-revision",
            "started_at_utc": "2026-07-19T00:00:00Z",
            "finished_at_utc": "2026-07-19T00:01:00Z",
            "generation_credits": "1",
            "generation_wall_seconds": "60",
            "retry_count": "0",
            "compilation_success": "true",
            "maintained_tests_passed": "10",
            "maintained_tests_total": "10",
            "independent_tests_passed": "10",
            "independent_tests_total": "10",
            "randomized_tests_passed": "10",
            "randomized_tests_total": "10",
            "stress_tests_passed": "10",
            "stress_tests_total": "10",
            "feature_points": "10",
            "feature_total": "10",
            "security_findings": "0",
            "architecture_score": "1",
            "maintainability_score": "1",
            "documentation_coverage_percent": "50",
            "cyclomatic_complexity": "10",
            "loc": str(loc),
            "module_count": "1",
            "class_count": "1",
            "function_count": "2",
            "performance_seconds": "1",
            "memory_bytes": "1024",
            "startup_seconds": "0.1",
            "notes": "",
        }

    def test_ready_manifest_and_rows_produce_deterministic_comparison(self) -> None:
        rows = [
            self.row("baseline", 1, 100),
            self.row("baseline", 2, 120),
            self.row("vinglish_zero", 1, 80),
            self.row("vinglish_zero", 2, 90),
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "manifest.json"
            path.write_text(json.dumps(self.manifest()), encoding="utf-8")
            analyze.validate_manifest(path, rows)
        analyze.validate(rows, require_complete=False)
        _, comparisons = analyze.analyze(rows)
        result = comparisons["json_parser/L3"]["loc"]
        self.assertEqual(result["mean_difference_vz_minus_baseline"], -25.0)
        self.assertIsNotNone(result["mean_difference_ci_95"])

    def test_blocked_manifest_cannot_enable_analysis(self) -> None:
        rows = [self.row("baseline", 1, 100)]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "manifest.json"
            path.write_text(json.dumps(self.manifest("blocked")), encoding="utf-8")
            with self.assertRaises(analyze.StudyError):
                analyze.validate_manifest(path, rows)

    def test_advanced_analysis_is_deterministic_and_preserves_failure_counts(self) -> None:
        rows = [
            self.row("baseline", 1, 100),
            self.row("baseline", 2, 120),
            self.row("vinglish_zero", 1, 80),
            self.row("vinglish_zero", 2, 90),
        ]
        failures = [{"category": "parser"}, {"category": "parser"}, {"category": "security"}]
        first = analyze.advanced_analysis(rows, failures)
        second = analyze.advanced_analysis(rows, failures)
        self.assertEqual(first, second)
        self.assertEqual(first["failure_frequency"], {"parser": 2, "security": 1})
        self.assertIn("loc", first["variance_decomposition"])
        self.assertIn("loc", first["correlations"])


if __name__ == "__main__":
    unittest.main()
