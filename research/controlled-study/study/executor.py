"""Frozen-command execution for generation and benchmark evaluation."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

try:
    import resource
except ImportError:  # pragma: no cover - Windows does not provide resource.
    resource = None  # type: ignore[assignment]

from .failures import classify
from .errors import StudyExecutionError
from .manifest import Command, ExecutionManifest, sha256_file
from .metrics import collect
from .scheduler import Run
from .storage import NOT_AVAILABLE, RUN_COLUMNS, StudyStorage


@dataclass(frozen=True)
class CommandResult:
    """Captured outcome of one exact command invocation."""

    name: str
    argv: list[str]
    exit_code: int
    duration_seconds: float
    stdout: str
    stderr: str
    timed_out: bool


def utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def sha256_text(text: str) -> str:
    return f"sha256:{hashlib.sha256(text.encode('utf-8')).hexdigest()}"


def render(command: Command, values: dict[str, str]) -> list[str]:
    """Render only documented placeholders in a shell-free argv template."""

    try:
        return [item.format_map(values) for item in command.argv]
    except KeyError as error:
        raise StudyExecutionError(f"command template references unknown placeholder: {error.args[0]}") from error


def execute(name: str, command: Command, values: dict[str, str], cwd: Path) -> CommandResult:
    argv = render(command, values)
    started = time.monotonic()
    try:
        completed = subprocess.run(
            argv,
            cwd=cwd,
            capture_output=True,
            check=False,
            text=True,
            timeout=command.timeout_seconds,
            env={**os.environ, "PYTHONHASHSEED": "0"},
        )
        return CommandResult(name, argv, completed.returncode, time.monotonic() - started, completed.stdout, completed.stderr, False)
    except subprocess.TimeoutExpired as error:
        stdout = error.stdout if isinstance(error.stdout, str) else ""
        stderr = error.stderr if isinstance(error.stderr, str) else ""
        return CommandResult(name, argv, 124, time.monotonic() - started, stdout, stderr, True)
    except OSError as error:
        return CommandResult(name, argv, 127, time.monotonic() - started, "", str(error), False)


def read_metrics(result: CommandResult) -> dict[str, Any]:
    """Read a successful evaluator's JSON metric object without inventing defaults."""

    if result.exit_code != 0:
        return {}
    try:
        value = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise StudyExecutionError(f"{result.name} succeeded but did not emit a JSON metric object") from error
    if not isinstance(value, dict):
        raise StudyExecutionError(f"{result.name} succeeded but did not emit a JSON metric object")
    return value


METRIC_REQUIREMENTS = {
    "generate": ("generation_credits", "retry_count"),
    "maintained_tests": ("passed", "total"),
    "independent_tests": ("passed", "total"),
    "randomized_tests": ("passed", "total"),
    "stress_tests": ("passed", "total"),
    "feature_evaluation": ("passed", "total"),
    "security_evaluation": ("findings",),
    "performance_evaluation": ("runtime_seconds", "memory_bytes", "startup_seconds"),
}


def validated_metrics(result: CommandResult) -> dict[str, Any]:
    """Require the schema declared for each successful metric-producing command."""

    if result.name not in METRIC_REQUIREMENTS:
        return {}
    metrics = read_metrics(result)
    if result.exit_code == 0:
        missing = [field for field in METRIC_REQUIREMENTS.get(result.name, ()) if field not in metrics]
        if missing:
            raise StudyExecutionError(f"{result.name} succeeded but omitted required metrics: {', '.join(missing)}")
    return metrics


