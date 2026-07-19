#!/usr/bin/env python3
"""Prepare and freeze reproducible inputs for one controlled generation study."""

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path
from typing import Any

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from freeze_manifest import freeze
from generation.common import (
    canonical_json, command_output, environment_manifest, executable, read_json,
    require_file, sha256_bytes, sha256_file, write_atomic,
)
from study.errors import StudyExecutionError
from study.manifest import REQUIRED_COMMANDS, load_manifest


PROMPT_FIELDS = ("system_prompt", "user_prompt", "benchmark_instructions", "evaluation_instructions")
REQUIRED_DEFINITION = (
    "study_id", "benchmark", "level", "arms", "replications_per_cell", "benchmark_spec",
    *PROMPT_FIELDS, "generator", "evaluation",
)


def _value(definition: dict[str, Any], name: str) -> Any:
    value = definition.get(name)
    if value in (None, "", []):
        raise StudyExecutionError(f"study definition is missing required field: {name}")
    return value


def _copy(source: Path, destination: Path) -> str:
    try:
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    except OSError as error:
        raise StudyExecutionError(f"cannot copy frozen input {source} to {destination}: {error}") from error
    return sha256_file(destination)


def _source(workspace: Path, value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else workspace / path


def _prompt_bundle(definition: dict[str, Any], root: Path, workspace: Path) -> Path:
    prompts: dict[str, str] = {}
    source_hashes: dict[str, str] = {}
    for field in PROMPT_FIELDS:
        source = require_file(_source(workspace, str(_value(definition, field))), field)
        destination = root / "prompts" / f"{field}.txt"
        source_hashes[field] = _copy(source, destination)
        prompts[field] = destination.read_text(encoding="utf-8")
    arms = _value(definition, "arm_instructions")
    if not isinstance(arms, dict) or not arms:
        raise StudyExecutionError("study definition arm_instructions must map each arm to an instruction file")
    arm_contents: dict[str, str] = {}
    for arm, source_name in sorted(arms.items()):
        if not isinstance(arm, str) or not isinstance(source_name, str):
            raise StudyExecutionError("arm_instructions entries must be string arm/file pairs")
        source = require_file(_source(workspace, source_name), f"arm instruction for {arm}")
        destination = root / "prompts" / "arms" / f"{arm}.txt"
        source_hashes[f"arm:{arm}"] = _copy(source, destination)
        arm_contents[arm] = destination.read_text(encoding="utf-8")
    bundle: dict[str, object] = {
        "format": "vz-controlled-prompt-bundle-v1",
        "prompts": prompts,
        "arm_instructions": arm_contents,
        "source_hashes": source_hashes,
        "metadata": {
            "benchmark": definition["benchmark"], "level": definition["level"], "arms": definition["arms"],
        },
    }
    bundle["sha256"] = sha256_bytes(canonical_json(bundle))
    path = root / "prompt-bundle.json"
    write_atomic(path, canonical_json(bundle))
    return path


def _commands(definition: dict[str, Any], workspace: Path) -> dict[str, dict[str, object]]:
    evaluation = _value(definition, "evaluation")
    if not isinstance(evaluation, dict) or not isinstance(evaluation.get("commands"), dict):
        raise StudyExecutionError("study definition evaluation.commands must be an object")
    commands: dict[str, dict[str, object]] = {}
    for name in REQUIRED_COMMANDS:
        if name == "generate":
            continue
        entry = evaluation["commands"].get(name)
        if not isinstance(entry, dict):
            raise StudyExecutionError(f"study definition evaluation.commands.{name} must be an object")
        argv = executable(entry.get("argv"), f"evaluation command {name}")
        timeout = entry.get("timeout_seconds")
        if not isinstance(timeout, int) or timeout <= 0:
            raise StudyExecutionError(f"evaluation command {name}.timeout_seconds must be a positive integer")
        commands[name] = {"argv": argv, "timeout_seconds": timeout}
    preflight = evaluation.get("preflight_commands", [])
    if not isinstance(preflight, list):
        raise StudyExecutionError("study definition evaluation.preflight_commands must be an array")
    for index, entry in enumerate(preflight):
        if not isinstance(entry, dict):
            raise StudyExecutionError(f"preflight command {index} must be an object")
        argv = executable(entry.get("argv"), f"preflight command {index}")
        timeout = entry.get("timeout_seconds", 30)
        if not isinstance(timeout, int) or timeout <= 0:
            raise StudyExecutionError(f"preflight command {index}.timeout_seconds must be a positive integer")
        command_output(argv, workspace, f"preflight command {index}", timeout)
    return commands


def prepare(definition_path: Path, output: Path, freeze_output: Path | None) -> Path:
    """Copy supplied immutable inputs, validate commands, and emit a freeze-ready draft."""

    definition = read_json(definition_path)
    for field in REQUIRED_DEFINITION:
        _value(definition, field)
    arms = definition["arms"]
    if not isinstance(arms, list) or not all(isinstance(arm, str) and arm for arm in arms):
        raise StudyExecutionError("study definition arms must be a non-empty string array")
    if sorted(arms) != sorted(_value(definition, "arm_instructions").keys()):
        raise StudyExecutionError("study definition arm_instructions must contain exactly the declared arms")
    if not isinstance(definition["replications_per_cell"], int) or definition["replications_per_cell"] <= 0:
        raise StudyExecutionError("study definition replications_per_cell must be a positive integer")
    workspace = definition_path.parent.resolve()
    output = output.resolve()
    if freeze_output is not None and freeze_output.resolve().parent != output:
        raise StudyExecutionError(
            "freeze_output must be inside the bundle directory so its hashed artifacts remain portable"
        )
    if output.exists() and any(output.iterdir()):
        raise StudyExecutionError(f"bundle output must be empty to preserve reproducibility: {output}")
    output.mkdir(parents=True, exist_ok=True)
    spec = require_file(_source(workspace, str(definition["benchmark_spec"])), "benchmark_spec")
    _copy(spec, output / "benchmark-specification.txt")
    prompt_bundle = _prompt_bundle(definition, output, workspace)
    generator = definition["generator"]
    if not isinstance(generator, dict):
        raise StudyExecutionError("study definition generator must be an object")
    generator_required = ("model_id", "model_snapshot", "temperature", "top_p", "seed_policy", "max_tokens", "tool_configuration", "retry_policy", "timeout_seconds", "generation_budget", "provider_command")
    missing = [field for field in generator_required if generator.get(field) in (None, "", [])]
    if missing:
        raise StudyExecutionError(f"generator configuration is missing required fields: {', '.join(missing)}")
    provider_command = executable(generator["provider_command"], "generator provider_command")
    if not isinstance(generator["timeout_seconds"], int) or generator["timeout_seconds"] <= 0:
        raise StudyExecutionError("generator timeout_seconds must be a positive integer")
    if not isinstance(generator["retry_policy"], dict) or not isinstance(generator["retry_policy"].get("max_attempts"), int) or generator["retry_policy"]["max_attempts"] <= 0:
        raise StudyExecutionError("generator retry_policy.max_attempts must be a positive integer")
    generator = {**generator, "provider_command": provider_command, "prompt_bundle_sha256": sha256_file(prompt_bundle)}
    write_atomic(output / "generator-configuration.json", canonical_json(generator))
    environment = environment_manifest(workspace)
    write_atomic(output / "environment-manifest.json", canonical_json(environment))
    commands = _commands(definition, workspace)
    wrapper = Path(__file__).with_name("run_generation.py").resolve()
    commands["generate"] = {
        "argv": [sys.executable, str(wrapper), "--bundle", str(output), "--run-dir", "{run_dir}", "--artifact-dir", "{artifact_dir}", "--benchmark", "{benchmark}", "--level", "{level}", "--arm", "{arm}"],
        "timeout_seconds": generator["timeout_seconds"],
    }
    evaluation_config = {"commands": commands, "preflight_commands": definition["evaluation"].get("preflight_commands", [])}
    write_atomic(output / "evaluation-configuration.json", canonical_json(evaluation_config))
    hardware_id = sha256_file(output / "environment-manifest.json")
    draft = {
        "study_id": definition["study_id"], "status": "draft", "benchmarks": [definition["benchmark"]],
        "levels": [definition["level"]], "arms": arms, "replications_per_cell": definition["replications_per_cell"],
        "required_artifacts": {"bindings": {
            "task_specification_bundle": "benchmark-specification.txt", "system_prompt": "prompt-bundle.json",
            "evaluation_prompt": "prompts/evaluation_instructions.txt", "model_snapshot": "generator-configuration.json",
            "generation_settings": "generator-configuration.json", "hardware_manifest": "environment-manifest.json",
            "evaluator_revision": "evaluation-configuration.json",
        }},
        "provenance": {
            "model_id": str(generator["model_id"]), "model_snapshot": str(generator["model_snapshot"]),
            "temperature": str(generator["temperature"]), "top_p": str(generator["top_p"]),
            "seed_policy": str(generator["seed_policy"]), "tool_budget": str(generator["tool_configuration"]),
            "git_commit": str(environment["git"]["commit"]), "hardware_id": hardware_id,
        },
        "commands": commands,
    }
    draft_path = output / "study-draft.json"
    write_atomic(draft_path, canonical_json(draft))
    if freeze_output is not None:
        freeze(draft_path, freeze_output)
        load_manifest(freeze_output)
    return draft_path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--definition", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--freeze-output", type=Path, help="optional ready-manifest destination")
    arguments = parser.parse_args()
    try:
        draft = prepare(arguments.definition, arguments.output, arguments.freeze_output)
    except StudyExecutionError as error:
        print(f"generation bundle blocked: {error}", file=sys.stderr)
        return 2
    print(f"generation bundle prepared: {draft}")
    if arguments.freeze_output is not None:
        print(f"frozen manifest: {arguments.freeze_output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
