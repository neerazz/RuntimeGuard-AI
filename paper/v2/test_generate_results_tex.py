#!/usr/bin/env python3

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("generate_results_tex.py")
SPEC = importlib.util.spec_from_file_location("generate_results_tex", MODULE_PATH)
assert SPEC and SPEC.loader
generator = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(generator)


class ResultsTexTests(unittest.TestCase):
    def write_run(self, root: Path, canonical: bool) -> Path:
        run = root / "run"
        (run / "summary").mkdir(parents=True)
        (run / "manifest.json").write_text(json.dumps({
            "canonical": canonical,
            "status": "complete",
            "run_id": "canonical-test",
            "source_digest_sha256": "ab" * 32,
            "host": {"python": "3.12.13", "hardware": {"chip": "Test Chip"}},
        }))
        modes = {}
        for key, p50, p99, throughput in [
            ("policy_only", 1.25, 4.5, 1000000.0),
            ("evidence:none", 200.0, 400.0, 18000.0),
            ("evidence:data", 15000.0, 27000.0, 265.0),
            ("evidence:full", 16000.0, 28000.0, 260.0),
        ]:
            modes[key] = {
                "p50_us_median": p50,
                "p99_us_median": p99,
                "throughput_rps_median": throughput,
                "repetitions": 20,
            }
        (run / "summary" / "key_findings.json").write_text(json.dumps({
            "canonical": canonical,
            "inline_condition": {"threads": 4, "prompt_bytes": 2048, "modes": modes},
            "largest_epoch": {
                "record_count": 100000,
                "seal_epoch_us_median": 100000.0,
                "verify_proof_us_median": 10.0,
                "verify_signature_us_median": 30.0,
            },
            "largest_recovery": {
                "record_count": 100000,
                "open_ms_median": 500.0,
                "read_sort_ms_median": 100.0,
            },
        }))
        return run

    def test_rejects_noncanonical_run(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = self.write_run(Path(temporary), canonical=False)
            with self.assertRaisesRegex(ValueError, "canonical"):
                generator.render_results(run)

    def test_emits_source_bound_macros_from_canonical_findings(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = self.write_run(Path(temporary), canonical=True)
            rendered = generator.render_results(run)
            self.assertIn(r"\def\RGRunID{canonical-test}", rendered)
            self.assertIn(r"\def\RGPolicyMedian{1.25}", rendered)
            self.assertIn(r"\def\RGDataThroughput{265}", rendered)
            self.assertIn(r"\def\RGEpochRecords{100,000}", rendered)
            self.assertIn("abababababab", rendered)


if __name__ == "__main__":
    unittest.main()
