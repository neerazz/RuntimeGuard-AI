#!/usr/bin/env python3

import importlib.util
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("audit_public_tree.py")
SPEC = importlib.util.spec_from_file_location("audit_public_tree", MODULE_PATH)
assert SPEC and SPEC.loader
auditor = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(auditor)


class PublicTreeAuditTests(unittest.TestCase):
    def test_clean_minimal_candidate_passes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for relative in [
                "README.md", "LICENSE", "CITATION.cff", "Cargo.toml",
                "docs/protocol-v2.md", "paper/v2/runtimeguard-v2.tex",
                "paper/v2/runtimeguard-v2.pdf", "paper/v2/generate_diagrams.py",
                "paper/v2/figures/protocol.pdf", "paper/v2/figures/protocol.png",
                "results/v2/runtimeguard-v2-canonical-test/preflight.log",
            ]:
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("independent research artifact\n")
            self.assertEqual(auditor.audit(root), [])

    def test_rejects_private_paths_placeholders_and_unreproducible_figures(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "paper/manuscript-os").mkdir(parents=True)
            (root / "paper/manuscript-os/state.json").write_text("{}")
            (root / "README.md").write_text("TODO /Users/example private draft\n")
            (root / "debug.log").write_text("debug\n")
            (root / "paper/v2/figures").mkdir(parents=True)
            (root / "paper/v2/figures/orphan.png").write_bytes(b"png")
            errors = "\n".join(auditor.audit(root))
            self.assertIn("blocked path", errors)
            self.assertIn("absolute local path", errors)
            self.assertIn("release placeholder", errors)
            self.assertIn("missing vector companion", errors)
            self.assertIn("blocked file type", errors)


if __name__ == "__main__":
    unittest.main()
