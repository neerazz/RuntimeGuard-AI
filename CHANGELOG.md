# Changelog

## 2.0.0 — Research-grade protocol rebuild

RuntimeGuard-AI 2.0.0 replaces the historical prototype with an implementation whose public claims map to code, tests, and reproducible evidence.

### Added

- Exact-source compiled policy descriptors and deterministic request/record commitments.
- Framed shard logs with explicit buffered, data-sync, and full-sync acknowledgement semantics.
- Ed25519-signed commit receipts with an externally supplied verification key.
- Single-writer evidence-directory lease, contiguous recovery, replay idempotency, and fail-stopped append errors.
- Domain-separated Merkle trees, logical-size-bound inclusion proofs, chained signed epoch statements, and receipt-to-epoch verification.
- Pinned Rust and Python environments, source-hashed non-overwriting experiments, raw observations, bootstrap confidence intervals, and figures generated from result files.
- Public protocol specification, single verification entrypoint, citation metadata, and a clean-tree release audit.

### Removed

- The unrelated repeated-squaring Groth16 circuit and its synthetic proving benchmark.
- Unsupported claims about zero-knowledge policy execution, asynchronous crash durability, public transparency anchoring, compromised-host anti-rollback, legal compliance, and deployment-scale throughput.
- Historical hard-coded and hypothetical figures, stale submission prose, generated build artifacts, and non-reproducible diagrams.

### Compatibility

Version 2 is a protocol and evidence-format break from the historical prototype. Existing prototype logs, figures, and benchmark outputs are not V2 evidence and are not accepted by the V2 recovery or analysis paths.
