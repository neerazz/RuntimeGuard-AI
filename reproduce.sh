#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"
export PATH="$HOME/.cargo/bin:$PATH"

MODE="${1:---verify}"
PYTHON_BIN="${PYTHON_BIN:-python3.12}"

case "$MODE" in
  --verify|--quick|--canonical) ;;
  *)
    echo "usage: ./reproduce.sh [--verify|--quick|--canonical]" >&2
    exit 2
    ;;
esac

if ! command -v cargo-audit >/dev/null 2>&1; then
  echo "cargo-audit is required; install it with: cargo install cargo-audit --locked" >&2
  exit 1
fi
if ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
  echo "$PYTHON_BIN is required (set PYTHON_BIN to another Python >=3.11 interpreter)" >&2
  exit 1
fi

if [[ ! -x .venv/bin/python ]]; then
  "$PYTHON_BIN" -m venv .venv
fi
.venv/bin/pip install --disable-pip-version-check -r experiments/v2/requirements.txt

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo audit
.venv/bin/python -m unittest \
  experiments/v2/test_run_experiments.py \
  experiments/v2/test_analyze.py \
  paper/v2/test_generate_results_tex.py \
  tools/test_audit_public_tree.py -v
.venv/bin/python -m py_compile \
  experiments/v2/run_experiments.py \
  experiments/v2/analyze.py \
  paper/v2/generate_diagrams.py \
  paper/v2/generate_results_tex.py \
  tools/audit_public_tree.py

if [[ "$MODE" == "--verify" ]]; then
  echo "verification complete"
  exit 0
fi

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
if [[ "$MODE" == "--quick" ]]; then
  RUN_ID="runtimeguard-v2-quick-${STAMP}"
  .venv/bin/python experiments/v2/run_experiments.py --quick --run-id "$RUN_ID"
else
  RUN_ID="runtimeguard-v2-canonical-${STAMP}"
  .venv/bin/python experiments/v2/run_experiments.py --run-id "$RUN_ID"
fi
.venv/bin/python experiments/v2/analyze.py "results/v2/$RUN_ID"
echo "results/v2/$RUN_ID"
