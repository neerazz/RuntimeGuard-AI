# RuntimeGuard-AI Overview (Paper-Ready Notes)

This document distills the blueprint, implementation, and initial measurements into a paper-friendly summary aimed at a Tier-1 venue (USENIX/CCS/ICSE). It highlights novelty, architecture, threat surface, evaluation hooks, and reproducibility details.

## Goals & Claims
- **Goal:** Demonstrate feasibility of cryptographically attested Article 14 compliance at runtime (human oversight, intervention capability, tamper-evident logging).
- **Claims (defensible for a reference system):**
  1) Merkle-backed audit log yields tamper-evident, append-only decisions for AI inferences.
  2) Deterministic attestation (digest over Merkle root + leaf count) enables verifiable snapshots without external trust.
  3) Policy engine encodes Article 14-inspired rules and enforces fail-safe escalation/block.
  4) Human-oversight loop is wired (escalation queue + reviewer actions) with certificate binding to proofs.
  5) Stack is self-contained (Python/React/SQLite) and reproducible via docker-compose.

## Architecture (as built)
- **Interceptor (FastAPI)**: Captures inference payload, calls mock model, evaluates policies, appends to Merkle log, returns decision + Merkle root/index.
- **Policy Engine**: Rules — protected categories (escalate), high-stakes context (escalate), confidence threshold (block <0.6), rate limit (block), prohibited content (block). Deterministic scoring and logging.
- **Merkle Audit Log (SQLite)**: SHA-256 leaves; recomputed roots after each append; verification per-leaf.
- **Attestation Stub**: Deterministic digest over `{merkle_root,total_leaves}` → stored as proof; verification rechecks against current root.
- **Certificates**: Aggregate counts (allow/block/escalate), bind to proof digest and period, store in SQLite.
- **Oversight UI (React/Vite)**: Escalation queue, audit viewer, Merkle root display, certificate list/issue.
- **Observability**: JSON logs (request IDs, decisions, merkle_index/root, timings), middleware timing, oversight stats endpoint.

## Data Flow (runtime)
1) Client → `/api/v1/inference` → interceptor records request hash.
2) Mock model returns deterministic risk + confidence.
3) Policy engine evaluates; decision = ALLOW/BLOCK/ESCALATE.
4) Storage: request row + decision → Merkle append → new root returned.
5) Oversight queue populated for ESCALATE; reviewers act via `/oversight/{id}/action`.
6) Proof issuance via `/certificates/issue` (attestation digest) → certificate persisted.
7) Proof verification via `/attestations/{proof_id}/verify` compares stored root vs current root.

## Implementation Details
- Language: Python 3.9+; FastAPI; SQLite persistence; deterministic mock model.
- Logging: Custom JSON formatter; root logger captures warnings; structured key/value messages.
- Schema: `src/core/schema.sql` aligns with blueprint (requests, decisions, merkle nodes, oversight, proofs, certificates).
- Reproducibility: `docker-compose up --build` or `uvicorn` + `npm run dev`; data stored in `data/audit_log.db`.

## APIs (key endpoints)
- `POST /api/v1/inference` → decision, rules_triggered, merkle_root/index, model output/confidence.
- `GET /api/v1/audit/logs` → recent decisions + merkle_root.
- `GET /api/v1/oversight/queue` / `POST /api/v1/oversight/{decision_id}/action`.
- `POST /api/v1/certificates/issue` → certificate (bound to proof).
- `GET /api/v1/attestations` / `POST /api/v1/attestations/{proof_id}/verify`.
- `GET /healthz`.

## Measurements (current)
- Synthetic benchmark (`python evaluation/run_benchmarks.py`, 50 reqs, local, mock model):
  - P50 latency ≈ 663 ms
  - P95 latency ≈ 884 ms
  - Max latency ≈ 4003 ms
  - Merkle root: `269c2f22e1f278ce17c97e3b542eb2a3f12eb37c46c8c2c43cb102f9d827db7e`
- In-process demo (`python scripts/demo_flow.py`): issues certificate after 5 requests; sample Merkle root `4e6ecb08...`.
- Live API run (credit_decision): BLOCK with rules `{ART14-HS, ART14-CONF}`, Merkle index ~105 (cumulative run).

## Threat Model (initial)
- Assumes honest-but-curious operator; storage integrity is guarded by Merkle root; DB access may be observable but modifications are detectable via root mismatch.
- Network adversary: mitigated by integrity checks; no transport security is shipped by default (expect TLS termination in deployment).
- Model outputs are untrusted; policy engine treats missing/low confidence as block/escalate (fail-safe).
- Dashboard requests are considered untrusted; backend performs action validation.

## Limitations (to disclose in paper)
- Attestation is deterministic digest, not zk-SNARK/Groth16; no trusted setup; no zero-knowledge privacy guarantees.
- Mock model only; no real high-risk AI inference.
- Benchmarks are localhost, synthetic workloads; no distributed or WAN latency considered.
- Single-node SQLite; no durability replication.
- Oversight authentication/authorization not implemented (add JWT/OIDC for production).

## Evaluation Plan (paper-ready)
- **Performance:** Latency/throughput vs batch size/concurrency; Merkle append cost; proof issuance/verification time.
- **Integrity:** Tamper detection — mutate DB leaf, verify root mismatch; replay/rollback detection.
- **Oversight Efficacy:** Escalation coverage for protected categories/high-stakes contexts; false-positive rate on benign workloads.
- **Scalability:** 100, 1K, 10K requests with Merkle growth; memory/disk footprint.
- **Ablations:** Policy rule toggles; rate-limit thresholds; confidence threshold sweeps.
- **Comparisons (qualitative):** Certificate Transparency-style logs; existing audit logging systems; zk attestation overhead (future).

## Paper Skeleton (to fill)
1) **Introduction:** Motivation (EU AI Act), need for runtime attestations; contributions list.
2) **Background:** Article 14 requirements; Merkle and attestation basics; human oversight patterns.
3) **Design:** Architecture, data flow, policy semantics, attestation format, certificate lifecycle, oversight UX.
4) **Implementation:** Tech stack, schemas, logging/observability, deployment model.
5) **Security/Threat Model:** Adversaries, assumptions, integrity properties, fail-safe behaviors.
6) **Evaluation:** Methodology; workloads; performance (latency/throughput), integrity checks, escalation coverage; limitations.
7) **Discussion:** Path to zk proofs (Groth16/PLONK), production hardening (authZ, HSMs, replication), regulatory implications.
8) **Related Work:** Audit logs, CT, zk systems, AI governance toolkits.
9) **Conclusion:** Feasibility and roadmap.

## Next Steps to Strengthen a Tier-1 Submission
- Swap deterministic proof with real zk circuit (poseidon/Merkle inclusion) and report proving/verification costs.
- Add authenticated oversight (OIDC), RBAC, and signed certificates.
- Expand workloads (credit underwriting, hiring, medical triage) with labeled edge cases; measure escalation precision/recall.
- Harden logging: include structured tracing IDs, ship OpenTelemetry hooks.
- Add durability (WAL + offsite snapshot) and remote attestation of binaries (TPM/SGX path, if applicable).
- Provide reproducible artifact: containerized benchmarks with fixed seeds; publish logs/figures.
