# RuntimeGuard-AI

**RuntimeGuard-AI** is a reference implementation for cryptographically attested AI oversight, designed to align with the **EU AI Act (Article 14)** requirements for high-risk AI systems.

It demonstrates a scalable, asynchronous architecture that separates **lightweight inline policy enforcement** from **computational heavy zero-knowledge (ZK) attestation**, ensuring that compliance does not degrade inference latency.

## Architecture

The system is built on **Theorem 1: Latency Separation**, enabling the critical path to remain non-blocking regardless of the background attestation load.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        RuntimeGuard-AI Architecture                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐               │
│  │   AI Model   │───▶│   Inline     │───▶│   Response   │               │
│  │              │    │   Policy     │    │   to User    │               │
│  └──────────────┘    └──────┬───────┘    └──────────────┘               │
│                             │ (Non-blocking Send)                        │
│                             ▼                                            │
│                      ┌──────────────┐                                   │
│                      │ Shard Queue  │                                   │
│                      └──────┬───────┘                                   │
│                             │ (Async Drain)                              │
│         ┌───────────────────┼───────────────────┐                       │
│         │                   │                   │                       │
│  ┌──────▼──────┐    ┌──────▼──────┐    ┌──────▼──────┐                 │
│  │   Merkle    │    │     ZK      │    │  Oversight  │                 │
│  │   Log       │    │   Prover    │    │   Service   │                 │
│  └─────────────┘    └─────────────┘    └─────────────┘                 │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Core Components (Rust)

*   **`src/inline`**: The high-performance Inline Policy Engine.
    *   **Latency Separation**: Uses `tokio` channels and a fallback disk buffer to ensure `evaluate()` never blocks the inference thread.
    *   **Policy Enforcement**: Evaluates requests against compliance rules (Article 14).
*   **`src/attestor`**: The ZK Proving Service.
    *   **Batch Attestation**: Aggregates logs and generates Groth16 proofs.
    *   **Benchmarking**: Measured at **62ms** for witness generation and **1.4s** for total proving (50k constraints) on generic CPU.

## Quickstart

### Prerequisites
- **Rust**: Latest stable (`rustup update`)

### Running the ZK Benchmark
Validate the cryptographic performance on your machine:

```bash
cd src/attestor
cargo run --release --bin bench_prove -- --constraints 50000 --samples 5
```

### Running the Inline Engine Tests
Verify the non-blocking behavior:

```bash
cd src/inline
cargo test
```

## Repository Structure

*   `src/inline/` - Rust crate for the Inline Policy Engine.
*   `src/attestor/` - Rust crate for ZK Attestation & Benchmarks.
*   `src/oversight/` - Placeholder for Human-in-the-Loop oversight logic.
*   `paper/` - The USENIX Security 2026 research paper and materials.
*   `evaluation/` - Synthetic traces and analysis scripts.

## Publication

This code accompanies the paper:
**"RuntimeGuard-AI: Scalable Tamper-Evident Accountability for High-Risk AI Systems Under the EU AI Act"**
*Submitted to USENIX Security 2026*

See `paper/main.md` for the full manuscript.

## License
MIT
