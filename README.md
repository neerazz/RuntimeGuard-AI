# RuntimeGuard-AI

**RuntimeGuard-AI** is the official reference implementation for the research paper:

> **"RuntimeGuard-AI: Scalable Tamper-Evident Accountability for High-Risk AI Systems Under the EU AI Act"**
> *Submitted to USENIX Security 2026*
>
> **Author:** Neeraj Kumar Singh Beshane

**Repository:** [https://github.com/neerazz/RuntimeGuard-AI](https://github.com/neerazz/RuntimeGuard-AI)

This repository provides the cryptographic attestation architecture described in the paper, designed to meet **Article 14 (Human Oversight)** requirements without compromising inference latency.

---

## 📄 Research Paper

The full manuscript and supplementary materials are available in the `paper/` directory:

*   **[RuntimeGuard_AI_Submission_Ready.md](paper/RuntimeGuard_AI_Submission_Ready.md)**: The complete research paper (Markdown, submission-ready).
*   **[research.md](paper/research.md)**: State-of-the-art analysis and novelty audit.
*   **[figures/](paper/figures/)**: All evaluation plots and architecture diagrams.

---

## 🏗️ Repository Structure

This project is strictly scoped to the artifacts described in the paper.

```
runtimeguard-ai/
├── src/
│   ├── inline/           # [Rust] Inline Policy Engine (Paper §5.2)
│   │   └── src/
│   │       ├── engine.rs # Theorem 1 (Latency Separation) via try_send + tokio::spawn
│   │       └── types.rs  # ComplianceRecord, PolicyResult
│   │
│   └── attestor/         # [Rust] ZK Attestation Service (Paper §5.3)
│       └── src/
│           ├── circuit.rs    # ComplianceCircuit (Groth16)
│           ├── merkle.rs     # Merkle Tree for Inclusion Proofs (Appendix B.2)
│           └── bin/
│               └── bench_prove.rs  # Benchmarking harness
│
├── paper/                # Research manuscript and figures
├── Cargo.toml            # Workspace definition
└── LICENSE               # MIT License
```

---

## 🚀 Key Claims Validation

### 1. Latency Separation (Theorem 1)
The **Inline Policy Engine** (`src/inline`) demonstrates how policy enforcement is decoupled from logging I/O.
*   **Code:** `src/inline/src/engine.rs` (lines 84-112)
*   **Mechanism:** Uses `tokio::spawn` to offload full-queue events to a disk buffer, ensuring the critical inference path never blocks.

### 2. ZK Attestation Performance (Table 6)
The **Attestor** (`src/attestor`) provides the benchmark harness to reproduce the paper's performance claims.
*   **Claim:** ~62ms Witness Generation, ~1.4s Total Proving (50k constraints).
*   **Run Benchmark:**
    ```bash
    cd src/attestor
    cargo run --release --bin bench_prove -- --constraints 50000 --samples 10
    ```

### 3. Merkle Inclusion Proofs (Appendix B.2)
The **Merkle Tree** implementation (`src/attestor/src/merkle.rs`) provides:
*   `MerkleTree::from_data()` - Construct tree from compliance records.
*   `generate_proof()` - Generate O(log n) inclusion proof.
*   `verify_proof()` - Verify a record's existence.

---

## 🛠️ Quickstart

### Prerequisites
*   **Rust**: Latest stable (`rustup update`)

### Run Tests
```bash
# Verify Inline Engine Logic
cd src/inline
cargo test

# Verify Cryptographic Components (Merkle + ZK)
cd ../attestor
cargo test
cargo run --release --bin bench_prove
```

---

## 📜 License
MIT
