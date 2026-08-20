# Writing your own RuntimeGuard policy

RuntimeGuard binds every decision to the exact bytes of the policy source that
produced it. This page is the complete guide from `git clone` to a signed,
verifiable decision under a policy you wrote.

Nothing here changes the protocol: the guarantees and boundaries are exactly
those in [`docs/protocol-v2.md`](protocol-v2.md). The policy language is a thin
authoring layer over the same `CompiledPolicy` type the engine has always used.

## Quickstart (about five minutes)

Rust `1.92.0` is pinned by `rust-toolchain.toml`; `rustup` handles it
automatically.

```bash
git clone https://github.com/neerazz/RuntimeGuard-AI.git
cd RuntimeGuard-AI

# 1. Build the CLI.
cargo build --release -p runtimeguard-cli

# 2. Generate a receipt signing key. The printed verifying key is what an
#    auditor uses; distribute it out of band.
./target/release/runtimeguard keygen --out /tmp/rg-signing.key

# 3. Validate the starter policy (prints its pinned source digest).
./target/release/runtimeguard validate examples/policies/starter.rgp

# 4. Evaluate a prompt and commit a durable, signed evidence record.
./target/release/runtimeguard evaluate \
  --policy examples/policies/starter.rgp \
  --evidence-dir /tmp/rg-evidence \
  --signing-key /tmp/rg-signing.key \
  --request-id demo-1 \
  --prompt "my SSN is 123-45-6789" > /tmp/receipt.json

# 5. Verify the receipt with only the public verifying key.
echo "<verifying key hex printed by keygen>" > /tmp/rg-verifying.key
./target/release/runtimeguard verify-receipt \
  --receipt /tmp/receipt.json \
  --verifying-key /tmp/rg-verifying.key \
  --require-durable
```

Step 4 prints the decision (`Blocked` for the SSN example), the triggered rule,
and the Ed25519-signed commit receipt. Step 5 verifies that receipt with the
independently supplied key, exactly as the protocol's audit flow requires.

## The policy source format

A policy is a UTF-8 text file. Line one is the header; `id=` and `version=`
are required; every other non-comment line is a rule.

```text
runtimeguard-policy/v1
id=my-team-policy
version=1

# Comments and blank lines are ignored (but still part of the pinned digest).
block <rule-name> <regex>
escalate <rule-name> <regex>
```

Semantics:

- **Rules run in file order.** The first matching rule decides the outcome.
- **`block`** returns `Decision::Blocked` with zero confidence.
- **`escalate`** returns `Decision::Escalated` for human review.
- **No rule matching** returns `Decision::Allowed`.
- Rule names must be unique within a file; regexes use Rust
  [`regex`](https://docs.rs/regex) syntax (no backreferences or lookaround).

## Policy identity is the exact source bytes

`PolicyDescriptor.digest` is the SHA-256 of the exact file you loaded —
including comments and whitespace. Consequences you should design around:

1. **Any edit is a new policy.** Even changing a comment changes the digest.
   Bump `version=` when you change semantics, but know the digest changes
   regardless.
2. **The evidence directory pins the digest.** The first `evaluate` against a
   fresh `--evidence-dir` writes a manifest binding that directory to the
   policy digest, shard count, sync policy, and receipt verifying key.
   Re-opening with a modified policy fails closed with a manifest conflict.
   That is the tamper-evidence working as designed: one evidence directory
   per policy version.
3. **Keep policy sources in version control.** The digest in each committed
   record lets an auditor prove which source produced a decision — but only
   if you can produce that source. Treat `.rgp` files like code.

## Durability boundaries

`--sync` selects the acknowledgement boundary, mirroring
`SyncPolicy` in the engine:

| Flag | Meaning | Receipt `durable` |
|---|---|---|
| `none` | buffered append, fastest, loses the tail on crash | `false` |
| `data` (default) | fdatasync-equivalent before acknowledgement | `true` |
| `full` | fsync-equivalent before acknowledgement | `true` |

A caller that needs crash-surviving evidence must reject receipts with
`durable = false` — `verify-receipt --require-durable` enforces this.

## Using a custom policy from Rust

The CLI is optional. Library users compile a policy source directly:

```rust
use runtimeguard_inline::engine::{EngineConfig, InlinePolicyEngine};
use runtimeguard_inline::durable_log::SyncPolicy;
use runtimeguard_inline::policy::CompiledPolicy;

let source = std::fs::read("examples/policies/starter.rgp")?;
let engine = InlinePolicyEngine::open(EngineConfig {
    log_directory: "/var/lib/runtimeguard/evidence".into(),
    num_shards: 4,
    sync_policy: SyncPolicy::Data,
    policy: CompiledPolicy::from_source(&source)?,
    receipt_signing_key: signing_key,
})?;
```

`CompiledPolicy::example_v1()` remains available and unchanged; it is the
fixture used by the benchmarks and the published measurements.

## Worked examples

[`examples/policies/`](../examples/policies/) ships four policies for real
deployments — a support assistant, a coding copilot, RAG over untrusted
documents, and the quickstart starter — with a README explaining when to
block versus escalate. Each is covered by realistic-prompt tests in
`src/inline/tests/example_policies.rs`, including inputs that must not fire.
Test your own policies the same way: the false-positive cases are the ones
that decide whether operators leave the guard switched on.

## Sealing epochs

Epoch attestation is unchanged and lives in `runtimeguard-attestor`
(`seal_epoch`, `epoch_proof`, `verify_record_inclusion`). See
[`docs/protocol-v2.md`](protocol-v2.md) for the audit flow.

## What this does NOT give you

The scope limits of the protocol apply to your policies too. RuntimeGuard does
not prove model execution, prevent a compromised signer from forking history,
provide semantic understanding of prompts (rules are regexes, not classifiers),
or establish legal/regulatory conformity. Benchmarks published in the paper
were measured with the example fixture policy; your policy's evaluation cost
depends on your rule set and is yours to measure (`bench_inline` accepts the
same engine, so the harness is reusable).
