#!/usr/bin/env python3
"""Fail closed when a RuntimeGuard public candidate contains private or stale artifacts."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

REQUIRED = {
    "README.md",
    "LICENSE",
    "CITATION.cff",
    "Cargo.toml",
    "docs/protocol-v2.md",
    "paper/v2/runtimeguard-v2.tex",
    "paper/v2/runtimeguard-v2.pdf",
    "paper/v2/generate_diagrams.py",
}
BLOCKED_PREFIXES = (
    ".git/",
    ".venv/",
    "paper/manuscript-os/",
    "paper/research/",
    "paper/v2/build/",
)
BLOCKED_SUFFIXES = {
    ".aux", ".blg", ".docx", ".dll", ".exe", ".log", ".pdb",
    ".pyc", ".rlib", ".rmeta", ".tmp", ".zip",
}
TEXT_SUFFIXES = {
    "", ".bib", ".cff", ".csv", ".json", ".md", ".py", ".rs",
    ".sh", ".tex", ".toml", ".txt", ".yaml", ".yml",
}
SCAN_EXEMPT = {
    "tools/audit_public_tree.py",
    "tools/test_audit_public_tree.py",
}
PLACEHOLDER_PATTERN = re.compile(
    r"\bTODO\b|\bTBD\b|draft pending|still executing|intermediate source|"
    r"under development|removed in the release candidate|ManuscriptOS|subagent|agent chain",
    re.IGNORECASE,
)
LOCAL_PATH_PATTERN = re.compile(r"/(?:Users|home)/[^\s'\"`]+")
AFFILIATION_PATTERN = re.compile(
    r"Parafin|Meta Reality Labs|Wayfair|JPMorgan|neeraj@parafin",
    re.IGNORECASE,
)
CANONICAL_PREFLIGHT_PATTERN = re.compile(
    r"^results/v2/runtimeguard-v2-canonical-[^/]+/preflight\.log$"
)


def audit(root: Path) -> list[str]:
    errors: list[str] = []
    files = [path for path in root.rglob("*") if path.is_file()]
    relative_files = {path.relative_to(root).as_posix() for path in files}

    for required in sorted(REQUIRED - relative_files):
        errors.append(f"missing required public artifact: {required}")

    for path in files:
        relative = path.relative_to(root).as_posix()
        if (
            any(relative.startswith(prefix) for prefix in BLOCKED_PREFIXES)
            or "/target/" in f"/{relative}/"
            or "/__pycache__/" in f"/{relative}/"
            or (relative.startswith("results/") and "quick" in relative.lower())
        ):
            errors.append(f"blocked path in public candidate: {relative}")
        if (
            path.suffix.lower() in BLOCKED_SUFFIXES
            and not CANONICAL_PREFLIGHT_PATTERN.fullmatch(relative)
        ):
            errors.append(f"blocked file type in public candidate: {relative}")
        if path.stat().st_size >= 100_000_000:
            errors.append(f"file exceeds 100 MB GitHub limit: {relative}")

        if relative in SCAN_EXEMPT or path.suffix.lower() not in TEXT_SUFFIXES:
            continue
        try:
            text = path.read_text()
        except UnicodeDecodeError:
            continue
        if LOCAL_PATH_PATTERN.search(text):
            errors.append(f"absolute local path in {relative}")
        if AFFILIATION_PATTERN.search(text):
            errors.append(f"employer or workplace affiliation in {relative}")
        if PLACEHOLDER_PATTERN.search(text):
            errors.append(f"release placeholder or internal workflow language in {relative}")

    figure_dir = root / "paper" / "v2" / "figures"
    if figure_dir.exists():
        for png in figure_dir.glob("*.png"):
            if not png.with_suffix(".pdf").exists():
                errors.append(
                    f"missing vector companion for paper figure: {png.relative_to(root).as_posix()}"
                )
    return sorted(set(errors))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path, nargs="?", default=Path.cwd())
    args = parser.parse_args()
    errors = audit(args.root.resolve())
    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        return 1
    print("public-tree audit: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
