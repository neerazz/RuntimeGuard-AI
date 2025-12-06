# Repository Guidelines

## Project Structure & Module Organization
- Core Python backend lives under `src/` (`core/` for policy/merkle/cert logic, `api/` for FastAPI entrypoints, `prover/` for circom + snarkjs assets, `mock/` for synthetic models/workloads).
- Frontend oversight dashboard sits in `dashboard/` (Vite + React). Evaluation harness in `evaluation/` (benchmarks, analysis, generated results). Configs and policy presets in `configs/`.
- Tests in `tests/` mirror module paths; docs and paper assets in `docs/` and `paper/`; automation scripts in `scripts/`.

## Build, Test, and Development Commands
- Backend dev: `python -m venv venv && venv\Scripts\activate` then install via `pip install -e .` or `pip install -r requirements.txt` (keep lockfile updated if added). Run API with `uvicorn src.api.main:app --reload`.
- Frontend: `cd dashboard && npm install && npm run dev` for local UI; `npm run build` to produce static assets.
- ZK circuits: `bash src/prover/scripts/compile.sh` to build circuits and keys; `node src/prover/scripts/prove.js input.json output.json` to test proof generation.
- One-shot setup and CI parity: `bash scripts/setup_dev.sh`; run the full suite with `bash scripts/run_all_tests.sh` when present.

## Coding Style & Naming Conventions
- Python: 4-space indent, type hints required, snake_case for modules/functions/vars, CapWords for classes. Prefer pure functions in `core/` and FastAPI dependency injection in `api/`. Keep functions roughly <= 50 lines; refactor shared helpers into `src/core/utils.py` when needed. Target PEP8; run `black`/`ruff` if configured before PRs.
- TypeScript/React: camelCase for props/state, PascalCase for components, colocate component styles; keep hooks pure and side-effects in `useEffect`. Favor functional components and `src/dashboard/src/components` folder patterns.
- Circom: name circuits with UpperCamelCase; keep witness/public signal names descriptive and documented in comments near constraints.
- Filenames: align with module purpose (e.g., `policy_engine.py`, `merkle_log.py`, `audit.tsx`). Avoid abbreviations.

## Testing Guidelines
- Use `pytest` for backend; mirror test files to source names (e.g., `tests/test_merkle_log.py`). Mark slow/prover-heavy cases with `@pytest.mark.slow` to keep default runs quick; prefer deterministic fixtures over randomness.
- Dashboard: `npm test` (or `npm run test:unit` if defined). Add React Testing Library tests for components that transform data or enforce oversight flows. Snapshots only for stable UI.
- Coverage: aim for >= 80% on core policy/merkle paths; include edge cases for tamper detection and decision escalation. Add integration tests hitting FastAPI routes and verifying Merkle root changes.

## Commit & Pull Request Guidelines
- Follow conventional, focused commits (verb in imperative, scoped where useful), e.g., `add merkle verification guard`, `fix prover key path`, `docs: outline oversight flow`.
- Before opening a PR: describe intent, list commands run, mention coverage deltas, and link issues. Include screenshots or clips for dashboard changes and proof/benchmark outputs for prover updates.
- Keep PRs small and logically scoped; prefer separate PRs for docs, backend, dashboard, and prover circuit changes. Request review when tests are green.

## Security, Configuration, and Secrets
- Never commit proving keys, `.zkey` ceremony artifacts, or generated witnesses; store them in `build/` with `.gitignore` coverage. Keep `.env` and API keys out of git; document required vars in `README` or `docs/`.
- Validate incoming payloads in FastAPI schemas; reject unsigned or malformed inference requests. Ensure Merkle tree state and proof artifacts are hashed before storage. Treat dashboard API calls as untrusted; sanitize user-visible fields.
