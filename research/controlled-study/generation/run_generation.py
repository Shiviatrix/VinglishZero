#!/usr/bin/env python3
"""Run one real external generation session with immutable prompts and provenance."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
import uuid
from pathlib import Path
from typing import Any

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from generation.common import canonical_json, environment_manifest, read_json, sha256_bytes, sha256_file, utc_now, write_atomic
from study.errors import StudyExecutionError


def _render(argv: list[str], values: dict[str, str]) -> list[str]:
    try:
        return [part.format_map(values) for part in argv]
    except KeyError as error:
        raise StudyExecutionError(f"generator provider_command references unknown placeholder: {error.args[0]}") from error


def _output(value: str | bytes | None) -> str:
    """Normalize timeout output before it enters the canonical provenance record."""

    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return value


def _bundle(path: Path, arm: str) -> tuple[dict[str, Any], dict[str, Any]]:
    bundle = read_json(path / "prompt-bundle.json")
    digest = bundle.get("sha256")
    without_digest = {key: value for key, value in bundle.items() if key != "sha256"}
    if digest != sha256_bytes(canonical_json(without_digest)):
        raise StudyExecutionError("prompt bundle digest mismatch")
    instructions = bundle.get("arm_instructions")
    if not isinstance(instructions, dict) or arm not in instructions:
        raise StudyExecutionError(f"prompt bundle has no frozen instruction for arm {arm!r}")
    configuration = read_json(path / "generator-configuration.json")
    if configuration.get("prompt_bundle_sha256") != sha256_file(path / "prompt-bundle.json"):
        raise StudyExecutionError("generator configuration does not match the frozen prompt bundle")
    return bundle, configuration


def generate(bundle_root: Path, run_dir: Path, artifact_dir: Path, benchmark: str, level: str, arm: str) -> dict[str, Any]:
    bundle, config = _bundle(bundle_root, arm)
    session_id = f"session-{uuid.uuid4()}"
    session = run_dir / "generation-session" / session_id
    session.mkdir(parents=True, exist_ok=False)
    request = {
        "session_id": session_id, "created_at_utc": utc_now(), "benchmark": benchmark, "level": level, "arm": arm,
        "system_prompt": bundle["prompts"]["system_prompt"], "user_prompt": bundle["prompts"]["user_prompt"],
        "benchmark_instructions": bundle["prompts"]["benchmark_instructions"],
        "arm_instructions": bundle["arm_instructions"][arm], "model": {
            key: config[key] for key in (
                "model_id", "model_snapshot", "temperature", "top_p", "seed_policy",
                "max_tokens", "tool_configuration", "generation_budget",
            )
        },
        "prompt_bundle_sha256": bundle["sha256"],
        "benchmark_specification_sha256": sha256_file(bundle_root / "benchmark-specification.txt"),
        "generator_configuration_sha256": sha256_file(bundle_root / "generator-configuration.json"),
    }
    request_path, response_path = session / "request.json", session / "response.json"
    write_atomic(request_path, canonical_json(request))
    attempts = config["retry_policy"]["max_attempts"]
    records: list[dict[str, object]] = []
    started = time.monotonic()
    for attempt in range(1, attempts + 1):
        values = {
            "request_path": str(request_path), "response_path": str(response_path), "artifact_dir": str(artifact_dir),
            "session_dir": str(session), "benchmark": benchmark, "level": level, "arm": arm, "session_id": session_id,
        }
        argv = _render(config["provider_command"], values)
        command_started = time.monotonic()
        try:
            completed = subprocess.run(argv, capture_output=True, text=True, check=False, timeout=config["timeout_seconds"])
            exit_code, stdout, stderr = completed.returncode, completed.stdout, completed.stderr
        except subprocess.TimeoutExpired as error:
            exit_code, stdout, stderr = 124, _output(error.stdout), _output(error.stderr)
        except OSError as error:
            exit_code, stdout, stderr = 127, "", str(error)
        records.append({"attempt": attempt, "argv": argv, "exit_code": exit_code, "duration_seconds": time.monotonic() - command_started, "stdout": stdout, "stderr": stderr})
        write_atomic(session / f"attempt-{attempt}.json", canonical_json(records[-1]))
        if exit_code == 0 and response_path.is_file():
            response = read_json(response_path)
            required = ("generation_credits", "token_usage", "tool_usage")
            missing = [field for field in required if field not in response]
            if missing:
                raise StudyExecutionError(f"generator response is missing required fields: {', '.join(missing)}")
            metadata = {
                "session_id": session_id, "status": "SUCCESS", "started_at_utc": request["created_at_utc"], "finished_at_utc": utc_now(),
                "generation_duration_seconds": time.monotonic() - started, "retry_count": attempt - 1,
                "request_sha256": sha256_file(request_path), "response_sha256": sha256_file(response_path),
                "prompt_bundle_sha256": bundle["sha256"], "configuration_sha256": sha256_file(bundle_root / "generator-configuration.json"),
                "benchmark_specification_sha256": request["benchmark_specification_sha256"],
                "environment": environment_manifest(bundle_root), "attempts": records,
            }
            write_atomic(session / "metadata.json", canonical_json(metadata))
            return {"generation_credits": response["generation_credits"], "retry_count": attempt - 1, "token_usage": response["token_usage"], "tool_usage": response["tool_usage"], "session_id": session_id}
    metadata = {"session_id": session_id, "status": "FAILED", "finished_at_utc": utc_now(), "generation_duration_seconds": time.monotonic() - started, "attempts": records}
    write_atomic(session / "metadata.json", canonical_json(metadata))
    raise StudyExecutionError(f"generator failed after {attempts} attempt(s); session: {session}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, required=True)
    parser.add_argument("--run-dir", type=Path, required=True)
    parser.add_argument("--artifact-dir", type=Path, required=True)
    parser.add_argument("--benchmark", required=True)
    parser.add_argument("--level", required=True)
    parser.add_argument("--arm", required=True)
    arguments = parser.parse_args()
    try:
        result = generate(arguments.bundle.resolve(), arguments.run_dir.resolve(), arguments.artifact_dir.resolve(), arguments.benchmark, arguments.level, arguments.arm)
    except StudyExecutionError as error:
        print(f"generation failed: {error}", file=sys.stderr)
        return 2
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
