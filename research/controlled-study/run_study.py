#!/usr/bin/env python3
"""Execute the immutable controlled generation study and render its raw-data report.

This program never generates mock implementations or estimates missing values.
It runs only frozen, shell-free command templates from a `ready` manifest.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

from study.errors import StudyExecutionError
from study.executor import StudyExecutor
from study.manifest import load_manifest
from study.report import render
from study.scheduler import schedule
from study.storage import StudyStorage


STUDY_ROOT = Path(__file__).resolve().parent


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=STUDY_ROOT / "data/study-manifest.json")
    parser.add_argument("--output", type=Path, default=STUDY_ROOT / "results/execution", help="durable output directory")
    parser.add_argument("--workspace", type=Path, default=Path.cwd(), help="working directory for frozen commands")
    parser.add_argument("--max-runs", type=int, help="execute at most this many unfinished cells")
    parser.add_argument("--analyze-only", action="store_true", help="render analysis from already completed observations")
    parser.add_argument("--repair-exports", action="store_true", help="explicitly rebuild CSV exports from validated observations")
    arguments = parser.parse_args()
    if arguments.max_runs is not None and arguments.max_runs <= 0:
        parser.error("--max-runs must be positive")
    try:
        manifest = load_manifest(arguments.manifest)
        storage = StudyStorage(arguments.output, verify_exports=not arguments.repair_exports)
        storage.bind_manifest(manifest.digest)
        if arguments.repair_exports:
            with storage.export_lock():
                backups = storage.repair_exports()
            for backup in backups:
                print(f"preserved mismatched export: {backup}")
        scheduled = schedule(manifest)
        if not arguments.analyze_only:
            executor = StudyExecutor(manifest, storage, arguments.workspace)
            executed = 0
            for run in scheduled:
                with storage.run_lock(run):
                    if storage.complete(run):
                        continue
                    executor.run(run)
                executed += 1
                if arguments.max_runs is not None and executed >= arguments.max_runs:
                    print(f"checkpointed {executed} run(s); resume with the same command")
                    return 0
        incomplete = [run for run in scheduled if not storage.complete(run)]
        if incomplete:
            raise StudyExecutionError(f"study is incomplete: {len(incomplete)} scheduled run(s) have no terminal observation")
        analysis = Path(__file__).parent / "analysis" / "analyze.py"
        completed = subprocess.run(
            [sys.executable, str(analysis), "--manifest", str(arguments.manifest), "--runs", str(storage.runs_path),
             "--failures", str(storage.failures_path), "--output", str(arguments.output / "analysis"), "--require-complete"],
            check=False,
        )
        if completed.returncode:
            return completed.returncode
        render(arguments.output / "analysis" / "summary.json", arguments.output / "analysis" / "comparisons.json", arguments.output / "report")
        print(f"study completed: {len(scheduled)} observed runs; reports: {arguments.output / 'report'}")
    except (StudyExecutionError, OSError, ValueError, KeyError, TypeError) as error:
        print(f"study blocked: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
