# RuntimeGuard-AI V2 experiments

This directory owns the successor paper's reproducible evidence pipeline. It does not reuse the interpolated or hypothetical values from the published version-of-record figure script.

## Research questions

1. What latency and throughput cost is added by record construction, Ed25519 commit-receipt signing, and each explicit durability boundary (`none`, `data`, and `full`) relative to the same compiled policy evaluator without evidence logging?
2. How do worker count and prompt size affect the policy-only and evidence paths?
3. How do signed-epoch construction, Merkle proof generation/verification, and Ed25519 verification scale with epoch size?
4. How does evidence-log opening, integrity validation, reading, and global sequence sorting scale with persisted record count?

The benchmark evaluates a small deterministic regex policy fixture. It does not claim to measure arbitrary policy languages, model inference, network latency, multi-host operation, trusted key management, or legal compliance.

## Preregistered full matrix

- Inline modes: policy-only, evidence/no-sync, evidence/data-sync, evidence/full-sync.
- Worker threads: 1, 4, 8.
- Prompt sizes: 128, 2,048, 16,384 bytes.
- Repetitions: 20 randomized runs per condition with seed `20260711`.
- Per run: 200 warm-up requests followed by 1,000 measured requests.
- Evidence shards: 4. Evidence commits are globally serialized to preserve a recoverable contiguous sequence; sharding distributes files, not commit ordering.
- Signed epochs: 100, 1,000, 10,000, and 100,000 records; 5 warm-ups and 30 measured repetitions.
- Recovery: 1,000, 10,000, and 100,000 persisted records; 5 warm-ups and 30 measured repetitions.

Per-request latency percentiles are calculated within each run. Reported condition estimates are medians across independent runs with deterministic 10,000-resample percentile-bootstrap 95% confidence intervals. Evidence overhead is paired with the policy-only run sharing repetition, worker count, and prompt size. Throughput uses condition wall-clock time rather than the sum of per-request latencies.

## Run

```bash
python3.12 -m venv .venv
.venv/bin/pip install -r experiments/v2/requirements.txt
.venv/bin/python experiments/v2/run_experiments.py
.venv/bin/python experiments/v2/analyze.py results/v2/<run-id>
```

Validate the harness without creating publishable evidence:

```bash
.venv/bin/python experiments/v2/run_experiments.py --quick
```

Python 3.11 or newer is required. The runner fails before creating a result directory unless the exact versions in `requirements.txt` are installed, and records those versions in the host manifest. Quick runs are marked `canonical: false`, carry `[QUICK]` figure titles, and are ignored by Git. The runner refuses to overwrite result directories or benchmark outputs.

If any command fails, the manifest records the exception type and message and binds `failure.log` by SHA-256. Failed or interrupted directories are diagnostic artifacts only and cannot be resumed or treated as evidence.

## Result contract

Each run contains:

- `preflight.log`: formatting, strict Clippy, all-target test, and dependency-audit gates with exit codes;
- `manifest.json`: redacted host metadata, source-file digests, exact matrix, completion state, and artifact SHA-256 values;
- `commands.jsonl`: one execution receipt per condition;
- `failure.log`: traceback for a failed run, present only on failure and hash-bound by the manifest;
- `raw/*.csv`: canonical measurements emitted by the Rust binaries;
- `summary/*.csv`: run-level and condition-level statistics;
- `summary/key_findings.json`: machine-readable selected-condition estimates and largest-scale component results;
- `figures/*.{png,pdf}`: figures generated only from the raw run data.

The host manifest intentionally omits serial number, hardware UUID, provisioning ID, username, and absolute repository location.
