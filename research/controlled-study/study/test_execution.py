"""Integration tests for immutable scheduling, command execution, and checkpoints."""

from __future__ import annotations

import hashlib
import json
import multiprocessing
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from study.errors import StudyExecutionError
from study.executor import StudyExecutor
from study.locking import FileLock
from study.manifest import load_manifest
from study.scheduler import schedule
from study.storage import StudyStorage


STUDY_ROOT = Path(__file__).parents[1]


def sha(value: str) -> str:
    return "sha256:" + hashlib.sha256(value.encode("utf-8")).hexdigest()


def crash_while_holding_lock(path: str, ready: multiprocessing.Event) -> None:
    with FileLock(Path(path)):
        ready.set()
        os._exit(0)


class StudyExecutionTests(unittest.TestCase):
    def ready_manifest(self, root: Path) -> Path:
        frozen = root / "frozen.txt"
        frozen.write_text("immutable input\n", encoding="utf-8")
        frontend = root / "frontend.py"
        frontend.write_text(
            """import json
import pathlib
import sys

stage, artifact = sys.argv[1:]
target = pathlib.Path(artifact)
if stage == 'generate':
    target.mkdir(parents=True, exist_ok=True)
    (target / 'implementation.py').write_text('def solution(items):\\n    return len(items)\\n', encoding='utf-8')
    with (target.parents[2] / 'generation-count.log').open('a', encoding='utf-8') as count:
        count.write('generated\\n')
    import time
    time.sleep(0.2)
if stage in {'maintained_tests', 'independent_tests', 'randomized_tests', 'stress_tests', 'feature_evaluation'}:
    print(json.dumps({'passed': 2, 'total': 2}))
elif stage == 'security_evaluation':
    print(json.dumps({'findings': 0}))
elif stage == 'performance_evaluation':
    print(json.dumps({'runtime_seconds': 0.01, 'memory_bytes': 512, 'startup_seconds': 0.001}))
elif stage == 'generate':
    print(json.dumps({'generation_credits': 3, 'retry_count': 0}))
""",
            encoding="utf-8",
        )
        stages = (
            "generate", "compile", "maintained_tests", "independent_tests", "randomized_tests", "stress_tests",
            "feature_evaluation", "security_evaluation", "performance_evaluation",
        )
        payload = {
            "study_id": "execution-test", "status": "ready", "benchmarks": ["json_parser"], "levels": ["L3"],
            "arms": ["baseline"], "replications_per_cell": 1,
            "required_artifacts": {
                "task_specification_bundle": "sha256:task", "system_prompt": "sha256:system",
                "evaluation_prompt": "sha256:evaluation", "model_snapshot": "test-model",
                "generation_settings": "sha256:settings", "hardware_manifest": "sha256:hardware",
                "evaluator_revision": "sha256:evaluator", "files": {"frozen.txt": sha("immutable input\n")},
            },
            "provenance": {"model_id": "test", "model_snapshot": "test-model", "temperature": "0", "top_p": "1", "seed_policy": "fixed", "tool_budget": "10", "git_commit": "test-commit", "hardware_id": "test-hardware"},
            "commands": {stage: {"argv": [sys.executable, str(frontend), stage, "{artifact_dir}"], "timeout_seconds": 10} for stage in stages},
        }
        manifest = root / "manifest.json"
        manifest.write_text(json.dumps(payload), encoding="utf-8")
        return manifest

    def test_execution_commits_a_complete_observation_and_is_resumable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = load_manifest(self.ready_manifest(root))
            run = schedule(manifest)[0]
            storage = StudyStorage(root / "output")
            observation = StudyExecutor(manifest, storage, root).run(run)
            self.assertTrue(storage.complete(run))
            self.assertEqual(observation["row"]["generation_credits"], 3)
            self.assertEqual(observation["row"]["loc"], 2)
            self.assertEqual(observation["row"]["independent_tests_passed"], 2)
            self.assertEqual(observation["row"]["terminal_state"], "SUCCESS")
            self.assertEqual(observation["row"]["architecture_score"], "NOT_AVAILABLE")
            self.assertTrue(all(value not in (None, "") for value in observation["row"].values()))
            self.assertTrue((storage.run_dir(run) / "logs" / "generate.stdout.log").is_file())
            self.assertEqual(len(storage.runs_path.read_text(encoding="utf-8").splitlines()), 2)
            recovered = StudyStorage(root / "output")
            self.assertTrue(recovered.complete(run))
            self.assertEqual(len(recovered.runs_path.read_text(encoding="utf-8").splitlines()), 2)

    def test_changed_frozen_input_is_rejected_before_execution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest_path = self.ready_manifest(root)
            (root / "frozen.txt").write_text("changed\n", encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "hash mismatch"):
                load_manifest(manifest_path)

    def test_freeze_utility_hash_binds_every_declared_input(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            ready = self.ready_manifest(root)
            payload = json.loads(ready.read_text(encoding="utf-8"))
            frozen = root / "frozen.txt"
            payload["status"] = "draft"
            payload["required_artifacts"] = {
                "bindings": {name: "frozen.txt" for name in (
                    "task_specification_bundle", "system_prompt", "evaluation_prompt", "model_snapshot",
                    "generation_settings", "hardware_manifest", "evaluator_revision",
                )},
            }
            draft = root / "draft.json"
            draft.write_text(json.dumps(payload), encoding="utf-8")
            output = root / "ready.json"
            completed = subprocess.run(
                [sys.executable, str(STUDY_ROOT / "freeze_manifest.py"), "--draft", str(draft), "--output", str(output)],
                capture_output=True, text=True, check=False,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            self.assertEqual(load_manifest(output).study_id, "execution-test")

    def test_stage_checkpoint_prevents_repeating_a_completed_generation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = load_manifest(self.ready_manifest(root))
            run = schedule(manifest)[0]
            storage = StudyStorage(root / "output")
            artifact = storage.run_dir(run) / "artifact"
            artifact.mkdir(parents=True)
            (artifact / "implementation.py").write_text("def solution(items):\n    return len(items)\n", encoding="utf-8")
            storage.write_attempt(run, {
                "started_at_utc": "2026-07-19T00:00:00Z",
                "commands": {"generate": {
                    "name": "generate", "argv": ["already-recorded"], "exit_code": 0, "duration_seconds": 0.01,
                    "stdout": '{"generation_credits": 3, "retry_count": 0}', "stderr": "", "timed_out": False,
                }},
            })
            observation = StudyExecutor(manifest, storage, root).run(run)
            self.assertEqual(observation["commands"][0]["argv"], ["already-recorded"])
            self.assertTrue(storage.complete(run))

    def test_runner_executes_ready_schedule_then_generates_all_report_formats(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest_path = self.ready_manifest(root)
            payload = json.loads(manifest_path.read_text(encoding="utf-8"))
            payload["arms"] = ["baseline", "vinglish_zero"]
            manifest_path.write_text(json.dumps(payload), encoding="utf-8")
            script = Path(__file__).parents[1] / "run_study.py"
            output = root / "output"
            completed = subprocess.run(
                [sys.executable, str(script), "--manifest", str(manifest_path), "--workspace", str(root), "--output", str(output)],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            for name in (
                "publication-report.md", "publication-report.html", "publication-report.pdf",
                "comparison-overview.svg", "distribution-overview.svg",
            ):
                self.assertTrue((output / "report" / name).is_file(), name)
            self.assertTrue((output / "analysis" / "comparisons.json").is_file())
            self.assertTrue((output / "analysis" / "advanced-analysis.json").is_file())

    def test_concurrent_runners_execute_each_generation_once(self) -> None:
        for runner_count in (2, 3):
            with self.subTest(runner_count=runner_count), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                manifest = self.ready_manifest(root)
                output = root / "output"
                command = [sys.executable, str(STUDY_ROOT / "run_study.py"), "--manifest", str(manifest), "--workspace", str(root), "--output", str(output)]
                runners = [subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True) for _ in range(runner_count)]
                outcomes = [runner.communicate(timeout=30) for runner in runners]
                self.assertTrue(all(runner.returncode == 0 for runner in runners), outcomes)
                self.assertEqual((output / "generation-count.log").read_text(encoding="utf-8").splitlines(), ["generated"])
                self.assertEqual(len((output / "runs.csv").read_text(encoding="utf-8").splitlines()), 2)

    def test_crash_releases_stale_os_lock(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            lock_path = Path(directory) / "stale.lock"
            ready = multiprocessing.Event()
            process = multiprocessing.Process(target=crash_while_holding_lock, args=(str(lock_path), ready))
            process.start()
            self.assertTrue(ready.wait(timeout=10))
            process.join(timeout=10)
            self.assertEqual(process.exitcode, 0)
            with FileLock(lock_path, timeout_seconds=1):
                pass

    def test_corrupt_csv_is_preserved_and_requires_explicit_repair(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = load_manifest(self.ready_manifest(root))
            run = schedule(manifest)[0]
            storage = StudyStorage(root / "output")
            StudyExecutor(manifest, storage, root).run(run)
            original = b"corrupt,csv\n"
            storage.runs_path.write_bytes(original)
            with self.assertRaisesRegex(StudyExecutionError, "integrity check failed"):
                StudyStorage(root / "output")
            self.assertEqual(storage.runs_path.read_bytes(), original)
            recovered = StudyStorage(root / "output", verify_exports=False)
            backups = recovered.repair_exports()
            self.assertEqual(len(backups), 1)
            self.assertEqual(backups[0].read_bytes(), original)
            StudyStorage(root / "output")

    def test_malformed_observation_has_a_structured_error(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            storage = StudyStorage(root / "output")
            path = storage.observations / "run-corrupt" / "observation.json"
            path.parent.mkdir(parents=True)
            path.write_text("{}", encoding="utf-8")
            with self.assertRaisesRegex(StudyExecutionError, r"\.row must be an object"):
                StudyStorage(root / "output")

    def test_valid_json_observation_tampering_is_detected_by_checksum(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = load_manifest(self.ready_manifest(root))
            run = schedule(manifest)[0]
            storage = StudyStorage(root / "output")
            StudyExecutor(manifest, storage, root).run(run)
            observation_path = storage.run_dir(run) / "observation.json"
            observation = json.loads(observation_path.read_text(encoding="utf-8"))
            observation["row"]["notes"] = "tampered"
            observation_path.write_text(json.dumps(observation), encoding="utf-8")
            with self.assertRaisesRegex(StudyExecutionError, "checksum mismatch"):
                StudyStorage(root / "output")

    def test_duplicate_schedule_entries_are_rejected_before_execution(self) -> None:
        for field, values in (("benchmarks", ["json_parser", "json_parser"]), ("levels", ["L3", "L3"]), ("arms", ["baseline", "baseline"])):
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                path = self.ready_manifest(root)
                payload = json.loads(path.read_text(encoding="utf-8"))
                payload[field] = values
                path.write_text(json.dumps(payload), encoding="utf-8")
                with self.assertRaisesRegex(StudyExecutionError, "duplicate entries"):
                    load_manifest(path)

    def test_valid_schedule_has_unique_cells_and_run_ids(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = self.ready_manifest(root)
            payload = json.loads(path.read_text(encoding="utf-8"))
            payload["arms"] = ["baseline", "vinglish_zero"]
            payload["replications_per_cell"] = 3
            path.write_text(json.dumps(payload), encoding="utf-8")
            runs = schedule(load_manifest(path))
            cells = {(run.benchmark, run.level, run.arm, run.replicate) for run in runs}
            self.assertEqual(len(cells), len(runs))
            self.assertEqual(len({run.generation_id for run in runs}), len(runs))

    def test_output_directory_rejects_a_changed_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = self.ready_manifest(root)
            first = load_manifest(path)
            storage = StudyStorage(root / "output")
            storage.bind_manifest(first.digest)
            payload = json.loads(path.read_text(encoding="utf-8"))
            payload["provenance"]["tool_budget"] = "11"
            path.write_text(json.dumps(payload), encoding="utf-8")
            second = load_manifest(path)
            with self.assertRaisesRegex(StudyExecutionError, "bound to a different manifest"):
                storage.bind_manifest(second.digest)

    def test_malformed_metric_output_commits_a_failed_terminal_observation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = self.ready_manifest(root)
            invalid = root / "invalid_metrics.py"
            invalid.write_text("print('{}')\n", encoding="utf-8")
            payload = json.loads(path.read_text(encoding="utf-8"))
            payload["commands"]["feature_evaluation"] = {"argv": [sys.executable, str(invalid)], "timeout_seconds": 10}
            path.write_text(json.dumps(payload), encoding="utf-8")
            manifest = load_manifest(path)
            run = schedule(manifest)[0]
            storage = StudyStorage(root / "output")
            observation = StudyExecutor(manifest, storage, root).run(run)
            self.assertEqual(observation["row"]["terminal_state"], "FAILED")
            self.assertNotEqual(observation["row"]["terminal_reason"], "")
            self.assertTrue(storage.complete(run))
            self.assertEqual(len(observation["failures"]), 1)

    def test_failed_compilation_commits_a_failed_terminal_observation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = self.ready_manifest(root)
            payload = json.loads(path.read_text(encoding="utf-8"))
            payload["commands"]["compile"] = {"argv": [sys.executable, "-c", "import sys; sys.exit(7)"], "timeout_seconds": 10}
            path.write_text(json.dumps(payload), encoding="utf-8")
            manifest = load_manifest(path)
            run = schedule(manifest)[0]
            observation = StudyExecutor(manifest, StudyStorage(root / "output"), root).run(run)
            self.assertEqual(observation["row"]["terminal_state"], "FAILED")
            self.assertEqual(observation["row"]["compilation_success"], "false")
            self.assertEqual(observation["failures"][0]["observed_by"], "compile")


if __name__ == "__main__":
    unittest.main()
