# RuntimeGuard-AI

Reference implementation for cryptographically attested AI oversight inspired by the EU AI Act (Article 14). This repo ships a runnable stack: FastAPI interceptor + policy engine + Merkle-backed audit log + deterministic attestation stub + compliance certificates + React dashboard.

## Repository map
- `RuntimeGuard-AI-Project-Blueprint.md` — full blueprint (strategy, architecture, roadmap).
- `ImplementationPlan.md` — executed implementation plan.
- `docs/overview.md` — architecture, data flow, API surface, evaluation summary (read this next).
- `src/` — backend (core logic, API, mock model, storage, attestation, certificates).
- `dashboard/` — Vite/React oversight UI.
- `evaluation/` — synthetic benchmarks.
- `scripts/` — setup, demo flow, tests.

## Quickstart (local)
```bash
python -m venv venv
source venv/bin/activate
pip install -e .
uvicorn src.api.main:app --host 0.0.0.0 --port 8000
```
Dashboard: `cd dashboard && npm install && npm run dev -- --host`

## One-command stack
```bash
docker-compose up --build
```
- API: http://localhost:8000
- Dashboard: http://localhost:5173

## API highlights
- `POST /api/v1/inference` — proxy to mock model, policy-evaluate, Merkle-log decision.
- `GET /api/v1/audit/logs` — latest decisions + Merkle root.
- `GET /api/v1/oversight/queue` / `POST /api/v1/oversight/{decision_id}/action` — human oversight.
- `POST /api/v1/certificates/issue` — generate attestation + compliance certificate.
- `POST /api/v1/attestations/{proof_id}/verify` — verify stored proof vs current Merkle root.

## Design (concise)
- **Policy Engine**: Article 14-inspired defaults (protected categories, high-stakes, confidence threshold, rate limiting, prohibited content).
- **Merkle Audit Log**: append-only SHA-256 tree in SQLite for tamper evidence.
- **Attestation Stub**: deterministic digest over Merkle root + leaf count; verification endpoint provided.
- **Certificates**: aggregate decisions, bind to proof digest, store in SQLite.
- **Observability**: JSON logs with request IDs, decisions, Merkle indices/roots, timings; middleware timing on every request.

## Benchmarks (synthetic, local)
- `python evaluation/run_benchmarks.py` (50 requests, mock model): P50 ≈ 663 ms, P95 ≈ 884 ms, max ≈ 4003 ms, Merkle root `269c2f22e1f278ce17c97e3b542eb2a3f12eb37c46c8c2c43cb102f9d827db7e`.

## Demo / tests
- In-process end-to-end: `python scripts/demo_flow.py` (logs requests, issues certificate).
- Tests: `python -m pytest` (core Merkle/policy/attestation certificate flows).

## Deployment tips
- Backend: `uvicorn src.api.main:app --host 0.0.0.0 --port 8000`
- Dashboard build: `cd dashboard && npm run build` (static in `dist/`)
- Data persists in `data/audit_log.db`; safe to mount a volume in containerized runs.

## Publication

This repository accompanies the paper:

**RuntimeGuard-AI: Cryptographically Attested Runtime Oversight for EU AI Act High-Risk Systems**

*Submission Target: USENIX Security 2026*

Paper materials:
- `paper/RuntimeGuard_AI_Paper.md` — Full manuscript (Markdown, LaTeX-ready)
- `paper/research.md` — SOTA analysis and novelty audit
- `paper/main.md` — Draft with evaluation summary
- `paper/figures/` — Generated evaluation figures

## Supplementary Materials (Code Artifacts)

| Artifact | Location | Description |
|----------|----------|-------------|
| Policy Engine | `src/core/policy_engine.py` | Article 14-inspired rule evaluation (230 LOC) |
| Merkle Log | `src/core/merkle_log.py` | Append-only audit log with SHA-256 (92 LOC) |
| Attestation Service | `src/core/attestation.py` | Proof generation and verification (112 LOC) |
| Certificate Service | `src/core/certificate.py` | Compliance certificate issuance |
| ZK Circuits | `src/prover/circuits/` | circom policy attestation circuit |
| Tests | `tests/` | pytest suite for core components |
| Benchmarks | `evaluation/` | Performance evaluation scripts |

### Reproducibility Commands

```bash
# Setup environment
python -m venv venv && source venv/bin/activate
pip install -e .

# Run tests
pytest tests/

# Run benchmarks
python evaluation/run_benchmarks.py

# Generate figures
python evaluation/generate_figures.py

# Full stack
docker-compose up --build
```

## Citation

```bibtex
@inproceedings{runtimeguard2026,
  title={{RuntimeGuard-AI}: Cryptographically Attested Runtime Oversight for {EU AI Act} High-Risk Systems},
  author={Anonymous},
  booktitle={USENIX Security Symposium},
  year={2026},
  note={Under review}
}
### Running ZK Benchmarks (Rust)

The ZK Attestation component (`src/attestor`) is written in Rust for performance validation.

1. **Install Rust:**
   ```bash
   # Linux/macOS
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Windows
   # Download installer from https://rustup.rs/
   ```

2. **Run the Benchmark:**
   ```bash
   cd src/attestor
   cargo run --release --bin bench_prove -- --constraints 50000 --samples 10
   ```

   **Expected Output:** JSON result with mean proving time. Use this to populate Table 7.3 in the paper.
