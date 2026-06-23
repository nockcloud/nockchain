# nockchain-math

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

Core finite-field, polynomial, and cryptographic math primitives used by Nockchain's zero-knowledge proving stack and consensus code.

## Role in the Workspace

This is a foundational library crate providing the base arithmetic that higher layers build on. It is depended on by `nockchain-types` and `zkvm-jetpack` (which re-exports `based`), and indirectly by the `nockchain` node. Types implement noun encode/decode (via `noun-serde`) and are designed to interoperate with the Nock kernel's representations; many also support `rkyv` and `serde` serialization.

## Key Components

- `belt` (`src/belt.rs`) — `Belt`, the base field element over the Goldilocks prime `2^64 - 2^32 + 1`, with field arithmetic.
- `felt` (`src/felt.rs`) — `Felt`, a degree-3 extension field element (`[Belt; 3]`).
- `poly`, `bpoly`, `fpoly` (`src/poly.rs`, `src/bpoly.rs`, `src/fpoly.rs`) — polynomial types over the base and extension fields.
- `mary`, `shape`, `structs` (`src/mary.rs`, `src/shape.rs`, `src/structs.rs`) — matrix/array (`mary`) layouts and supporting data structures.
- `tip5` (`src/tip5/`) — the Tip5 hash function and sponge construction.
- `crypto` (`src/crypto/`) — `argon2` (password/PoW hashing) and `cheetah` (elliptic-curve) primitives.
- `zoon` (`src/zoon/`) — ordered map/set types (`zmap`, `zset`) and common helpers.
- `convert`, `noun_ext`, `owned_based_noun`, `handle` — conversions between Rust math types and nouns, plus helpers for allocating results into noun memory.
