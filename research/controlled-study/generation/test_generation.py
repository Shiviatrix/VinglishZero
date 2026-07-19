"""Integration tests for frozen generation inputs and session provenance."""

from __future__ import annotations

import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

from generation.common import sha256_file
from generation.prepare_bundle import prepare
from generation.run_generation import generate
from freeze_manifest import freeze
from study.errors import StudyExecutionError
from study.manifest import load_manifest


class GenerationHarnessTests(unittest.TestCase):
    def definition(self, root: Path) -> Path:
        inputs = root / "inputs"
        inputs.mkdir()
        for name, content in {
            "spec.txt": "Implement the frozen sample benchmark.\n",
            "system.txt": "You are a deterministic implementation generator.\n",
            "user.txt": "Implement the requested benchmark.\n",
            "benchmark.txt": "Return the collection length.\n",
            "evaluation.txt": "Run every frozen evaluator.\n",
            "baseline.txt": "Use no semantic reasoning tools.\n",
            "zero.txt": "Use Vinglish Zero only as declared.\n",
        }.items():
            (inputs / name).write_text(content, encoding="utf-8")
        provider = root / "provider.py"
        provider.write_text(
            """import json
import pathlib
import sys
request, response, artifact = map(pathlib.Path, sys.argv[1:])
artifact.mkdir(parents=True, exist_ok=True)
(artifact / 'implementation.py').write_text('def answer(items):\\n    return len(items)\\n', encoding='utf-8')
response.write_text(json.dumps({'generation_credits': 1, 'token_usage': 12, 'tool_usage': []}), encoding='utf-8')
""",
            encoding="utf-8",
        )
        commands = {
            name: {"argv": [sys.executable, "-c", "pass"], "timeout_seconds": 10}
            for name in ("compile", "maintained_tests", "independent_tests", "randomized_tests", "stress_tests", "feature_evaluation", "security_evaluation", "performance_evaluation")
        }
        payload = {
            "study_id": "generation-harness-test", "benchmark": "json_parser", "level": "L3",
            "arms": ["baseline", "vinglish_zero"], "replications_per_cell": 1,
            "benchmark_spec": "inputs/spec.txt", "system_prompt": "inputs/system.txt", "user_prompt": "inputs/user.txt",
            "benchmark_instructions": "inputs/benchmark.txt", "evaluation_instructions": "inputs/evaluation.txt",
            "arm_instructions": {"baseline": "inputs/baseline.txt", "vinglish_zero": "inputs/zero.txt"},
            "generator": {
                "model_id": "test-model", "model_snapshot": "test-model-2026-07-19", "temperature": 0,
                "top_p": 1, "seed_policy": "provider-recorded", "max_tokens": 1000, "tool_configuration": {"tools": []},
                "retry_policy": {"max_attempts": 1}, "timeout_seconds": 30, "generation_budget": 1,
                "provider_command": [sys.executable, str(provider), "{request_path}", "{response_path}", "{artifact_dir}"],
            },
            "evaluation": {"commands": commands, "preflight_commands": [{"argv": [sys.executable, "-c", "pass"], "timeout_seconds": 10}]},
        }
        path = root / "study-definition.json"
        path.write_text(json.dumps(payload), encoding="utf-8")
        return path

    def test_prepared_bundle_freezes_to_a_valid_manifest_and_records_a_session(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            definition = self.definition(root)
            bundle, manifest = root / "bundle", root / "bundle" / "study-ready.json"
            prepare(definition, bundle, manifest)
            frozen = load_manifest(manifest)
            self.assertEqual(frozen.study_id, "generation-harness-test")
            self.assertEqual(sha256_file(bundle / "prompt-bundle.json"), json.loads((bundle / "generator-configuration.json").read_text())["prompt_bundle_sha256"])
            result = generate(bundle, root / "run", root / "run" / "artifact", "json_parser", "L3", "baseline")
            self.assertEqual(result["generation_credits"], 1)
            sessions = list((root / "run" / "generation-session").glob("*/metadata.json"))
            self.assertEqual(len(sessions), 1)
            self.assertEqual(json.loads(sessions[0].read_text())["status"], "SUCCESS")

    def test_identical_inputs_recreate_identical_frozen_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            definition = self.definition(root)
            bundle, manifest = root / "bundle", root / "bundle" / "study-ready.json"
            prepare(definition, bundle, manifest)
            first = manifest.read_bytes()
            shutil.rmtree(bundle)
            prepare(definition, bundle, manifest)
            self.assertEqual(first, manifest.read_bytes())

    def test_changed_prompt_changes_manifest_digest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            definition = self.definition(root)
            first_bundle, first_manifest = root / "one", root / "one" / "one.json"
            prepare(definition, first_bundle, first_manifest)
            (root / "inputs" / "user.txt").write_text("Changed prompt.\n", encoding="utf-8")
            second_bundle, second_manifest = root / "two", root / "two" / "two.json"
            prepare(definition, second_bundle, second_manifest)
            self.assertNotEqual(sha256_file(first_manifest), sha256_file(second_manifest))

    def test_missing_required_inputs_refuse_preparation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            definition = self.definition(root)
            payload = json.loads(definition.read_text())
            del payload["system_prompt"]
            definition.write_text(json.dumps(payload), encoding="utf-8")
            with self.assertRaisesRegex(StudyExecutionError, "system_prompt"):
                prepare(definition, root / "bundle", None)

    def test_missing_benchmark_specification_refuses_preparation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            definition = self.definition(root)
            (root / "inputs" / "spec.txt").unlink()
            with self.assertRaisesRegex(StudyExecutionError, "benchmark_spec"):
                prepare(definition, root / "bundle", None)

    def test_missing_provenance_refuses_freezing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            definition = self.definition(root)
            bundle = root / "bundle"
            draft = prepare(definition, bundle, None)
            payload = json.loads(draft.read_text(encoding="utf-8"))
            del payload["provenance"]["model_id"]
            draft.write_text(json.dumps(payload), encoding="utf-8")
            with self.assertRaisesRegex(StudyExecutionError, "missing execution provenance"):
                freeze(draft, bundle / "ready.json")

    def test_missing_provider_response_fields_refuses_generation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            definition = self.definition(root)
            bundle = root / "bundle"
            prepare(definition, bundle, bundle / "ready.json")
            provider = root / "bad_provider.py"
            provider.write_text("import pathlib, sys; pathlib.Path(sys.argv[2]).write_text('{}')\n", encoding="utf-8")
            config = json.loads((bundle / "generator-configuration.json").read_text())
            config["provider_command"] = [sys.executable, str(provider), "{request_path}", "{response_path}", "{artifact_dir}"]
            (bundle / "generator-configuration.json").write_text(json.dumps(config), encoding="utf-8")
            with self.assertRaisesRegex(StudyExecutionError, "does not match the frozen prompt bundle|missing required fields"):
                generate(bundle, root / "run", root / "run" / "artifact", "json_parser", "L3", "baseline")


if __name__ == "__main__":
    unittest.main()
