# raw-tx-checker

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

Small command-line tool that reads a jammed hashable noun produced from a raw transaction and prints its Tip5 hash.

## Role in the Workspace

This is a binary crate. It is a debugging/inspection utility: given a `.jam` file containing the hashable noun of a raw transaction, it cues the noun, runs the `hash_hashable` Tip5 jet from `zkvm-jetpack`, and reports the resulting digest. It is useful for verifying transaction IDs and cross-checking hashing behavior outside the wallet/node.

## Key Components

- `main.rs` — parses a single `JAM_PATH` argument, reads and cues the jammed noun, computes the Tip5 digest via `hash_hashable`, decodes it into a `nockchain_types` `Hash`, and prints the per-limb hex values and the base58 form.

## Usage

```
cargo run --release --bin raw-tx-checker -- <JAM_PATH>
```

`<JAM_PATH>` is a file containing a jammed hashable noun produced from a raw transaction. Output is the Tip5 digest limbs in hex plus the base58 representation.
