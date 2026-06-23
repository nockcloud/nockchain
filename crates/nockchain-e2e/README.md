# nockchain-e2e

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

End-to-end test harness for Nockchain: runs multi-node YAML scenarios against real or containerized nockchain binaries and provides gRPC-driven sizing, sync, and peer-speedup diagnostics.

## Role in the Workspace

This crate is both a library and a binary. The library implements scenario execution and gRPC tooling against running nodes; the `nockchain-e2e` binary exposes a CLI to run scenarios and one-off measurement commands. It consumes the scenario schema from `nockchain-testkit`, talks to nodes via `nockapp-grpc-proto`/`tonic`, and can launch nodes directly or through Docker (`testcontainers`).

## Key Components

- `runner` — `run_scenario` and `RunOptions`; executes a `nockchain-testkit` scenario, managing node lifecycle, steps, and asserts.
- `node` — node process/container management used by the runner.
- `grpc` — gRPC client helpers, e.g. `wait_for_height`, `wait_for_demo_live`, `wait_for_seed_catch_up`.
- `sizing` — block/transaction size analysis: `build_fan_in_block_size_summary` and `collect_block_tx_distribution` (Gen2 batch cap pressure reporting).
- `peer_speedup` — `assert_peer_speedup` for verifying sync speedup across servers.
- `upgrade`, `report` — node upgrade handling and report data structures.
- `main.rs` (bin) — clap CLI dispatching the subcommands below.

## Usage

The bundled scenarios and fixtures live in [`e2e/`](./e2e/README.md)
(`e2e/scenarios/*.yaml`, `e2e/fixtures/`).

```
# Run a bundled scenario (defaults to target/release/nockchain)
cargo run --release --bin nockchain-e2e -- run crates/nockchain-e2e/e2e/scenarios/smoke.yaml

# Run against a Docker image instead of a local binary
cargo run --release --bin nockchain-e2e -- run crates/nockchain-e2e/e2e/scenarios/smoke.yaml --docker --docker-image <IMAGE>
```

Diagnostic subcommands: `block-size-report`, `assert-peer-speedup`, `wait-for-public-height`, `wait-for-demo-live` (alias `wait-for-review-ready`), `wait-for-seed-catch-up`, and `block-tx-distribution`. Run `nockchain-e2e --help` (or `<subcommand> --help`) for full flags.
