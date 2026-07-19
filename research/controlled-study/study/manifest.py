"""Validation and loading for immutable, command-driven study manifests."""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .errors import StudyExecutionError


REQUIRED_ARTIFACTS = (
    "task_specification_bundle",
    "system_prompt",
    "evaluation_prompt",
    "model_snapshot",
    "generation_settings",
    "hardware_manifest",
    "evaluator_revision",
)
REQUIRED_COMMANDS = (
    "generate",
    "compile",
    "maintained_tests",
    "independent_tests",
    "randomized_tests",
    "stress_tests",
    "feature_evaluation",
    "security_evaluation",
    "performance_evaluation",
)


@dataclass(frozen=True)
class Command:
    """A frozen argv template, executed without a shell."""

    argv: tuple[str, ...]
    timeout_seconds: int


@dataclass(frozen=True)
class ExecutionManifest:
    """The validated input required to execute one immutable study."""

    path: Path
    digest: str
    study_id: str
    benchmarks: tuple[str, ...]
    levels: tuple[str, ...]
    arms: tuple[str, ...]
    replications_per_cell: int
    required_artifacts: dict[str, str]
    provenance: dict[str, str]
    commands: dict[str, Command]


def sha256_file(path: Path) -> str:
    """Return the exact SHA-256 encoding used by the manifest contract."""

    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return f"sha256:{digest.hexdigest()}"


def _require_mapping(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise StudyExecutionError(f"manifest {label} must be an object")
    return value


def _command(name: str, value: Any) -> Command:
    entry = _require_mapping(value, f"commands.{name}")
    argv = entry.get("argv")
    timeout = entry.get("timeout_seconds")
    if not isinstance(argv, list) or not argv or not all(isinstance(item, str) and item for item in argv):
        raise StudyExecutionError(f"manifest commands.{name}.argv must be a non-empty string array")
    if not isinstance(timeout, int) or timeout <= 0:
        raise StudyExecutionError(f"manifest commands.{name}.timeout_seconds must be a positive integer")
    return Command(tuple(argv), timeout)


def _validate_frozen_files(path: Path, artifacts: dict[str, str]) -> None:
    """Verify every file-backed artifact before a single generation is run."""

    files = artifacts.get("files")
    if not isinstance(files, dict) or not files:
        raise StudyExecutionError("manifest required_artifacts.files must contain frozen relative paths and hashes")
    root = path.parent
    for relative, expected in files.items():
        if not isinstance(relative, str) or not isinstance(expected, str) or not expected.startswith("sha256:"):
            raise StudyExecutionError("manifest required_artifacts.files entries must be relative paths with sha256 hashes")
        candidate = (root / relative).resolve()
        if root.resolve() not in candidate.parents:
            raise StudyExecutionError(f"manifest artifact escapes study directory: {relative}")
        if not candidate.is_file():
            raise StudyExecutionError(f"frozen manifest artifact is missing: {relative}")
        actual = sha256_file(candidate)
        if actual != expected:
            raise StudyExecutionError(f"frozen manifest artifact hash mismatch: {relative}")


def load_manifest(path: Path) -> ExecutionManifest:
    """Load a ready manifest and reject incomplete or mutable experiment inputs."""

    try:
        encoded = path.read_bytes()
        raw = json.loads(encoded.decode("utf-8"))
    except OSError as error:
        raise StudyExecutionError(f"cannot read manifest {path}: {error}") from error
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise StudyExecutionError(f"cannot decode manifest {path}: {error}") from error
    if raw.get("status") != "ready":
        raise StudyExecutionError(
            "study manifest is not ready; freeze the required specifications and provenance before execution"
        )
    required_artifacts = _require_mapping(raw.get("required_artifacts"), "required_artifacts")
    missing = [key for key in REQUIRED_ARTIFACTS if not required_artifacts.get(key)]
    if missing:
        raise StudyExecutionError(f"manifest has missing immutable provenance: {', '.join(missing)}")
    _validate_frozen_files(path, required_artifacts)
    commands = _require_mapping(raw.get("commands"), "commands")
    absent_commands = [name for name in REQUIRED_COMMANDS if name not in commands]
    if absent_commands:
        raise StudyExecutionError(f"manifest is missing execution commands: {', '.join(absent_commands)}")
    provenance = _require_mapping(raw.get("provenance"), "provenance")
    required_provenance = (
        "model_id", "model_snapshot", "temperature", "top_p", "seed_policy", "tool_budget", "git_commit", "hardware_id",
    )
    missing_provenance = [key for key in required_provenance if provenance.get(key) in (None, "")]
    if missing_provenance:
        raise StudyExecutionError(f"manifest has missing execution provenance: {', '.join(missing_provenance)}")
    fields = ("study_id", "benchmarks", "levels", "arms", "replications_per_cell")
    if any(raw.get(field) in (None, "", []) for field in fields):
        raise StudyExecutionError("manifest is missing schedule fields")
    if not isinstance(raw["replications_per_cell"], int) or raw["replications_per_cell"] <= 0:
        raise StudyExecutionError("manifest replications_per_cell must be a positive integer")
    for field in ("benchmarks", "levels", "arms"):
        if not isinstance(raw[field], list) or not all(isinstance(value, str) and value for value in raw[field]):
            raise StudyExecutionError(f"manifest {field} must be a non-empty string array")
        duplicates = sorted({value for value in raw[field] if raw[field].count(value) > 1})
        if duplicates:
            raise StudyExecutionError(f"manifest {field} contains duplicate entries: {', '.join(duplicates)}")
    return ExecutionManifest(
        path=path.resolve(),
        digest=f"sha256:{hashlib.sha256(encoded).hexdigest()}",
        study_id=raw["study_id"],
        benchmarks=tuple(raw["benchmarks"]),
        levels=tuple(raw["levels"]),
        arms=tuple(raw["arms"]),
        replications_per_cell=raw["replications_per_cell"],
        required_artifacts={key: str(value) for key, value in required_artifacts.items() if key != "files"},
        provenance={key: str(value) for key, value in provenance.items()},
        commands={name: _command(name, commands[name]) for name in REQUIRED_COMMANDS},
    )
