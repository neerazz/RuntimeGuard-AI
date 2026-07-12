#!/usr/bin/env python3
"""Validate, summarize, and plot one RuntimeGuard-AI V2 result run."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Callable

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

REPO = Path(__file__).resolve().parents[2]
SEED = 20260711
BOOTSTRAP_RESAMPLES = 10_000
MODE_ORDER = ["policy_only", "evidence:none", "evidence:data", "evidence:full"]
MODE_LABELS = {
    "policy_only": "Policy only",
    "evidence:none": "Evidence / no sync",
    "evidence:data": "Evidence / data sync",
    "evidence:full": "Evidence / full sync",
}
COLORS = {
    "policy_only": "#4C78A8",
    "evidence:none": "#59A14F",
    "evidence:data": "#F28E2B",
    "evidence:full": "#E15759",
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def validate_source_snapshot(manifest: dict, repo: Path = REPO) -> None:
    actual = []
    for expected in manifest.get("source_files", []):
        path = repo / expected["path"]
        if not path.is_file():
            raise ValueError(f"source drift: missing {expected['path']}")
        actual.append({"path": expected["path"], "sha256": sha256(path)})
    encoded = json.dumps(actual, separators=(",", ":"), sort_keys=True).encode()
    digest = hashlib.sha256(encoded).hexdigest()
    if actual != manifest.get("source_files") or digest != manifest.get("source_digest_sha256"):
        raise ValueError(
            f"source drift: {manifest.get('source_digest_sha256')} != {digest}"
        )


def validate_run(run_dir: Path, manifest: dict) -> None:
    if manifest.get("status") != "complete":
        raise ValueError(f"run is not complete: {manifest.get('status')}")
    for relative, expected in manifest.get("artifacts", {}).items():
        path = run_dir / relative
        actual = sha256(path)
        if actual != expected:
            raise ValueError(f"artifact digest mismatch for {relative}: {actual} != {expected}")


def percentile(values: pd.Series, quantile: float) -> float:
    return float(np.quantile(values.to_numpy(dtype=float), quantile))


def bootstrap_ci(
    values: np.ndarray,
    statistic: Callable[[np.ndarray], float] = np.median,
) -> tuple[float, float]:
    values = np.asarray(values, dtype=float)
    if len(values) == 1:
        return float(values[0]), float(values[0])
    rng = np.random.default_rng(SEED)
    indices = rng.integers(0, len(values), size=(BOOTSTRAP_RESAMPLES, len(values)))
    estimates = np.apply_along_axis(statistic, 1, values[indices])
    low, high = np.quantile(estimates, [0.025, 0.975])
    return float(low), float(high)


def mode_key(frame: pd.DataFrame) -> pd.Series:
    return np.where(
        frame["mode"].eq("policy_only"),
        "policy_only",
        "evidence:" + frame["sync_policy"],
    )


def summarize_inline(raw: pd.DataFrame) -> tuple[pd.DataFrame, pd.DataFrame, pd.DataFrame]:
    numeric = ["latency_ns", "condition_elapsed_ns", "threads", "prompt_bytes", "repetition"]
    for column in numeric:
        raw[column] = pd.to_numeric(raw[column])
    raw["mode_key"] = mode_key(raw)
    run_keys = [
        "condition_id",
        "repetition",
        "mode_key",
        "threads",
        "prompt_bytes",
    ]
    run_rows = []
    for keys, frame in raw.groupby(run_keys, sort=False):
        elapsed = frame["condition_elapsed_ns"].unique()
        if len(elapsed) != 1:
            raise ValueError(f"condition {keys[0]} has inconsistent elapsed time")
        run_rows.append(
            {
                **dict(zip(run_keys, keys, strict=True)),
                "requests": len(frame),
                "p50_us": percentile(frame["latency_ns"], 0.50) / 1_000,
                "p95_us": percentile(frame["latency_ns"], 0.95) / 1_000,
                "p99_us": percentile(frame["latency_ns"], 0.99) / 1_000,
                "throughput_rps": len(frame) * 1_000_000_000 / elapsed[0],
            }
        )
    run_summary = pd.DataFrame(run_rows)

    condition_keys = ["mode_key", "threads", "prompt_bytes"]
    condition_rows = []
    for keys, frame in run_summary.groupby(condition_keys, sort=False):
        row = dict(zip(condition_keys, keys, strict=True))
        for metric in ["p50_us", "p95_us", "p99_us", "throughput_rps"]:
            values = frame[metric].to_numpy(dtype=float)
            low, high = bootstrap_ci(values)
            row[f"{metric}_median"] = float(np.median(values))
            row[f"{metric}_ci_low"] = low
            row[f"{metric}_ci_high"] = high
        row["repetitions"] = len(frame)
        condition_rows.append(row)
    condition_summary = pd.DataFrame(condition_rows)

    baseline = run_summary.loc[run_summary["mode_key"] == "policy_only"].rename(
        columns={
            "p50_us": "baseline_p50_us",
            "p99_us": "baseline_p99_us",
            "throughput_rps": "baseline_throughput_rps",
        }
    )
    compared = run_summary.loc[run_summary["mode_key"] != "policy_only"].merge(
        baseline[
            [
                "repetition",
                "threads",
                "prompt_bytes",
                "baseline_p50_us",
                "baseline_p99_us",
                "baseline_throughput_rps",
            ]
        ],
        on=["repetition", "threads", "prompt_bytes"],
        validate="many_to_one",
    )
    compared["p50_overhead_percent"] = (
        compared["p50_us"] / compared["baseline_p50_us"] - 1
    ) * 100
    compared["p99_overhead_percent"] = (
        compared["p99_us"] / compared["baseline_p99_us"] - 1
    ) * 100
    compared["throughput_change_percent"] = (
        compared["throughput_rps"] / compared["baseline_throughput_rps"] - 1
    ) * 100
    overhead_rows = []
    for keys, frame in compared.groupby(condition_keys, sort=False):
        row = dict(zip(condition_keys, keys, strict=True))
        for metric in [
            "p50_overhead_percent",
            "p99_overhead_percent",
            "throughput_change_percent",
        ]:
            values = frame[metric].to_numpy(dtype=float)
            low, high = bootstrap_ci(values)
            row[f"{metric}_median"] = float(np.median(values))
            row[f"{metric}_ci_low"] = low
            row[f"{metric}_ci_high"] = high
        row["paired_repetitions"] = len(frame)
        overhead_rows.append(row)
    overhead = pd.DataFrame(overhead_rows)
    return run_summary, condition_summary, overhead


def generic_summary(raw: pd.DataFrame, group: str, metrics: list[str]) -> pd.DataFrame:
    raw[group] = pd.to_numeric(raw[group])
    for metric in metrics:
        raw[metric] = pd.to_numeric(raw[metric])
    rows = []
    for key, frame in raw.groupby(group, sort=True):
        row = {group: key, "repetitions": len(frame)}
        for metric in metrics:
            values = frame[metric].to_numpy(dtype=float)
            low, high = bootstrap_ci(values)
            row[f"{metric}_median"] = float(np.median(values))
            row[f"{metric}_ci_low"] = low
            row[f"{metric}_ci_high"] = high
        rows.append(row)
    return pd.DataFrame(rows)


def save_figure(fig: plt.Figure, figures: Path, stem: str) -> None:
    fig.tight_layout()
    fig.savefig(figures / f"{stem}.png", dpi=220, bbox_inches="tight")
    fig.savefig(figures / f"{stem}.pdf", bbox_inches="tight")
    plt.close(fig)


def plot_inline_latency(summary: pd.DataFrame, figures: Path, quick: bool) -> None:
    prompt = 2048 if 2048 in summary["prompt_bytes"].values else summary["prompt_bytes"].min()
    threads = 4 if 4 in summary["threads"].values else summary["threads"].max()
    frame = summary[(summary["prompt_bytes"] == prompt) & (summary["threads"] == threads)].copy()
    frame["order"] = frame["mode_key"].map({key: index for index, key in enumerate(MODE_ORDER)})
    frame = frame.sort_values("order")
    x = np.arange(len(frame))
    fig, ax = plt.subplots(figsize=(8.2, 4.8))
    for offset, metric, marker in [(-0.08, "p50", "o"), (0.08, "p99", "s")]:
        center = frame[f"{metric}_us_median"].to_numpy()
        low = frame[f"{metric}_us_ci_low"].to_numpy()
        high = frame[f"{metric}_us_ci_high"].to_numpy()
        ax.errorbar(
            x + offset,
            center,
            yerr=np.vstack([center - low, high - center]),
            fmt=marker,
            markersize=7,
            capsize=3,
            linewidth=1.8,
            color="#2F2F2F" if metric == "p50" else "#7A5195",
            label=metric,
        )
    ax.set_xticks(x, [MODE_LABELS[key] for key in frame["mode_key"]], rotation=12, ha="right")
    ax.set_yscale("log")
    ax.set_ylabel("Per-request latency (µs)")
    ax.set_title(f"Inline latency: {threads} threads, {prompt}-byte prompts" + (" [QUICK]" if quick else ""))
    ax.legend(frameon=False)
    ax.grid(axis="y", alpha=0.25)
    save_figure(fig, figures, "inline_latency")


def plot_throughput(summary: pd.DataFrame, figures: Path, quick: bool) -> None:
    prompt = 2048 if 2048 in summary["prompt_bytes"].values else summary["prompt_bytes"].min()
    frame = summary[summary["prompt_bytes"] == prompt]
    fig, ax = plt.subplots(figsize=(7.4, 4.6))
    offsets = {"policy_only": -0.06, "evidence:none": -0.02, "evidence:data": 0.02, "evidence:full": 0.06}
    markers = {"policy_only": "o", "evidence:none": "s", "evidence:data": "^", "evidence:full": "D"}
    for key in MODE_ORDER:
        group = frame[frame["mode_key"] == key].sort_values("threads")
        if group.empty:
            continue
        center = group["throughput_rps_median"].to_numpy()
        low = group["throughput_rps_ci_low"].to_numpy()
        high = group["throughput_rps_ci_high"].to_numpy()
        ax.errorbar(
            group["threads"] + offsets[key],
            center,
            yerr=np.vstack([center - low, high - center]),
            marker=markers[key],
            linewidth=2,
            capsize=3,
            label=MODE_LABELS[key],
            color=COLORS[key],
        )
    ax.set_xlabel("Worker threads")
    ax.set_ylabel("Throughput (requests/s)")
    ax.set_yscale("log")
    ax.set_title(f"Throughput scaling: {prompt}-byte prompts" + (" [QUICK]" if quick else ""))
    ax.grid(alpha=0.25)
    ax.legend(frameon=False, ncols=2)
    save_figure(fig, figures, "throughput_scaling")


def plot_epoch(summary: pd.DataFrame, figures: Path, quick: bool) -> None:
    fig, ax = plt.subplots(figsize=(7.0, 4.5))
    for metric, label, color in [
        ("seal_epoch_ns_median", "Build + sign epoch", "#4C78A8"),
        ("generate_proof_ns_median", "Generate inclusion proof", "#F28E2B"),
        ("verify_proof_ns_median", "Verify inclusion proof", "#59A14F"),
        ("verify_signature_ns_median", "Verify epoch signature", "#E15759"),
    ]:
        ax.plot(summary["record_count"], summary[metric] / 1_000, marker="o", label=label, color=color)
    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlabel("Records per epoch")
    ax.set_ylabel("Latency (µs, log scale)")
    ax.set_title("Attestation operation scaling" + (" [QUICK]" if quick else ""))
    ax.grid(which="both", alpha=0.25)
    ax.legend(frameon=False, loc="center left", bbox_to_anchor=(1.01, 0.5))
    save_figure(fig, figures, "attestation_scaling")


def plot_recovery(summary: pd.DataFrame, figures: Path, quick: bool) -> None:
    fig, ax = plt.subplots(figsize=(6.8, 4.4))
    ax.plot(summary["record_count"], summary["open_ns_median"] / 1_000_000, marker="o", label="Open + validate")
    ax.plot(summary["record_count"], summary["recover_ns_median"] / 1_000_000, marker="s", label="Read + sort")
    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlabel("Persisted records")
    ax.set_ylabel("Latency (ms, log scale)")
    ax.set_title("Evidence recovery scaling" + (" [QUICK]" if quick else ""))
    ax.grid(which="both", alpha=0.25)
    ax.legend(frameon=False)
    save_figure(fig, figures, "recovery_scaling")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("run_dir", type=Path)
    args = parser.parse_args()
    run_dir = args.run_dir.resolve()
    manifest = json.loads((run_dir / "manifest.json").read_text())
    validate_source_snapshot(manifest)
    validate_run(run_dir, manifest)

    summary_dir = run_dir / "summary"
    figures_dir = run_dir / "figures"
    summary_dir.mkdir(exist_ok=False)
    figures_dir.mkdir(exist_ok=False)

    inline_raw = pd.read_csv(run_dir / "raw" / "inline.csv")
    run_summary, condition_summary, overhead = summarize_inline(inline_raw)
    run_summary.to_csv(summary_dir / "inline_runs.csv", index=False)
    condition_summary.to_csv(summary_dir / "inline_conditions.csv", index=False)
    overhead.to_csv(summary_dir / "inline_overhead.csv", index=False)

    epoch_raw = pd.read_csv(run_dir / "raw" / "epoch.csv")
    epoch_summary = generic_summary(
        epoch_raw,
        "record_count",
        [
            "build_tree_ns",
            "seal_epoch_ns",
            "generate_proof_ns",
            "verify_proof_ns",
            "verify_signature_ns",
        ],
    )
    epoch_summary.to_csv(summary_dir / "epoch.csv", index=False)

    recovery_raw = pd.read_csv(run_dir / "raw" / "recovery.csv")
    recovery_summary = generic_summary(
        recovery_raw,
        "record_count",
        ["open_ns", "recover_ns"],
    )
    recovery_summary.to_csv(summary_dir / "recovery.csv", index=False)

    quick = not manifest["canonical"]
    plot_inline_latency(condition_summary, figures_dir, quick)
    plot_throughput(condition_summary, figures_dir, quick)
    plot_epoch(epoch_summary, figures_dir, quick)
    plot_recovery(recovery_summary, figures_dir, quick)

    selected_prompt = 2048 if 2048 in condition_summary["prompt_bytes"].values else int(
        condition_summary["prompt_bytes"].min()
    )
    selected_threads = 4 if 4 in condition_summary["threads"].values else int(
        condition_summary["threads"].max()
    )
    selected = condition_summary[
        (condition_summary["prompt_bytes"] == selected_prompt)
        & (condition_summary["threads"] == selected_threads)
    ]
    findings = {
        "canonical": bool(manifest["canonical"]),
        "inline_condition": {
            "threads": selected_threads,
            "prompt_bytes": selected_prompt,
            "modes": {
                row["mode_key"]: {
                    "p50_us_median": float(row["p50_us_median"]),
                    "p50_us_ci_95": [
                        float(row["p50_us_ci_low"]),
                        float(row["p50_us_ci_high"]),
                    ],
                    "p99_us_median": float(row["p99_us_median"]),
                    "p99_us_ci_95": [
                        float(row["p99_us_ci_low"]),
                        float(row["p99_us_ci_high"]),
                    ],
                    "throughput_rps_median": float(row["throughput_rps_median"]),
                    "throughput_rps_ci_95": [
                        float(row["throughput_rps_ci_low"]),
                        float(row["throughput_rps_ci_high"]),
                    ],
                    "repetitions": int(row["repetitions"]),
                }
                for _, row in selected.iterrows()
            },
        },
        "largest_epoch": {
            "record_count": int(epoch_summary.iloc[-1]["record_count"]),
            "seal_epoch_us_median": float(epoch_summary.iloc[-1]["seal_epoch_ns_median"] / 1_000),
            "verify_proof_us_median": float(
                epoch_summary.iloc[-1]["verify_proof_ns_median"] / 1_000
            ),
            "verify_signature_us_median": float(
                epoch_summary.iloc[-1]["verify_signature_ns_median"] / 1_000
            ),
        },
        "largest_recovery": {
            "record_count": int(recovery_summary.iloc[-1]["record_count"]),
            "open_ms_median": float(recovery_summary.iloc[-1]["open_ns_median"] / 1_000_000),
            "read_sort_ms_median": float(
                recovery_summary.iloc[-1]["recover_ns_median"] / 1_000_000
            ),
        },
        "scope": "Controlled single-host component benchmark; quick runs are pipeline validation only.",
    }
    findings_path = summary_dir / "key_findings.json"
    findings_path.write_text(json.dumps(findings, indent=2, sort_keys=True) + "\n")

    generated = [*summary_dir.glob("*"), *figures_dir.glob("*")]
    manifest["analysis"] = {
        "bootstrap_resamples": BOOTSTRAP_RESAMPLES,
        "random_seed": SEED,
        "artifacts": {
            path.relative_to(run_dir).as_posix(): sha256(path) for path in sorted(generated)
        },
    }
    (run_dir / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(run_dir)


if __name__ == "__main__":
    main()
