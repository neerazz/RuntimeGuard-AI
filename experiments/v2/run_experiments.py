#!/usr/bin/env python3
"""Run the preregistered RuntimeGuard-AI V2 benchmark matrix.

The runner never overwrites a result directory. It records a source-tree digest,
a redacted host manifest, randomized condition order, raw per-operation samples,
and a JSONL execution receipt. Use --quick only for harness validation; quick
runs are marked non-canonical in the manifest.
"""

from __future__ import annotations

import argparse
import csv
import datetime as dt
import hashlib
from importlib import metadata
import itertools
import json
import os
import platform
import random
import shutil
import subprocess
import sys
import time
import traceback
from pathlib import Path
from typing import Any, Optional

REPO = Path(__file__).resolve().parents[2]
SEED = 20260711
REQUIRED_PYTHON = (3, 11)
REQUIRED_PACKAGES = {
    "numpy": "2.4.2",
    "pandas": "3.0.0",
    "matplotlib": "3.10.8",
}
SOURCE_GLOBS = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "src/**/*.rs",
    "src/**/Cargo.toml",
    "experiments/v2/*.py",
    "experiments/v2/*.md",
    "experiments/v2/requirements.txt",
    "docs/protocol-v2.md",
)


def run(command: list[str], *, capture: bool = False) -> str:
    result = subprocess.run(
        command,
        cwd=REPO,
        check=True,
        text=True,
        capture_output=capture,
    )
    return result.stdout.strip() if capture else ""


def redact_public_log(text: str) -> str:
    return text.replace(str(REPO), "<REPO>").replace(str(Path.home()), "<HOME>")


