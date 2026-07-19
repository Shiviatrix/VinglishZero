"""Validated, atomic, resumable storage for controlled-study observations."""

from __future__ import annotations

import csv
import hashlib
import io
import json
import os
import tempfile
from pathlib import Path
from typing import Any

from .errors import StudyExecutionError
from .locking import FileLock
from .scheduler import Run


NOT_AVAILABLE = "NOT_AVAILABLE"
TERMINAL_STATES = {"SUCCESS", "FAILED", "SKIPPED", "ABORTED", "INTERRUPTED"}
RUN_COLUMNS = (
    "study_id,study_version,manifest_sha256,benchmark,benchmark_version,level,arm,replicate,generation_id,terminal_state,terminal_reason,artifact_path,artifact_sha256,"
    "prompt_sha256,system_prompt_sha256,evaluation_prompt_sha256,model_id,model_snapshot,"
    "temperature,top_p,seed_policy,tool_budget,generation_token_usage,generation_tool_usage,git_commit,hardware_id,os,python_version,rust_version,evaluator_revision,"
    "started_at_utc,finished_at_utc,generation_credits,generation_wall_seconds,retry_count,execution_duration_seconds,evaluation_duration_seconds,"
    "compilation_success,maintained_tests_passed,maintained_tests_total,independent_tests_passed,"
    "independent_tests_total,randomized_tests_passed,randomized_tests_total,stress_tests_passed,"
    "stress_tests_total,feature_points,feature_total,security_findings,architecture_score,"
    "maintainability_score,documentation_coverage_percent,cyclomatic_complexity,loc,module_count,"
    "class_count,function_count,performance_seconds,memory_bytes,peak_allocation_bytes,cpu_seconds,throughput,startup_seconds,notes"
).split(",")
FAILURE_COLUMNS = (
    "study_id,generation_id,benchmark,level,arm,replicate,category,severity,observed_by,"
    "test_case_id,reproducible,artifact_path,detail"
).split(",")


