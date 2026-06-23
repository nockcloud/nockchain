# Architecture Deep Dives

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (explanatory deep dives; canonical boundaries live in [`ARCHITECTURE.md`](../../ARCHITECTURE.md))

Long-form, explanatory architecture documentation. For canonical system
boundaries and invariants, start with the spine doc
[`ARCHITECTURE.md`](../../ARCHITECTURE.md); these pages expand on the *how*.

## Contents

- [`tx-engine/`](./tx-engine/README.md) — a 16-part deep dive into the
  transaction engine: UTXO/note model, witness separation, lock Merkle proofs,
  validation pipeline, and the cryptographic and data-structure primitives
  (Tip5, Schnorr/Cheetah, Goldilocks field, Merkle commitments, the STARK
  proof stack).

## Related

- Runtime (NockVM) and PMA internals: [`crates/nockvm/`](../../crates/nockvm/README.md) and [`docs/pma/`](../pma/README.md)
- Bridge subsystem architecture: [`crates/bridge/docs/architecture.md`](../../crates/bridge/docs/architecture.md)
- Full documentation map: [`docs/README.md`](../README.md)