def run_logged(command: list[str], log: Path) -> None:
    result = subprocess.run(
        command,
        cwd=REPO,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    with log.open("a") as handle:
        handle.write(f"$ {' '.join(command)}\n")
        output = redact_public_log(result.stdout)
        handle.write(output)
        if not output.endswith("\n"):
            handle.write("\n")
        handle.write(f"[exit={result.returncode}]\n\n")
    if result.returncode != 0:
        raise subprocess.CalledProcessError(result.returncode, command)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def source_manifest() -> tuple[list[dict[str, str]], str]:
    paths: set[Path] = set()
    for pattern in SOURCE_GLOBS:
        paths.update(path for path in REPO.glob(pattern) if path.is_file())
    entries = [
        {"path": path.relative_to(REPO).as_posix(), "sha256": sha256_file(path)}
        for path in sorted(paths)
    ]
    encoded = json.dumps(entries, separators=(",", ":"), sort_keys=True).encode()
    return entries, hashlib.sha256(encoded).hexdigest()


def verify_source_closure(
    expected_files: list[dict[str, str]],
    expected_digest: str,
    actual_files: Optional[list[dict[str, str]]] = None,
    actual_digest: Optional[str] = None,
) -> None:
    if actual_files is None or actual_digest is None:
        actual_files, actual_digest = source_manifest()
    if actual_files != expected_files or actual_digest != expected_digest:
        raise RuntimeError(
            f"source drift invalidates run: {expected_digest} != {actual_digest}"
        )


def git_source_dirty(
    source_files: list[dict[str, str]], repo: Path = REPO
) -> bool:
    result = subprocess.run(
        [
            "git",
            "status",
            "--porcelain",
            "--untracked-files=all",
            "--",
            *[entry["path"] for entry in source_files],
        ],
        cwd=repo,
        check=True,
        text=True,
        capture_output=True,
    )
    return bool(result.stdout.strip())


def validate_environment(
    python_version: Optional[tuple[int, int]] = None,
    package_versions: Optional[dict[str, str]] = None,
) -> dict[str, str]:
    version = python_version or sys.version_info[:2]
    if version < REQUIRED_PYTHON:
        raise RuntimeError("RuntimeGuard experiments require Python 3.11 or newer")
    if package_versions is None:
        try:
            package_versions = {
                package: metadata.version(package) for package in REQUIRED_PACKAGES
            }
        except metadata.PackageNotFoundError as error:
            raise RuntimeError(f"missing pinned experiment dependency: {error.name}") from error
    mismatches = [
        f"{package}=={required} (found {package_versions.get(package, 'missing')})"
        for package, required in REQUIRED_PACKAGES.items()
        if package_versions.get(package) != required
    ]
    if mismatches:
        raise RuntimeError("experiment dependency mismatch: " + ", ".join(mismatches))
    return package_versions


def host_manifest(package_versions: dict[str, str]) -> dict[str, Any]:
    hardware: dict[str, Any] = {}
    try:
        raw = json.loads(run(["system_profiler", "-json", "SPHardwareDataType"], capture=True))
        item = raw.get("SPHardwareDataType", [{}])[0]
        # Deliberately exclude serial number, hardware UUID, and provisioning IDs.
        hardware = {
            "model_name": item.get("machine_name"),
            "model_identifier": item.get("machine_model"),
            "chip": item.get("chip_type"),
            "cores": item.get("number_processors"),
            "memory": item.get("physical_memory"),
        }
    except (subprocess.CalledProcessError, json.JSONDecodeError, IndexError):
        hardware = {"collection_error": "system_profiler unavailable"}
    return {
        "system": platform.system(),
        "release": platform.release(),
        "machine": platform.machine(),
        "python": platform.python_version(),
        "python_packages": package_versions,
        "hardware": hardware,
        "rustc": run(["rustc", "--version"], capture=True),
    }


def write_json(path: Path, value: Any) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    temporary.replace(path)


def append_csv(source: Path, destination: Path, extra: dict[str, Any]) -> None:
    with source.open(newline="") as input_handle:
        reader = csv.DictReader(input_handle)
        if reader.fieldnames is None:
            raise RuntimeError(f"missing CSV header in {source}")
        fieldnames = [*extra.keys(), *reader.fieldnames]
        exists = destination.exists()
        with destination.open("a", newline="") as output_handle:
            writer = csv.DictWriter(output_handle, fieldnames=fieldnames)
            if not exists:
                writer.writeheader()
            for row in reader:
                writer.writerow({**extra, **row})


def inline_conditions(quick: bool) -> list[dict[str, Any]]:
    repetitions = range(2 if quick else 20)
    threads = [1, 4] if quick else [1, 4, 8]
    prompt_bytes = [128, 2048] if quick else [128, 2048, 16384]
    modes = [
        ("policy-only", "none"),
        ("evidence", "none"),
        ("evidence", "data"),
        ("evidence", "full"),
    ]
    return [
        {
            "repetition": repetition,
            "threads": thread_count,
            "prompt_bytes": size,
            "mode": mode,
            "sync": sync,
        }
        for repetition, thread_count, size, (mode, sync) in itertools.product(
            repetitions, threads, prompt_bytes, modes
        )
    ]


def record_receipt(path: Path, payload: dict[str, Any]) -> None:
    with path.open("a") as handle:
        handle.write(json.dumps(payload, sort_keys=True) + "\n")


def record_failure(manifest: dict[str, Any], run_dir: Path, error: Exception) -> None:
    failure_log = run_dir / "failure.log"
    failure_log.write_text(traceback.format_exc())
    manifest["status"] = "failed"
    manifest["failed_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    manifest["failure"] = {
        "type": type(error).__name__,
        "message": str(error),
        "log_sha256": sha256_file(failure_log),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--quick", action="store_true", help="small non-canonical smoke matrix")
    parser.add_argument("--output-root", type=Path, default=REPO / "results" / "v2")
    parser.add_argument("--run-id")
    args = parser.parse_args()
    package_versions = validate_environment()

    timestamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run_id = args.run_id or f"runtimeguard-v2-{'quick' if args.quick else 'canonical'}-{timestamp}"
    run_dir = args.output_root / run_id
    if run_dir.exists():
        raise SystemExit(f"refusing to overwrite existing run directory: {run_dir}")
    raw_dir = run_dir / "raw"
    work_dir = run_dir / "work"
    raw_dir.mkdir(parents=True)
    work_dir.mkdir()

    files, source_digest = source_manifest()
    conditions = inline_conditions(args.quick)
    random.Random(SEED).shuffle(conditions)
    manifest: dict[str, Any] = {
        "schema_version": 1,
        "run_id": run_id,
        "canonical": not args.quick,
        "status": "running",
        "started_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "randomization_seed": SEED,
        "git_commit": run(["git", "rev-parse", "HEAD"], capture=True),
        "git_branch": run(["git", "branch", "--show-current"], capture=True),
        "git_dirty": git_source_dirty(files),
        "git_dirty_scope": "source_files",
        "source_digest_sha256": source_digest,
        "source_files": files,
        "host": host_manifest(package_versions),
        "matrix": {
            "inline": {
                "conditions": len(conditions),
                "requests_per_condition": 200 if args.quick else 1000,
                "warmup_requests": 20 if args.quick else 200,
                "shards": 4,
            },
            "epoch_record_counts": [100, 1000] if args.quick else [100, 1000, 10000, 100000],
            "epoch_repetitions": 3 if args.quick else 30,
            "recovery_record_counts": [100, 1000] if args.quick else [1000, 10000, 100000],
            "recovery_repetitions": 3 if args.quick else 30,
        },
        "completed": {"inline": 0, "epoch": 0, "recovery": 0},
    }
    write_json(run_dir / "manifest.json", manifest)
    receipt_path = run_dir / "commands.jsonl"

    try:
        preflight_log = run_dir / "preflight.log"
        for command in (
            ["cargo", "fmt", "--all", "--", "--check"],
            ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
            ["cargo", "test", "--workspace", "--all-targets"],
            ["cargo", "audit"],
        ):
            run_logged(command, preflight_log)
        manifest["preflight"] = {
            "status": "passed",
            "log_sha256": sha256_file(preflight_log),
        }
        write_json(run_dir / "manifest.json", manifest)

        run(["cargo", "build", "--release", "--workspace", "--all-targets"])
        inline_binary = REPO / "target" / "release" / "bench_inline"
        epoch_binary = REPO / "target" / "release" / "bench_epoch"
        recovery_binary = REPO / "target" / "release" / "bench_recovery"

        for index, condition in enumerate(conditions):
            condition_id = f"inline-{index:04d}"
            output = work_dir / f"{condition_id}.csv"
            logs = work_dir / f"{condition_id}-logs"
            started = time.perf_counter_ns()
            command = [
                str(inline_binary),
                "--run-id", run_id,
                "--mode", condition["mode"],
                "--requests", str(200 if args.quick else 1000),
                "--warmup", str(20 if args.quick else 200),
                "--threads", str(condition["threads"]),
                "--shards", "4",
                "--sync", condition["sync"],
                "--prompt-bytes", str(condition["prompt_bytes"]),
                "--log-directory", str(logs),
                "--output", str(output),
            ]
            run(command)
            append_csv(output, raw_dir / "inline.csv", {
                "condition_id": condition_id,
                "repetition": condition["repetition"],
            })
            output.unlink()
            shutil.rmtree(logs, ignore_errors=True)
            record_receipt(receipt_path, {
                "condition_id": condition_id,
                "kind": "inline",
                "condition": condition,
                "elapsed_ns": time.perf_counter_ns() - started,
                "status": "passed",
            })
            manifest["completed"]["inline"] = index + 1
            write_json(run_dir / "manifest.json", manifest)

        epoch_counts = manifest["matrix"]["epoch_record_counts"]
        for index, count in enumerate(epoch_counts):
            condition_id = f"epoch-{count}"
            output = work_dir / f"{condition_id}.csv"
            started = time.perf_counter_ns()
            run([
                str(epoch_binary),
                "--run-id", run_id,
                "--records", str(count),
                "--repetitions", str(manifest["matrix"]["epoch_repetitions"]),
                "--warmup", "1" if args.quick else "5",
                "--output", str(output),
            ])
            append_csv(output, raw_dir / "epoch.csv", {"condition_id": condition_id})
            output.unlink()
            record_receipt(receipt_path, {
                "condition_id": condition_id,
                "kind": "epoch",
                "record_count": count,
                "elapsed_ns": time.perf_counter_ns() - started,
                "status": "passed",
            })
            manifest["completed"]["epoch"] = index + 1
            write_json(run_dir / "manifest.json", manifest)

        recovery_counts = manifest["matrix"]["recovery_record_counts"]
        for index, count in enumerate(recovery_counts):
            condition_id = f"recovery-{count}"
            output = work_dir / f"{condition_id}.csv"
            logs = work_dir / f"{condition_id}-logs"
            started = time.perf_counter_ns()
            run([
                str(recovery_binary),
                "--run-id", run_id,
                "--records", str(count),
                "--repetitions", str(manifest["matrix"]["recovery_repetitions"]),
                "--warmup", "1" if args.quick else "5",
                "--shards", "4",
                "--log-directory", str(logs),
                "--output", str(output),
            ])
            append_csv(output, raw_dir / "recovery.csv", {"condition_id": condition_id})
            output.unlink()
            shutil.rmtree(logs, ignore_errors=True)
            record_receipt(receipt_path, {
                "condition_id": condition_id,
                "kind": "recovery",
                "record_count": count,
                "elapsed_ns": time.perf_counter_ns() - started,
                "status": "passed",
            })
            manifest["completed"]["recovery"] = index + 1
            write_json(run_dir / "manifest.json", manifest)

        verify_source_closure(files, source_digest)
        manifest["source_closure"] = {
            "status": "passed",
            "verified_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        }
        manifest["status"] = "complete"
        manifest["completed_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
        for name in ("inline.csv", "epoch.csv", "recovery.csv"):
            path = raw_dir / name
            manifest.setdefault("artifacts", {})[f"raw/{name}"] = sha256_file(path)
        manifest.setdefault("artifacts", {})["preflight.log"] = sha256_file(preflight_log)
        write_json(run_dir / "manifest.json", manifest)
        work_dir.rmdir()
    except Exception as error:
        record_failure(manifest, run_dir, error)
        write_json(run_dir / "manifest.json", manifest)
        raise

    print(run_dir.relative_to(REPO))
    return 0


if __name__ == "__main__":
    sys.exit(main())
