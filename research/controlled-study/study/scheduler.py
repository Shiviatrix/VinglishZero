"""Deterministic factorial scheduling for independently generated runs."""

from __future__ import annotations

import hashlib
from dataclasses import dataclass

from .manifest import ExecutionManifest


@dataclass(frozen=True, order=True)
class Run:
    """One preregistered experimental unit."""

    study_id: str
    benchmark: str
    level: str
    arm: str
    replicate: int
    generation_id: str


def run_id(study_id: str, benchmark: str, level: str, arm: str, replicate: int) -> str:
    """Create a stable globally unique ID from an immutable factorial cell."""

    identity = f"{study_id}\0{benchmark}\0{level}\0{arm}\0{replicate}".encode("utf-8")
    return f"run-{hashlib.sha256(identity).hexdigest()[:20]}"


def schedule(manifest: ExecutionManifest) -> list[Run]:
    """Return every cell in canonical order without random scheduling choices."""

    return [
        Run(
            manifest.study_id,
            benchmark,
            level,
            arm,
            replicate,
            run_id(manifest.study_id, benchmark, level, arm, replicate),
        )
        for benchmark in manifest.benchmarks
        for level in manifest.levels
        for arm in manifest.arms
        for replicate in range(1, manifest.replications_per_cell + 1)
    ]