class StudyExecutor:
    """Executes one scheduler unit and records all observable execution evidence."""

    def __init__(self, manifest: ExecutionManifest, storage: StudyStorage, workspace: Path) -> None:
        self.manifest = manifest
        self.storage = storage
        self.workspace = workspace.resolve()

    def run(self, run: Run) -> dict[str, Any]:
        """Execute a run and commit a terminal state for every recoverable failure."""

        directory = self.storage.run_dir(run)
        artifact = directory / "artifact"
        started_at = utc_now()
        values = {
            "study_id": run.study_id,
            "benchmark": run.benchmark,
            "level": run.level,
            "arm": run.arm,
            "replicate": str(run.replicate),
            "generation_id": run.generation_id,
            "run_dir": str(directory),
            "artifact_dir": str(artifact),
            "manifest_sha256": self.manifest.digest,
        }
        results: list[CommandResult] = []
        try:
            artifact.mkdir(parents=True, exist_ok=True)
            attempt = self.storage.read_attempt(run) or {"started_at_utc": started_at, "commands": {}}
            started_at = attempt["started_at_utc"]
            for name, command in self.manifest.commands.items():
                recovered = attempt["commands"].get(name)
                if recovered is not None:
                    try:
                        result = CommandResult(**recovered)
                    except TypeError as error:
                        raise StudyExecutionError(f"attempt checkpoint has malformed command result: {name}") from error
                else:
                    result = execute(name, command, values, self.workspace)
                    self.storage.write_log(run, name, result.stdout, result.stderr)
                    attempt["commands"][name] = result.__dict__
                    self.storage.write_attempt(run, attempt)
                results.append(result)
            ended_at = utc_now()
            command_metrics = {result.name: validated_metrics(result) for result in results}
            generated = results[0]
            compile_result = next(result for result in results if result.name == "compile")
            native_metrics = collect(artifact)
            performance = command_metrics["performance_evaluation"]
            feature = command_metrics["feature_evaluation"]
            security = command_metrics["security_evaluation"]
            artifact_hash = sha256_file(artifact) if artifact.is_file() else self._tree_hash(artifact)
            state = "SUCCESS" if all(result.exit_code == 0 for result in results) else "FAILED"
            reason = NOT_AVAILABLE if state == "SUCCESS" else "one or more frozen commands returned a non-zero exit status"
            row = self._row(run, artifact, artifact_hash, started_at, ended_at, generated, compile_result, results, native_metrics, performance, feature, security, state, reason)
            failures = self._failures(run, artifact, results)
            observation = {
                "run": values,
                "row": row,
                "commands": [result.__dict__ for result in results],
                "native_metrics": native_metrics,
                "command_metrics": command_metrics,
                "failures": failures,
                "host": self._host_metadata(),
            }
            self.storage.commit(run, observation)
            return observation
        except KeyboardInterrupt:
            return self._commit_terminal_failure(run, artifact, values, results, started_at, "INTERRUPTED", "runner interrupted")
        except (StudyExecutionError, OSError, ValueError, TypeError) as error:
            return self._commit_terminal_failure(run, artifact, values, results, started_at, "FAILED", str(error))

    def _commit_terminal_failure(
        self, run: Run, artifact: Path, values: dict[str, str], results: list[CommandResult], started_at: str, state: str, reason: str,
    ) -> dict[str, Any]:
        """Persist a schema-complete terminal record even if evaluation cannot continue."""

        try:
            artifact_hash = self._tree_hash(artifact) if artifact.exists() else NOT_AVAILABLE
        except OSError:
            artifact_hash = NOT_AVAILABLE
        row = {field: NOT_AVAILABLE for field in RUN_COLUMNS}
        row.update({
            "study_id": run.study_id, "study_version": run.study_id, "manifest_sha256": self.manifest.digest, "benchmark": run.benchmark,
            "benchmark_version": self.manifest.required_artifacts["task_specification_bundle"], "level": run.level,
            "arm": run.arm, "replicate": str(run.replicate), "generation_id": run.generation_id,
            "terminal_state": state, "terminal_reason": reason or NOT_AVAILABLE,
            "artifact_path": str(artifact.relative_to(self.storage.root)), "artifact_sha256": artifact_hash,
            "prompt_sha256": self.manifest.required_artifacts["task_specification_bundle"],
            "system_prompt_sha256": self.manifest.required_artifacts["system_prompt"],
            "evaluation_prompt_sha256": self.manifest.required_artifacts["evaluation_prompt"],
            "model_id": self.manifest.provenance["model_id"], "model_snapshot": self.manifest.provenance["model_snapshot"],
            "temperature": self.manifest.provenance["temperature"], "top_p": self.manifest.provenance["top_p"],
            "seed_policy": self.manifest.provenance["seed_policy"], "tool_budget": self.manifest.provenance["tool_budget"],
            "git_commit": self.manifest.provenance["git_commit"], "hardware_id": self.manifest.provenance["hardware_id"],
            "os": platform.platform(), "python_version": platform.python_version(), "rust_version": self._rust_version(),
            "evaluator_revision": self.manifest.required_artifacts["evaluator_revision"],
            "started_at_utc": started_at, "finished_at_utc": utc_now(), "compilation_success": NOT_AVAILABLE,
        })
        failure = {
            "study_id": run.study_id, "generation_id": run.generation_id, "benchmark": run.benchmark,
            "level": run.level, "arm": run.arm, "replicate": str(run.replicate), "category": "unknown",
            "severity": "error", "observed_by": "study_runner", "test_case_id": "", "reproducible": "true",
            "artifact_path": row["artifact_path"], "detail": reason[:10_000],
        }
        observation = {
            "run": values, "row": row, "commands": [result.__dict__ for result in results],
            "native_metrics": {}, "command_metrics": {}, "failures": [failure], "host": self._host_metadata(),
        }
        self.storage.commit(run, observation)
        return observation

    @staticmethod
    def _tree_hash(directory: Path) -> str:
        digest = hashlib.sha256()
        for path in sorted(path for path in directory.rglob("*") if path.is_file()):
            digest.update(str(path.relative_to(directory)).encode("utf-8"))
            digest.update(b"\0")
            digest.update(path.read_bytes())
            digest.update(b"\0")
        return f"sha256:{digest.hexdigest()}"

    def _row(
        self, run: Run, artifact: Path, artifact_hash: str, started_at: str, ended_at: str, generated: CommandResult,
        compile_result: CommandResult, results: list[CommandResult], native: dict[str, Any], performance: dict[str, Any],
        feature: dict[str, Any], security: dict[str, Any], terminal_state: str, terminal_reason: str,
    ) -> dict[str, Any]:
        test = {result.name: result for result in results}
        def outcome(name: str) -> tuple[int, int]:
            metrics = read_metrics(test[name])
            return int(metrics.get("passed", 0)), int(metrics.get("total", 0))
        maintained, independent, randomized, stress = (outcome(name) for name in ("maintained_tests", "independent_tests", "randomized_tests", "stress_tests"))
        return {
            "study_id": run.study_id, "study_version": run.study_id, "manifest_sha256": self.manifest.digest, "benchmark": run.benchmark,
            "benchmark_version": self.manifest.required_artifacts["task_specification_bundle"], "level": run.level, "arm": run.arm,
            "replicate": run.replicate, "generation_id": run.generation_id, "terminal_state": terminal_state,
            "terminal_reason": terminal_reason,
            "artifact_path": str(artifact.relative_to(self.storage.root)), "artifact_sha256": artifact_hash,
            "prompt_sha256": self.manifest.required_artifacts["task_specification_bundle"],
            "system_prompt_sha256": self.manifest.required_artifacts["system_prompt"],
            "evaluation_prompt_sha256": self.manifest.required_artifacts["evaluation_prompt"],
            "model_id": self.manifest.provenance["model_id"], "model_snapshot": self.manifest.provenance["model_snapshot"],
            "temperature": self.manifest.provenance["temperature"], "top_p": self.manifest.provenance["top_p"],
            "seed_policy": self.manifest.provenance["seed_policy"], "tool_budget": self.manifest.provenance["tool_budget"],
            "generation_token_usage": command_metrics_value(generated, "token_usage"),
            "generation_tool_usage": json.dumps(command_metrics_value(generated, "tool_usage"), sort_keys=True),
            "git_commit": self.manifest.provenance["git_commit"], "hardware_id": self.manifest.provenance["hardware_id"],
            "os": platform.platform(), "python_version": platform.python_version(), "rust_version": self._rust_version(),
            "evaluator_revision": self.manifest.required_artifacts["evaluator_revision"],
            "started_at_utc": started_at, "finished_at_utc": ended_at,
            "generation_credits": command_metrics_value(generated, "generation_credits"),
            "generation_wall_seconds": generated.duration_seconds, "retry_count": command_metrics_value(generated, "retry_count"),
            "execution_duration_seconds": sum(result.duration_seconds for result in results),
            "evaluation_duration_seconds": sum(result.duration_seconds for result in results if result.name not in {"generate", "compile"}),
            "compilation_success": str(compile_result.exit_code == 0).lower(),
            "maintained_tests_passed": maintained[0], "maintained_tests_total": maintained[1],
            "independent_tests_passed": independent[0], "independent_tests_total": independent[1],
            "randomized_tests_passed": randomized[0], "randomized_tests_total": randomized[1],
            "stress_tests_passed": stress[0], "stress_tests_total": stress[1],
            "feature_points": feature.get("passed", 0), "feature_total": feature.get("total", 0),
            "security_findings": security.get("findings", 0), "architecture_score": NOT_AVAILABLE, "maintainability_score": NOT_AVAILABLE,
            "documentation_coverage_percent": native["documentation_coverage_percent"], "cyclomatic_complexity": native["cyclomatic_complexity"],
            "loc": native["loc"], "module_count": native["module_count"], "class_count": native["class_count"],
            "function_count": native["function_count"], "performance_seconds": performance.get("runtime_seconds", NOT_AVAILABLE),
            "memory_bytes": performance.get("memory_bytes", NOT_AVAILABLE), "peak_allocation_bytes": performance.get("peak_allocation_bytes", NOT_AVAILABLE),
            "cpu_seconds": performance.get("cpu_seconds", NOT_AVAILABLE), "throughput": performance.get("throughput", NOT_AVAILABLE),
            "startup_seconds": performance.get("startup_seconds", NOT_AVAILABLE),
            "notes": NOT_AVAILABLE,
        }

    @staticmethod
    def _rust_version() -> str:
        try:
            completed = subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=False, timeout=5)
        except OSError:
            return "unavailable"
        return completed.stdout.strip() if completed.returncode == 0 else "unavailable"

    def _failures(self, run: Run, artifact: Path, results: list[CommandResult]) -> list[dict[str, str]]:
        rows = []
        for result in results:
            if result.exit_code == 0:
                continue
            detail = (result.stderr or result.stdout).strip()
            rows.append({
                "study_id": run.study_id, "generation_id": run.generation_id, "benchmark": run.benchmark,
                "level": run.level, "arm": run.arm, "replicate": str(run.replicate), "category": classify(detail),
                "severity": "error", "observed_by": result.name, "test_case_id": "", "reproducible": "true",
                "artifact_path": str(artifact.relative_to(self.storage.root)), "detail": detail[:10_000],
            })
        return rows

    @staticmethod
    def _host_metadata() -> dict[str, Any]:
        peak_rss = None
        if resource is not None:
            peak_rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        return {"python": sys.version, "platform": platform.platform(), "peak_rss": peak_rss}


def command_metrics_value(result: CommandResult, field: str) -> Any:
    return read_metrics(result).get(field, 0)
