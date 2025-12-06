#!/usr/bin/env bash
set -e

echo "=== RuntimeGuard-AI Setup ==="

python --version
python -m venv venv
source venv/bin/activate

pip install --upgrade pip
pip install -e ".[dev]"

echo "Initializing database..."
python - <<'PY'
from src.core.db import init_db
init_db()
print("DB ready")
PY

echo "To run API: source venv/bin/activate && uvicorn src.api.main:app --reload"
