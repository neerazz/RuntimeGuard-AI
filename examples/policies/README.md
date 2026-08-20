# Example policies

Four policies covering deployments people actually run. Each is a working
starting point, not a finished control: read the rules, keep what fits your
system, and delete what does not.

Every rule here is exercised against realistic prompts in
[`src/inline/tests/example_policies.rs`](../../src/inline/tests/example_policies.rs),
including inputs that must **not** fire. Copy that pattern for your own
policies — a guard nobody trusts gets turned off.

| Policy | Deployment | Posture |
|---|---|---|
| [`starter.rgp`](starter.rgp) | Quickstart walkthrough | Minimal, three rules |
| [`support-agent-pii.rgp`](support-agent-pii.rgp) | Support assistant forwarding ticket text to a hosted model | Block identifiers, escalate account changes |
| [`engineering-copilot-secrets.rgp`](engineering-copilot-secrets.rgp) | Coding assistant with repo context | Block credential shapes, escalate env/DSN lines |
| [`rag-untrusted-content.rgp`](rag-untrusted-content.rgp) | RAG over user-uploaded documents | Escalate only — humans decide |

## Choosing block versus escalate

The split is the whole design, and it is about false positives.

**Block** when the matched string is self-evidently the thing: an AWS key ID in
`AKIA…` form is a credential, and no legitimate prompt needs to carry it. A
false positive costs one rejected prompt.

**Escalate** when the phrase is ambiguous in context. `rag-untrusted-content.rgp`
blocks nothing, because a security runbook legitimately discusses prompt
injection and a changelog legitimately quotes old instructions. Blocking those
produces noise, and operators respond to noise by disabling the guard — which
is strictly worse than routing to a human.

## Using one

```bash
runtimeguard validate examples/policies/support-agent-pii.rgp
runtimeguard evaluate \
  --policy examples/policies/support-agent-pii.rgp \
  --evidence-dir ./evidence \
  --signing-key rg-signing.key \
  --request-id ticket-88231 \
  --prompt "Customer says: my SSN is 123-45-6789"
```

The evidence directory pins the policy's source digest on first use. Editing
the policy — including its comments — is a new policy identity and needs a new
evidence directory. See [`docs/writing-policies.md`](../../docs/writing-policies.md).

## Known limits

These are regex rules, not classifiers. They match **shape**, not meaning:

- The card rule checks issuer prefixes and grouping, not a Luhn checksum, so it
  both over-matches some digit runs and misses unusual issuer ranges.
- Secret detection covers documented credential formats. A bespoke internal
  token format needs a rule you write.
- Injection phrasing is unbounded. Escalation rules catch common shapes and
  will miss novel ones; they are a tripwire, not a boundary.

RuntimeGuard's guarantee is about the *evidence* — that the decision was bound
to an exact policy, committed durably, and is externally verifiable. It is not
a claim that the policy caught everything.
