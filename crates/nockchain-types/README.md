# nockchain-types

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

Shared Rust type definitions for the Nockchain blockchain: the transaction engine data model, blockchain constants, and Ethereum-compatible address types, all with noun encode/decode support.

## Role in the Workspace

This is a foundational library crate consumed by the node (`nockchain`), the prover/jet crate (`zkvm-jetpack`), and other tooling. It provides the Rust-side mirror of the kernel's data structures and constants so that host code can construct, encode, and decode the nouns exchanged with the Nock kernel. Types implement `NounEncode`/`NounDecode` (via `noun-serde`) and `serde` where relevant, and the crate depends on `nockchain-math` for field/hash primitives and `nockvm` for noun types.

## Key Components

- `tx_engine` (`src/tx_engine/`) — the transaction-engine type model, split into `common` (shared types such as `Hash` and page types), `v0`, and `v1` schema versions; re-exported from the crate root.
- `blockchain_constants` (`src/blockchain_constants.rs`) — network/consensus constants and helpers, including `fakenet_blockchain_constants(...)`, the `Seconds` newtype, and builder methods for fakenet phase/ASERT overrides.
- `eth` (`src/eth.rs`) — `EthAddress`, a 20-byte Ethereum-compatible address wrapper backed by `alloy`, with noun and hex conversions.

All three modules are re-exported via `pub use` from `src/lib.rs`, so consumers import directly from the crate root.

A separate README for the precomputed jam fixtures lives at `jams/README.md`.
