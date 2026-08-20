# Contributing to RuntimeGuard-AI

Thanks for your interest. RuntimeGuard-AI is a research prototype with an
evidence-first culture: every public claim must map to code, tests, or a
canonical measurement. Contributions are held to the same bar.

## Getting started

Rust `1.92.0` is pinned by `rust-toolchain.toml`. The single verification
entrypoint is:

```bash
./reproduce.sh --verify
```

which runs `cargo fmt --check`, strict Clippy, all workspace tests,
`cargo audit`, and the Python tooling tests. CI runs the same gates plus the
clone-to-first-policy smoke test (`.github/workflows/verify.yml`).

## Ways to contribute

1. **Policies.** Share policy sources (`.rgp` files) under
   `examples/policies/` with a comment header describing the threat they
   address. See [`docs/writing-policies.md`](docs/writing-policies.md).
2. **Engine and attestor code.** Bug fixes and features in
   `src/inline`, `src/attestor`, and `src/cli`. Behavior changes need tests
   that fail without the change.
3. **Documentation.** Corrections and clarifications to `docs/` and the
   README, holding to the project's claim discipline (below).

## Claim discipline (non-negotiable)

- Do not add claims about zero-knowledge execution, model-inference
  correctness, transparency witnessing, anti-rollback under a compromised
  host, or regulatory conformity. `docs/protocol-v2.md` defines the boundary.
- Performance numbers in documentation must come from canonical runs under
  `results/` with an immutable run directory; quick runs are diagnostics.
- New protocol-visible fields require a schema-version discussion in the PR.

## Pull request checklist

- [ ] `./reproduce.sh --verify` passes locally.
- [ ] New behavior has a failing-first test.
- [ ] Public docs updated when observable behavior changed.
- [ ] No unexplained changes to `results/` or `paper/` artifacts.

## Security

If you find a vulnerability in the protocol implementation (signature
verification, recovery validation, lease handling, frame parsing), please do
not open a public issue. Email the maintainer via the contact on the GitHub
profile with reproduction details.
