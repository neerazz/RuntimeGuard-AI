# RuntimeGuard-AI

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18527375.svg)](https://doi.org/10.5281/zenodo.18527375)
[![GitHub release](https://img.shields.io/github/v/release/neerazz/RuntimeGuard-AI?label=release)](https://github.com/neerazz/RuntimeGuard-AI/releases/tag/v2.0.0)
[![arXiv](https://img.shields.io/badge/arXiv-2608.17176-b31b1b.svg)](https://arxiv.org/abs/2608.17176)
[![V2 paper](https://img.shields.io/badge/paper-local%20PDF-8b1e3f)](paper/v2/runtimeguard-v2.pdf)
[![ORCID](https://img.shields.io/badge/ORCID-0009--0002--2125--1805-a6ce39?logo=orcid)](https://orcid.org/0009-0002-2125-1805)
[![Conf42 LLMs 2026](https://img.shields.io/badge/invited%20talk-Conf42%20LLMs%202026-5b3fd6)](https://youtu.be/EelEvPhQLDk)

RuntimeGuard-AI V2 is a research prototype for policy-bound, crash-recoverable AI decision evidence. It synchronously commits a deterministic policy decision at an explicit durability boundary, returns an externally verifiable Ed25519 receipt, and later groups committed records into chained, signed Merkle epochs.

The implementation is intentionally narrow. It does not claim zero-knowledge policy execution, model-inference correctness, external transparency witnessing, compromised-host anti-rollback, or EU AI Act compliance.

## Publication history

The original peer-reviewed journal article is preserved as the immutable version of record at [DOI 10.5281/zenodo.18527375](https://doi.org/10.5281/zenodo.18527375). Its historical ZK, performance, and regulatory claims are not evidence for V2. The corrected successor paper, **“The Acknowledgment Point Is the System: Durable Policy-Decision Receipts for AI Audit Evidence,”** is public as [arXiv:2608.17176](https://arxiv.org/abs/2608.17176) ([DOI 10.48550/arXiv.2608.17176](https://doi.org/10.48550/arXiv.2608.17176)). The repository retains the exact [`paper/v2/runtimeguard-v2.pdf`](paper/v2/runtimeguard-v2.pdf), and its canonical evidence is [`results/v2/runtimeguard-v2-canonical-20260712-release`](results/v2/runtimeguard-v2-canonical-20260712-release).

Version 2 is a protocol and evidence-format break. See [`CHANGELOG.md`](CHANGELOG.md) for the exact additions, removals, and claim corrections.

## Recognition and public record

RuntimeGuard-AI is part of **Neeraj Kumar Singh Beshane's work as an Independent Researcher in AI security**. The project has moved through publication, public technical speaking, and a source- and evidence-backed V2 rebuild.

| Milestone | Public evidence |
|---|---|
| **Archived publication** | The original journal paper is preserved at [Zenodo DOI 10.5281/zenodo.18527375](https://doi.org/10.5281/zenodo.18527375). It remains the immutable historical version of record. |
| **Invited technical talk** | Neeraj delivered **“RuntimeGuard-AI: Cryptographic Compliance for High-Risk AI Pipelines”** at [Conf42 LLMs 2026](https://www.conf42.com/llms2026) on June 18, 2026. [Watch the recording](https://youtu.be/EelEvPhQLDk) · [Verify the speaker credential](https://certified.fyi/c/D6lQuWMq-byYEePKD-ZqnMXTeJ-HI8GdZj7). |
| **Public V2 paper** | [arXiv:2608.17176](https://arxiv.org/abs/2608.17176), **“The Acknowledgment Point Is the System,”** makes the corrected six-page successor publicly citable under CC BY 4.0. |
| **Research-grade successor** | [RuntimeGuard-AI V2.0.0](https://github.com/neerazz/RuntimeGuard-AI/releases/tag/v2.0.0) replaces unsupported prototype claims with tested protocol semantics, canonical measurements, and reproducible release artifacts. |
| **Researcher identity** | [Neeraj Kumar Singh Beshane — ORCID 0009-0002-2125-1805](https://orcid.org/0009-0002-2125-1805). |

The publication and talk are milestones worth preserving; V2 is also explicit about what the prototype did not establish. That correction is part of the research record, not a footnote to hide.

## Implemented V2 properties

- exact-source compiled policy identity and deterministic request/record commitments;
- framed checksummed shard logs with buffered, data-sync, and full-sync acknowledgement modes;
- an OS-backed exclusive writer lease, contiguous sequence recovery, and fail-stopped append errors;
- exact request replay idempotency and conflicting request-ID rejection;
- signed commit receipts verified against an independently supplied Ed25519 key;
- domain-separated SHA-256 Merkle trees with logical-size-bound inclusion proofs;
- signed epoch statements binding policy, sequence range, tree size, predecessor statement, and signer key ID;
- production-code benchmarks with immutable run directories, source hashes, raw observations, and data-derived figures.

The precise guarantee and threat boundaries are specified in [`docs/protocol-v2.md`](docs/protocol-v2.md).

## Use it with your own policies

RuntimeGuard is adoptable as-is: clone, write a policy file, and get signed,
crash-recoverable decision receipts without writing Rust. The
`runtimeguard` CLI wraps the same `InlinePolicyEngine` the paper measures.

```bash
cargo build --release -p runtimeguard-cli
./target/release/runtimeguard keygen --out rg-signing.key
./target/release/runtimeguard validate examples/policies/starter.rgp
./target/release/runtimeguard evaluate \
  --policy examples/policies/starter.rgp \
  --evidence-dir ./evidence \
  --signing-key rg-signing.key \
  --request-id demo-1 \
  --prompt "my SSN is 123-45-6789"
```

Policies are ordered `block`/`escalate` regex rules in a plain-text
`runtimeguard-policy/v1` file whose exact bytes are digest-pinned into every
record and receipt. [`examples/policies/`](examples/policies/) carries working
policies for a support assistant, a coding copilot, and RAG over untrusted
documents, each covered by realistic-prompt tests. The full authoring guide —
format, identity semantics, durability boundaries, and the library API — is
[`docs/writing-policies.md`](docs/writing-policies.md).

## Repository map

```text
src/inline/       policy evaluation, durable evidence log, commit receipts, benchmarks
src/attestor/     Merkle proofs, signed epochs, verification, benchmarks
src/cli/          runtimeguard CLI: keygen, validate, evaluate, verify-receipt
examples/policies/ worked policies for real deployments, with tests
experiments/v2/   preregistered runner, analysis, and reproducibility contract
docs/             public protocol specification
paper/v2/         successor figures-as-code and manuscript artifacts
results/v2/       canonical evidence only; quick/failed runs are excluded from releases
```

## Verify the implementation

Rust `1.92.0` is pinned by `rust-toolchain.toml`.

```bash
./reproduce.sh --verify
```

The entrypoint runs formatting, strict Clippy, all-target tests, dependency audit, Python tests, and script compilation. The equivalent commands are:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo audit
```

Run the non-canonical smoke matrix:

```bash
python3.12 -m venv .venv
.venv/bin/pip install -r experiments/v2/requirements.txt
./reproduce.sh --quick
```

See [`experiments/v2/README.md`](experiments/v2/README.md) before running or interpreting the full matrix. Quick and failed runs are pipeline diagnostics, never publication evidence.

## License

MIT
