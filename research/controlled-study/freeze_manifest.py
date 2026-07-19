#!/usr/bin/env python3
"""Freeze a study draft into a ready, hash-bound manifest exactly once.

This tool does not generate benchmark data. It converts explicitly supplied
relative input bindings into SHA-256 hashes, then validates the resulting ready
manifest before publishing it atomically.
"""

from __future__ import annotations

import argparse
import json
import os
import tempfile
from pathlib import Path

from study.errors import StudyExecutionError
from study.manifest import REQUIRED_ARTIFACTS, load_manifest, sha256_file


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
            json.dump(value, handle, indent=2, sort_keys=True)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def freeze(draft: Path, output: Path) -> None:
    """Turn a draft with artifact bindings into a ready immutable manifest."""

    try:
        manifest = json.loads(draft.read_text(encoding="utf-8"))
    except OSError as error:
        raise StudyExecutionError(f"cannot read draft manifest {draft}: {error}") from error
    except json.JSONDecodeError as error:
        raise StudyExecutionError(f"cannot decode draft manifest {draft}: {error}") from error
    if manifest.get("status") != "draft":
        raise StudyExecutionError("only a manifest with status 'draft' can be frozen")
    artifacts = manifest.get("required_artifacts")
    if not isinstance(artifacts, dict):
        raise StudyExecutionError("draft manifest required_artifacts must be an object")
    bindings = artifacts.get("bindings")
    if not isinstance(bindings, dict):
        raise StudyExecutionError("draft manifest required_artifacts.bindings must map every required artifact to a file")
    root = draft.parent.resolve()
    files: dict[str, str] = {}
    for name in REQUIRED_ARTIFACTS:
        relative = bindings.get(name)
        if not isinstance(relative, str) or not relative:
            raise StudyExecutionError(f"draft manifest is missing artifact binding: {name}")
        candidate = (root / relative).resolve()
        if root not in candidate.parents or not candidate.is_file():
            raise StudyExecutionError(f"draft artifact is not a readable relative file: {relative}")
        digest = sha256_file(candidate)
        artifacts[name] = digest
        files[relative] = digest
    artifacts.pop("bindings", None)
    artifacts["files"] = dict(sorted(files.items()))
    manifest["status"] = "ready"
    manifest.pop("reason", None)
    write_json(output, manifest)
    try:
        load_manifest(output)
    except StudyExecutionError:
        output.unlink(missing_ok=True)
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--draft", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    try:
        freeze(arguments.draft, arguments.output)
    except StudyExecutionError as error:
        print(f"freeze blocked: {error}")
        return 2
    print(f"frozen manifest: {arguments.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
