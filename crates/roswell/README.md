# roswell

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

NockApp-based proof and test harness for Nockchain: it boots the Roswell Hoon kernel and drives proof conformance, proof generation/assembly/verification, and Hoon test/benchmark suites.

## Role in the Workspace

This crate is both a library and a binary. The library wraps the `kernels-roswell` kernel in a `Roswell` host type that pokes named Hoon commands and decodes their effects; the `roswell` binary exposes those commands as CLI subcommands. It is the entry point referenced from the root README's "Roswell proof and test harness" section and is used both in CI (test/bench suites) and for generating and verifying zk proof artifacts.

## Key Components

- `Roswell` (lib) — boots the kernel (`boot`, `boot_with_hot_state`), sends commands (`roswell_command`, `poke_command`), and offers typed helpers: `test_puzzle`, `prove_puzzle`, `make_proof_snapshot`, `make_proof_stream_window`, `assemble_proof_stream`, `assemble_proof_continuation`, `check_proof`, `compute`, plus `peek_proof`/`peek_decode`.
- `RoswellCommand` (lib) — enum of all Hoon command tags (e.g. `test`, `test-ci`, `prove-puzzle`, `verify-proof`) and their `as_str` wire names.
- `CommandOutput` / `Effect` (lib) — decode kernel effects (`exit`, `file` read/write), determine success, and write proof artifact files.
- Helpers (lib) — `validate_puzzle_length` (power-of-two check), `make_tas`, `list_to_noun`, `cue_file_to_stack`, `proof_version_atom`.
- `main.rs` (bin) — clap CLI mapping subcommands to kernel commands.

## Usage

The kernel jam is built with `make assets/roswell.jam`. Common invocations (boot flags `--new --ephemeral` come from the shared NockApp boot CLI):

```
# Run the public CI suite
cargo run --release --bin roswell -- --new --ephemeral run-suite

# Generate a complete proof jam for the built-in puzzle (version 2, length 1)
cargo run --release --bin roswell -- --new --ephemeral prove-puzzle 2 1 --filename proof-v2-len1

# Verify a proof jam
cargo run --release --bin roswell -- --new --ephemeral check-proof --proof proof-v2-len1.jam
```

Other subcommands include `test <NAME>`, `test-verifier`, `bench-verifier`, `test-crypto`, `test-dumb`/`bench-dumb`, `test-wallet`/`test-wallet-shard`, `test-zoon`, `test-bridge`, `test-puzzle`, `make-proof-snapshot`, `make-proof-stream-window`, `assemble-proof-stream`, `assemble-proof-continuation`, `compute`, `bench-h-zoon`, and `dec-benchmark`. Run `roswell --help` for the full list.
