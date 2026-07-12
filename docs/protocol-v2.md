# RuntimeGuard-AI V2 Protocol

Status: research prototype specification for the V2 implementation. This document does not modify or supersede the publication of record at DOI `10.5281/zenodo.18527375`.

## Scope

V2 implements one narrow claim: a deterministic policy decision can be bound to an explicit durable commit receipt, recovered after restart, and later included in a signed, policy-bound Merkle epoch that an auditor verifies using an externally supplied Ed25519 trust anchor.

V2 does not implement or claim zero-knowledge policy compliance, external transparency-log witnessing, compromised-host anti-rollback, cross-machine consensus, distributed exactly-once processing, key revocation, HSM isolation, or legal/regulatory compliance.

## Trust and threat boundary

Trusted for the stated guarantees:

- the compiled policy source loaded at engine creation;
- the OS/filesystem semantics used by `sync_data` or `sync_all`;
- the receipt and epoch signing keys while they remain uncompromised;
- an auditor's independently obtained Ed25519 verifying keys;
- clients retaining signed commit receipts that they may later challenge.

In scope:

- process interruption and restart;
- incomplete tail writes;
- complete-frame corruption;
- request replay and conflicting reuse of a request identifier;
- record reordering, duplication, and sequence gaps during recovery;
- policy, record, epoch-statement, inclusion-proof, and signature tampering;
- self-signed attacker epochs presented under an untrusted key.

Out of scope:

- a root adversary that can replace program code, keys, logs, and trusted verifier configuration;
- rollback to an older but internally valid filesystem snapshot;
- a signer that deliberately forks epoch history without any external comparison or witness;
- bypasses around the guarded call site;
- durability stronger than the host OS and storage device actually provide.

## Synchronous commit protocol

One engine instance holds an OS-backed exclusive writer lease on the evidence directory. A second process or engine instance cannot open that directory for writing until the lease is released. Within the writer, each request follows these steps under a process-local commit mutex:

1. Compute `request_commitment = SHA-256("runtimeguard/request/v2" || canonical_request_fields)`.
2. Look up the request identifier in the recovered idempotency index.
   - Same identifier and same commitment: return the original decision and a deterministically reconstructed signed receipt without appending a second record.
   - Same identifier and a different commitment: reject with a request-ID conflict.
3. Evaluate the compiled policy. Its descriptor digest is derived from the exact policy source bytes and is pinned in the engine manifest.
4. Allocate the next contiguous global sequence and assign `shard = sequence mod K`.
5. Construct a V2 compliance record and its protocol-stable binary commitment. The framed JSON is only a local storage encoding and is not the cryptographic canonicalization.
6. Append one checksummed `RGL2` frame to the selected shard.
7. Apply the configured acknowledgement boundary:
   - `None`: buffered append; the returned receipt has `durable = false`.
   - `Data`: `sync_data`; the returned receipt has `durable = true` under host semantics.
   - `Full`: `sync_all`; the returned receipt has `durable = true` under host semantics.
8. Sign a V2 commit receipt binding the request identifier, request commitment, record commitment, sequence, durability bit, and receipt verifying key.
9. Advance the in-memory sequence/index and return the policy result plus signed receipt.

A caller that requires crash-surviving evidence must reject receipts with `durable = false`. RuntimeGuard does not describe the buffered mode as durable.

If append or synchronization fails, the engine enters a fail-stopped state and rejects subsequent evaluations. A restart is required after the storage problem is repaired.

## Recovery invariants

At open, RuntimeGuard:

- checks the manifest schema, `RGL2` format, shard count, sync policy, policy digest, and receipt verifying key;
- acquires the evidence-directory writer lease before opening any shard;
- validates every frame length, payload checksum, JSON record, and record commitment;
- truncates only an incomplete tail frame;
- rejects corruption in any complete frame;
- requires each record to reside in `sequence mod K`;
- rejects duplicate request identifiers;
- requires recovered global sequences to be exactly the contiguous prefix `0..N-1`.

These checks provide process-restart recovery and detect local omission/reordering within the retained evidence set. They do not detect rollback of the whole directory to an earlier valid prefix.

## Asynchronous epoch attestation

Attestation operates on recovered committed records and is outside the guarded decision's synchronous path.

An epoch:

- contains a non-empty, contiguous sequence range;
- contains records from exactly one policy descriptor;
- commits the ordered record commitments in a SHA-256 Merkle tree with separate leaf and internal-node domains;
- signs the logical record count, first/last sequence, first/last evaluation time, policy identity/version/digest, Merkle root, previous statement hash, and signer key identifier;
- uses duplicate-last handling for odd tree levels;
- exposes proofs that bind the record's sequence-derived index and the signed logical tree size.

The first epoch uses an all-zero previous statement hash. Every later epoch includes the hash of the previous signed statement. An auditor must additionally require `next.first_sequence = previous.last_sequence + 1` when validating a chain.

## Audit flow

1. Obtain trusted receipt and epoch verifying keys independently of the artifact being checked.
2. Verify the client's commit receipt and require `durable = true` when durability is part of the claim.
3. Match its record commitment to a recovered compliance record.
4. Verify the epoch signature under the trusted epoch key.
5. Verify policy equality, sequence-to-index mapping, signed logical tree size, and Merkle inclusion.
6. Verify epoch-to-epoch statement hashes and contiguous sequence ranges.
7. Compare observed epoch heads or use an external witness if fork/equivocation resistance is required.

A retained signed receipt creates a concrete omission challenge: the operator must produce a matching record and inclusion proof or fail the challenge. V2 does not provide a public gossip protocol or independent witness, so it must not claim global omission or equivocation prevention.

## Key lifecycle

The manifest pins the receipt verifying key, and each epoch carries a key identifier derived from its Ed25519 verifying key. V2 requires the verifier to supply trusted keys externally. Rotation, revocation, HSM-backed signing, and public key transparency are future work; the key identifier alone is not a trust mechanism.

## Evidence and claim rules

- Benchmarks must import the production modules above.
- Quick runs validate only the pipeline and are never publication evidence.
- Canonical runs are immutable, seeded, environment-described, and analyzed from committed raw CSV files.
- Buffered, data-sync, and full-sync modes are reported separately.
- Component benchmarks support only component-cost claims; they do not establish deployment-scale SLOs.
- EU AI Act Articles 12, 14, and 26 are regulatory context. RuntimeGuard does not establish conformity with them.