class StudyStorage:
    """Observations are canonical; CSV exports are integrity-checked derivatives."""

    def __init__(self, root: Path, *, verify_exports: bool = True) -> None:
        self.root = root.resolve()
        self.runs_path = self.root / "runs.csv"
        self.failures_path = self.root / "failures.csv"
        self.observations = self.root / "observations"
        try:
            self.observations.mkdir(parents=True, exist_ok=True)
        except OSError as error:
            raise StudyExecutionError(f"cannot create study storage {self.observations}: {error}") from error
        if verify_exports:
            with self.export_lock():
                self.verify_exports()

    def export_lock(self) -> FileLock:
        return FileLock(self.root / ".locks" / "exports.lock")

    def run_lock(self, run: Run) -> FileLock:
        return FileLock(self.root / ".locks" / f"{run.generation_id}.lock")

    def bind_manifest(self, digest: str) -> None:
        """Bind this output directory to one immutable manifest byte sequence."""

        path = self.root / "manifest.sha256"
        encoded = (digest + "\n").encode("ascii")
        with self.export_lock():
            if not path.exists():
                self._atomic_bytes(path, encoded)
                return
            try:
                existing = path.read_bytes()
            except OSError as error:
                raise StudyExecutionError(f"cannot read manifest binding {path}: {error}") from error
            if existing != encoded:
                raise StudyExecutionError(
                    f"study output {self.root} is bound to a different manifest; start a new output directory"
                )

    def run_dir(self, run: Run) -> Path:
        return self.observations / run.generation_id

    def complete(self, run: Run) -> bool:
        path = self.run_dir(run) / "observation.json"
        if not path.is_file():
            return False
        self._read_observation(path, expected_generation_id=run.generation_id)
        return True

    def read_attempt(self, run: Run) -> dict[str, Any] | None:
        path = self.run_dir(run) / "attempt.json"
        if not path.is_file():
            return None
        try:
            value = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            raise StudyExecutionError(f"cannot recover attempt {path}: {error}") from error
        if not isinstance(value, dict):
            raise StudyExecutionError(f"attempt checkpoint {path} must be an object, got {type(value).__name__}")
        if not isinstance(value.get("commands"), dict):
            raise StudyExecutionError(f"attempt checkpoint {path}.commands must be an object")
        if not isinstance(value.get("started_at_utc"), str):
            raise StudyExecutionError(f"attempt checkpoint {path}.started_at_utc must be a string")
        return value

    def write_attempt(self, run: Run, value: dict[str, Any]) -> None:
        """Publish a stage checkpoint immediately after an external command returns."""

        self.write_json(self.run_dir(run) / "attempt.json", value)

    def verify_exports(self) -> None:
        """Reject changed or malformed derived CSVs; never repair them implicitly."""

        rows, failures = self._canonical_records()
        self._verify_or_create(self.runs_path, self._csv_bytes(RUN_COLUMNS, rows), "runs.csv")
        self._verify_or_create(self.failures_path, self._csv_bytes(FAILURE_COLUMNS, failures), "failures.csv")

    def repair_exports(self) -> list[Path]:
        """Explicitly rebuild derived exports, retaining a forensic copy of mismatches."""

        rows, failures = self._canonical_records()
        repaired: list[Path] = []
        for path, data in (
            (self.runs_path, self._csv_bytes(RUN_COLUMNS, rows)),
            (self.failures_path, self._csv_bytes(FAILURE_COLUMNS, failures)),
        ):
            if path.exists() and path.read_bytes() != data:
                digest = hashlib.sha256(path.read_bytes()).hexdigest()[:16]
                backup = path.with_name(f"{path.name}.corrupt-{digest}")
                if not backup.exists():
                    self._atomic_bytes(backup, path.read_bytes())
                repaired.append(backup)
            self._atomic_bytes(path, data)
        return repaired

    def commit(self, run: Run, observation: dict[str, Any], failures: list[dict[str, Any]] | None = None) -> None:
        """Atomically persist exactly one terminal observation and refresh exports."""

        if failures is not None and observation.get("failures") != failures:
            raise StudyExecutionError("commit failures argument must match observation.failures")
        self._validate_observation(observation, self.run_dir(run) / "observation.json", run.generation_id)
        path = self.run_dir(run) / "observation.json"
        with self.export_lock():
            self.verify_exports()
            if path.exists():
                existing = self._read_observation(path, expected_generation_id=run.generation_id)
                if existing != observation:
                    raise StudyExecutionError(f"terminal observation already exists for {run.generation_id}")
                return
            self.write_json(path, observation)
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            self._atomic_bytes(path.with_suffix(".sha256"), f"sha256:{digest}\n".encode("ascii"))
            rows, failures = self._canonical_records()
            self._atomic_bytes(self.runs_path, self._csv_bytes(RUN_COLUMNS, rows))
            self._atomic_bytes(self.failures_path, self._csv_bytes(FAILURE_COLUMNS, failures))

    def write_json(self, path: Path, value: Any) -> None:
        """Atomically publish JSON or fail with a user-facing storage error."""

        try:
            encoded = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
            self._atomic_bytes(path, encoded)
        except (OSError, TypeError, ValueError) as error:
            raise StudyExecutionError(f"cannot write {path}: {error}") from error

    def write_log(self, run: Run, name: str, stdout: str, stderr: str) -> tuple[str, str]:
        directory = self.run_dir(run) / "logs"
        try:
            directory.mkdir(parents=True, exist_ok=True)
            stdout_path, stderr_path = directory / f"{name}.stdout.log", directory / f"{name}.stderr.log"
            self._atomic_bytes(stdout_path, stdout.encode("utf-8"))
            self._atomic_bytes(stderr_path, stderr.encode("utf-8"))
        except OSError as error:
            raise StudyExecutionError(f"cannot write logs for {run.generation_id}/{name}: {error}") from error
        return str(stdout_path.relative_to(self.root)), str(stderr_path.relative_to(self.root))

    def _canonical_records(self) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
        observations = [
            self._read_observation(path, expected_generation_id=path.parent.name)
            for path in sorted(self.observations.glob("*/observation.json"))
        ]
        rows = sorted((record["row"] for record in observations), key=lambda row: row["generation_id"])
        failures = [failure for record in observations for failure in record["failures"]]
        return rows, failures

    def _read_observation(self, path: Path, expected_generation_id: str | None = None) -> dict[str, Any]:
        try:
            value = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            raise StudyExecutionError(f"cannot recover observation {path}: {error}") from error
        self._validate_observation(value, path, expected_generation_id)
        checksum_path = path.with_suffix(".sha256")
        try:
            expected_checksum = checksum_path.read_text(encoding="ascii").strip()
        except OSError as error:
            raise StudyExecutionError(f"observation checksum is missing at {checksum_path}: {error}") from error
        try:
            actual_checksum = f"sha256:{hashlib.sha256(path.read_bytes()).hexdigest()}"
        except OSError as error:
            raise StudyExecutionError(f"cannot checksum observation {path}: {error}") from error
        if expected_checksum != actual_checksum:
            raise StudyExecutionError(
                f"observation checksum mismatch at {path}; expected {expected_checksum!r}, got {actual_checksum!r}"
            )
        return value

    @staticmethod
    def _validate_observation(value: Any, path: Path, expected_generation_id: str | None = None) -> None:
        if not isinstance(value, dict):
            raise StudyExecutionError(f"observation {path} must be an object, got {type(value).__name__}")
        row = value.get("row")
        if not isinstance(row, dict):
            raise StudyExecutionError(f"observation {path}.row must be an object, got {type(row).__name__}")
        missing = [field for field in RUN_COLUMNS if field not in row]
        if missing:
            raise StudyExecutionError(f"observation {path}.row is missing required fields: {', '.join(missing)}")
        for field in RUN_COLUMNS:
            if row[field] in (None, ""):
                raise StudyExecutionError(f"observation {path}.row.{field} must be non-empty, got {row[field]!r}")
            if not isinstance(row[field], (str, int, float, bool)):
                raise StudyExecutionError(
                    f"observation {path}.row.{field} expected scalar string/number/bool, got {type(row[field]).__name__}: {row[field]!r}"
                )
        if row["terminal_state"] not in TERMINAL_STATES:
            raise StudyExecutionError(f"observation {path}.row.terminal_state is invalid: {row['terminal_state']!r}")
        if expected_generation_id is not None and row["generation_id"] != expected_generation_id:
            raise StudyExecutionError(
                f"observation {path}.row.generation_id expected {expected_generation_id!r}, got {row['generation_id']!r}"
            )
        if not isinstance(value.get("commands"), list) or not all(isinstance(command, dict) for command in value["commands"]):
            raise StudyExecutionError(f"observation {path}.commands must be a list, got {type(value.get('commands')).__name__}")
        failures = value.get("failures")
        if not isinstance(failures, list) or not all(isinstance(failure, dict) for failure in failures):
            raise StudyExecutionError(f"observation {path}.failures must be a list of objects")
        for index, failure in enumerate(failures):
            missing_failure_fields = [field for field in FAILURE_COLUMNS if field not in failure]
            if missing_failure_fields:
                raise StudyExecutionError(
                    f"observation {path}.failures[{index}] is missing required fields: {', '.join(missing_failure_fields)}"
                )

    def _verify_or_create(self, path: Path, expected: bytes, label: str) -> None:
        if not path.exists():
            self._atomic_bytes(path, expected)
            return
        try:
            actual = path.read_bytes()
        except OSError as error:
            raise StudyExecutionError(f"cannot read {label} at {path}: {error}") from error
        if actual != expected:
            raise StudyExecutionError(
                f"{label} integrity check failed at {path}; preserved file differs from canonical observations. "
                "inspect it and use --repair-exports explicitly if recovery is intended"
            )

    @staticmethod
    def _csv_bytes(columns: list[str], rows: list[dict[str, Any]]) -> bytes:
        stream = io.StringIO(newline="")
        writer = csv.DictWriter(stream, fieldnames=columns, extrasaction="raise", lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
        return stream.getvalue().encode("utf-8")

    @staticmethod
    def _atomic_bytes(path: Path, data: bytes) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
        try:
            with os.fdopen(descriptor, "wb") as handle:
                handle.write(data)
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(temporary, path)
        finally:
            if os.path.exists(temporary):
                os.unlink(temporary)
