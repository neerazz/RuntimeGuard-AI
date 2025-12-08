# RuntimeGuard-AI

**RuntimeGuard-AI** is the official reference implementation for the research paper:

> **"RuntimeGuard-AI: Scalable Tamper-Evident Accountability for High-Risk AI Systems Under the EU AI Act"**
> *Submitted to USENIX Security 2026*

**Repository:** [https://github.com/neerazz/RuntimeGuard-AI](https://github.com/neerazz/RuntimeGuard-AI)

This repository provides the cryptographic attestation architecture described in the paper, designed to meet **Article 14 (Human Oversight)** requirements without compromising inference latency.

## 📄 Research Paper

The full manuscript and supplementary materials are available in the `paper/` directory:

*   **[RuntimeGuard_AI_Paper.md](paper/RuntimeGuard_AI_Paper.md)**: The complete research paper (Markdown/LaTeX-ready).
*   **[main.md](paper/main.md)**: Draft manuscript with detailed evaluation summary.
*   **[research.md](paper/research.md)**: State-of-the-art analysis and novelty audit.

## 🏗️ Repository Structure

This project is strictly scoped to the artifacts described in the paper. Legacy experimental code has been removed.

```
runtimeguard-ai/
├── src/
│   ├── inline/      # [Rust] Inline Policy Engine (Paper Appendix B.1)
│   │                # Implements Theorem 1 (Latency Separation) via non-blocking fallback
│   │
│   └── attestor/    # [Rust] ZK Attestation Service (Paper Section 5.3)
│                    # Benchmarks for Groth16 witness generation & proving
│
├── paper/           # Research manuscript and figures
└── products/        # (Optional) Build artifacts
```

## 🚀 Key Claims Validation

### 1. Latency Separation (Theorem 1)
The **Inline Policy Engine** (`src/inline`) demonstrates how policy enforcement is decoupled from logging I/O.
*   **Code:** `src/inline/src/engine.rs`
*   **Mechanism:** Uses `tokio::spawn` to offload full-queue events to a disk buffer, ensuring the critical inference path never blocks.

### 2. ZK Attestation Performance (Table 6)
The **Attestor** (`src/attestor`) provides the benchmark harness to reproduce the paper's performance claims.
*   **Claim:** ~62ms Witness Generation, ~1.4s Total Proving (50k constraints).
*   **Run Benchmark:**
    ```bash
    cd src/attestor
    cargo run --release --bin bench_prove -- --constraints 50000 --samples 10
    ```

## 🛠️ Quickstart

### Prerequisites
*   **Rust**: Latest stable (`rustup update`)

### Run Tests
```bash
# Verify Inline Engine Logic
cd src/inline
cargo test

# Verify Cryptographic Benchmarks
cd ../attestor
cargo run --release --bin bench_prove
```

## 📜 License
MIT
