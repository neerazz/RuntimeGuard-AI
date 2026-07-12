#!/usr/bin/env python3
"""Generate source-bound LaTeX result macros from a canonical RuntimeGuard run."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

MODE_PREFIXES = {
    "policy_only": "Policy",
    "evidence:none": "Buffered",
    "evidence:data": "Data",
    "evidence:full": "Full",
}


def format_number(value: float | int) -> str:
    rendered = f"{float(value):,.3f}".rstrip("0").rstrip(".")
    return rendered


def macro(name: str, value: Any) -> str:
    return f"\\def\\RG{name}{{{value}}}"


def render_results(run_dir: Path) -> str:
    manifest = json.loads((run_dir / "manifest.json").read_text())
    findings = json.loads((run_dir / "summary" / "key_findings.json").read_text())
    if manifest.get("status") != "complete":
        raise ValueError(f"run is not complete: {manifest.get('status')}")
    if not manifest.get("canonical") or not findings.get("canonical"):
        raise ValueError("result macros require a canonical run")

    inline = findings["inline_condition"]
    lines = [
        "% Generated from canonical key_findings.json; do not edit manually.",
        macro("RunID", manifest["run_id"]),
        macro("SourceDigest", manifest["source_digest_sha256"]),
        macro("SourceDigestShort", manifest["source_digest_sha256"][:12]),
        macro("PythonVersion", manifest["host"]["python"]),
        macro("HostChip", manifest["host"]["hardware"]["chip"]),
        macro("Threads", inline["threads"]),
        macro("PromptBytes", format_number(inline["prompt_bytes"])),
    ]
    for mode, prefix in MODE_PREFIXES.items():
        values = inline["modes"][mode]
        lines.extend(
            [
                macro(f"{prefix}Median", format_number(values["p50_us_median"])),
                macro(f"{prefix}Tail", format_number(values["p99_us_median"])),
                macro(
                    f"{prefix}Throughput",
                    format_number(values["throughput_rps_median"]),
                ),
                macro(f"{prefix}Repetitions", values["repetitions"]),
            ]
        )

    epoch = findings["largest_epoch"]
    recovery = findings["largest_recovery"]
    lines.extend(
        [
            macro("EpochRecords", format_number(epoch["record_count"])),
            macro("EpochSealUS", format_number(epoch["seal_epoch_us_median"])),
            macro("EpochProofUS", format_number(epoch["verify_proof_us_median"])),
            macro("EpochSignatureUS", format_number(epoch["verify_signature_us_median"])),
            macro("RecoveryRecords", format_number(recovery["record_count"])),
            macro("RecoveryOpenMS", format_number(recovery["open_ms_median"])),
            macro("RecoveryReadSortMS", format_number(recovery["read_sort_ms_median"])),
        ]
    )
    return "\n".join(lines) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("run_dir", type=Path)
    parser.add_argument("output", type=Path, nargs="?", default=Path(__file__).with_name("generated-results.tex"))
    args = parser.parse_args()
    rendered = render_results(args.run_dir)
    temporary = args.output.with_suffix(args.output.suffix + ".tmp")
    temporary.write_text(rendered)
    temporary.replace(args.output)
    print(args.output.resolve())


if __name__ == "__main__":
    main()
