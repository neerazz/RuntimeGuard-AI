# RuntimeGuard-AI Implementation Plan

This plan translates the blueprint into concrete tasks for a self-contained, end-to-end prototype with strong logging/observability. The goal is to deliver a deployable stack that runs locally with minimal setup (Python + Node for dashboard) and ships sane defaults.

## Objectives
- Provide a FastAPI backend that intercepts requests, evaluates policies, writes tamper-evident Merkle-backed audit logs, and emits compliance certificates.
- Offer a stubbed ZK attestation flow (deterministic proof artifact + verifier) to keep the project runnable without circom toolchain.
- Surface human oversight (escalations + actions) and audit browsing via REST + a lightweight React/Vite dashboard.
- Ship configs, scripts, and tests so the system can be run and validated on any machine.

## Work Breakdown
1) **Repository scaffold**
   - Create pyproject/requirements, runtime settings, logging config, base README.
   - Establish directories from blueprint (src/core, src/api, src/mock, src/prover, dashboard, configs, tests, scripts, evaluation).
   - Add data/ path for SQLite DB and Merkle persistence.

2) **Core backend logic**
   - Data models: requests, decisions, oversight actions, certificates.
   - Policy engine: built-in Article 14 rules with deterministic decisions + structured results.
   - Merkle audit log: append-only leaf hashing, root tracking, verification helper.
   - Certificate generator: summarize decisions, bind to Merkle root + attestation digest.
   - Observability: structlog-style logging, request/decision tracing IDs.

3) **API layer (FastAPI)**
   - Middleware interceptor capturing request metadata and model outputs.
   - Routes: inference proxy, audit log listing/roots, oversight queue/actions, certificates, health.
   - Persistence: SQLite via lightweight DAO; ensure migrations/bootstrap of schema.
   - Attestation stub: pseudo proof generator (hash chain) + verifier endpoint.

4) **Mock model + workload**
   - Deterministic mock AI model with configurable responses and confidence scores.
   - Synthetic workload generator script (e.g., send N requests, collect metrics).
   - Batch attestation trigger for demo.

5) **Dashboard (Vite/React)**
   - Views: escalation queue, audit log list, compliance status/certificate viewer.
   - Shared API hook client with base URL config.
   - Basic theming and loading/error states; polls backend for updates.

6) **Evaluation + scripts**
   - Benchmark harness to send load and measure latency/throughput.
   - Setup script to initialize env, install deps, bootstrap DB, and start services.
   - docker-compose for one-shot run (backend + dashboard).

7) **Testing**
   - Unit tests: policy engine, Merkle log, certificate hashing, attestation stub.
   - Integration smoke: inference -> decision -> log -> certificate path.

## Deliverables & Order
1. Scaffold + core backend (models, policy, Merkle, cert, logging).
2. API with persistence + attestation stub + mock model/workload.
3. Dashboard UI + scripts (setup, docker-compose) + evaluation harness.
4. Tests and docs updates; sanity run of key flows.

## Logging & Observability
- Use structured JSON logging with request IDs, decision IDs, Merkle index/root, and timing.
- API middleware to time requests and emit metrics.
- Audit endpoints expose latest Merkle root and attestation digest for verification.

