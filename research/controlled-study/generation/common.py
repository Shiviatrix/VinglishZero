"""Shared deterministic helpers for controlled-study generation inputs."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from study.errors import StudyExecutionError


def utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def canonical_json(value: object) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")


def sha256_bytes(value: bytes) -> str:
    return f"sha256:{hashlib.sha256(value).hexdigest()}"


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    try:
        with path.open("rb") as source:
            for block in iter(lambda: source.read(1024 * 1024), b""):
                digest.update(block)
    except OSError as error:
        raise StudyExecutionError(f"cannot hash {path}: {error}") from error
    return f"sha256:{digest.hexdigest()}"


def write_atomic(path: Path, data: bytes) -> None:
    try:
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
    except OSError as error:
        raise StudyExecutionError(f"cannot write {path}: {error}") from error


def read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise StudyExecutionError(f"cannot read JSON {path}: {error}") from error
    if not isinstance(value, dict):
        raise StudyExecutionError(f"JSON document {path} must be an object")
    return value


def require_file(path: Path, label: str) -> Path:
    resolved = path.expanduser().resolve()
    if not resolved.is_file():
        raise StudyExecutionError(f"{label} must be a readable file: {path}")
    return resolved


def executable(argv: object, label: str) -> list[str]:
    if not isinstance(argv, list) or not argv or not all(isinstance(item, str) and item for item in argv):
        raise StudyExecutionError(f"{label} must be a non-empty argv string array")
    program = argv[0]
    if Path(program).is_absolute():
        if not os.access(program, os.X_OK):
            raise StudyExecutionError(f"{label} executable is unavailable: {program}")
    elif shutil.which(program) is None:
        raise StudyExecutionError(f"{label} executable is unavailable on PATH: {program}")
    return list(argv)


def command_output(argv: list[str], cwd: Path, label: str, timeout_seconds: int = 30) -> dict[str, object]:
    try:
        completed = subprocess.run(argv, cwd=cwd, capture_output=True, text=True, check=False, timeout=timeout_seconds)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise StudyExecutionError(f"{label} could not execute: {error}") from error
    if completed.returncode != 0:
        raise StudyExecutionError(f"{label} failed with exit code {completed.returncode}: {completed.stderr.strip()[:1000]}")
    return {"argv": argv, "stdout": completed.stdout, "stderr": completed.stderr, "exit_code": completed.returncode}


def git_state(cwd: Path) -> dict[str, str]:
    def probe(argv: list[str], fallback: str) -> str:
        try:
            completed = subprocess.run(argv, cwd=cwd, capture_output=True, text=True, check=False, timeout=10)
        except OSError:
            return fallback
        return completed.stdout.strip() if completed.returncode == 0 and completed.stdout.strip() else fallback
    return {
        "commit": probe(["git", "rev-parse", "HEAD"], "NOT_AVAILABLE"),
        "status": probe(["git", "status", "--porcelain=v1"], "NOT_AVAILABLE"),
    }


def environment_manifest(cwd: Path) -> dict[str, object]:
    memory_bytes: int | str = "NOT_AVAILABLE"
    try:
        pages, page_size = os.sysconf("SC_PHYS_PAGES"), os.sysconf("SC_PAGE_SIZE")
        memory_bytes = int(pages) * int(page_size)
    except (AttributeError, OSError, ValueError):
        pass
    try:
        rust = subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=False, timeout=10)
        rust_version = rust.stdout.strip() if rust.returncode == 0 else "NOT_AVAILABLE"
    except OSError:
        rust_version = "NOT_AVAILABLE"
    return {
        "working_directory": str(cwd.resolve()),
        "git": git_state(cwd),
        "python_version": sys.version,
        "rust_version": rust_version,
        "os": platform.platform(),
        "cpu_count": os.cpu_count() or "NOT_AVAILABLE",
        "machine": platform.machine(),
        "memory_bytes": memory_bytes,
    }
