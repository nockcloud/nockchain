# zkvm-jetpack

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

Native (jet) accelerations and supporting forms for Nockchain's zero-knowledge VM proving and verification, exposed as a hot-state table that the Nock runtime uses to replace Hoon code with fast Rust implementations.

## Role in the Workspace

This library crate supplies the prover/verifier "jets" and the proof-system forms used when running the Nockchain kernel. The node (`nockchain`) and the `nockchain-peek` tool both call `produce_prover_hot_state()` to obtain the `Vec<HotEntry>` registered with the Nock VM. It builds on `nockchain-math` (field/poly/Tip5/curve primitives) and `nockchain-types`, and uses `rayon` for parallelism. It re-exports `nockchain_math::based`.

## Key Components

- `hot` (`src/hot.rs`) — `produce_prover_hot_state()` assembles every jet group (base field/poly, curve, Tip5, NTT, table generation, verifier, base58, zoon, etc.) into the hot-state table consumed by the runtime.
- `jets` (`src/jets/`) — the Rust jet implementations, grouped by concern: field arithmetic (`base_jets`, `fext_jets`, `bp_jets`, `fp_jets`), NTT (`ntt_jets`, `fpntt_jets`), curves (`cheetah_jets`, `ec_point_jets`), hashing (`tip5_jets`, `tip5_sponge`, `crypto_jets`), proof and trace generation (`proof_gen_jets`, `trace_gen_jets`, `compute_table_jets_v2`, `memory_table_jets_v2`), verification (`verifier_jets`), and utilities (`mary_jets`, `shape_jets`, `base58_jets`, `zoon_jets`, `mega_jets`).
- `form` (`src/form/`) — the proof-system building blocks: `proof`, `verify`, `verifier_math`, `merk` (Merkle), `challenges`, `mega`, `preprocess`, `term`, `tog`, `config`, and a `math` submodule.
- `utils` (`src/utils.rs`) — shared helpers used across jets and forms.
