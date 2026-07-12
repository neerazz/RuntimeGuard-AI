#!/usr/bin/env python3

import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("analyze.py")
SPEC = importlib.util.spec_from_file_location("analyze", MODULE_PATH)
assert SPEC and SPEC.loader
analyze = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(analyze)


class AnalysisSourceBindingTests(unittest.TestCase):
    def test_analysis_rejects_source_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "analyze.py"
            source.write_text("version one\n")
            entry = {
                "path": "analyze.py",
                "sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            }
            encoded = json.dumps([entry], separators=(",", ":"), sort_keys=True).encode()
            manifest = {
                "source_files": [entry],
                "source_digest_sha256": hashlib.sha256(encoded).hexdigest(),
            }
            analyze.validate_source_snapshot(manifest, root)

            source.write_text("version two\n")
            with self.assertRaisesRegex(ValueError, "source drift"):
                analyze.validate_source_snapshot(manifest, root)


if __name__ == "__main__":
    unittest.main()
