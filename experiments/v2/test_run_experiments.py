#!/usr/bin/env python3

import hashlib
import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("run_experiments.py")
SPEC = importlib.util.spec_from_file_location("run_experiments", MODULE_PATH)
assert SPEC and SPEC.loader
run_experiments = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(run_experiments)


class FailureReceiptTests(unittest.TestCase):
    def test_git_dirty_status_is_scoped_to_measured_source_files(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
            subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=repo, check=True)
            subprocess.run(["git", "config", "user.name", "Test"], cwd=repo, check=True)
            source = repo / "source.py"
            source.write_text("version one\n")
            subprocess.run(["git", "add", "source.py"], cwd=repo, check=True)
            subprocess.run(["git", "commit", "-qm", "fixture"], cwd=repo, check=True)
            entries = [{"path": "source.py", "sha256": "unused"}]
            self.assertFalse(run_experiments.git_source_dirty(entries, repo))

            (repo / "unrelated.txt").write_text("not measured\n")
            self.assertFalse(run_experiments.git_source_dirty(entries, repo))

            source.write_text("version two\n")
            self.assertTrue(run_experiments.git_source_dirty(entries, repo))

    def test_source_closure_rejects_any_in_scope_drift(self) -> None:
        expected_files = [{"path": "src/lib.rs", "sha256": "a" * 64}]
        changed_files = [{"path": "src/lib.rs", "sha256": "b" * 64}]
        with self.assertRaisesRegex(RuntimeError, "source drift"):
            run_experiments.verify_source_closure(
                expected_files,
                "c" * 64,
                changed_files,
                "d" * 64,
            )

    def test_public_log_redaction_removes_repository_and_home_paths(self) -> None:
        raw = f"built {run_experiments.REPO}/target; cache {Path.home()}/.cargo/advisory-db"
        redacted = run_experiments.redact_public_log(raw)
        self.assertNotIn(str(run_experiments.REPO), redacted)
        self.assertNotIn(str(Path.home()), redacted)
        self.assertIn("<REPO>/target", redacted)
        self.assertIn("<HOME>/.cargo/advisory-db", redacted)

    def test_environment_validation_rejects_old_python_and_dependency_drift(self) -> None:
        expected = {"numpy": "2.4.2", "pandas": "3.0.0", "matplotlib": "3.10.8"}
        with self.assertRaisesRegex(RuntimeError, "Python 3.11 or newer"):
            run_experiments.validate_environment((3, 9), expected)
        with self.assertRaisesRegex(RuntimeError, "numpy==2.4.2"):
            run_experiments.validate_environment(
                (3, 12),
                {"numpy": "2.0.2", "pandas": "3.0.0", "matplotlib": "3.10.8"},
            )

    def test_record_failure_persists_exception_type_message_and_traceback_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run_dir = Path(temporary)
            manifest = {"status": "running"}
            try:
                raise RuntimeError("benchmark condition failed")
            except RuntimeError as error:
                run_experiments.record_failure(manifest, run_dir, error)

            failure_log = run_dir / "failure.log"
            self.assertEqual(manifest["status"], "failed")
            self.assertEqual(manifest["failure"]["type"], "RuntimeError")
            self.assertEqual(manifest["failure"]["message"], "benchmark condition failed")
            self.assertTrue(failure_log.exists())
            self.assertIn("RuntimeError: benchmark condition failed", failure_log.read_text())
            self.assertEqual(
                manifest["failure"]["log_sha256"],
                hashlib.sha256(failure_log.read_bytes()).hexdigest(),
            )


if __name__ == "__main__":
    unittest.main()
