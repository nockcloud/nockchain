# nockchain-bridge-sequencer

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

A Nockchain node colocated with the bridge withdrawal sequencer service, responsible for ordering, submitting, and confirming bridge withdrawals from Base to Nockchain.

## Role in the Workspace

This binary crate wraps `nockchain::run_nockchain_app` and runs a withdrawal sequencer alongside it in the same process. It watches confirmed Base block height, verifies Base-side withdrawal/burn events, serves the withdrawal sequencer gRPC service, and drives the confirmation and orphan-retry loops that submit withdrawal transactions to the colocated public Nockchain node. Sequencer logic lives in the `bridge` crate (`bridge::withdrawal::sequencer::*`); this crate is the deployable entrypoint that wires it to a node, CLI/config/env, and an optional durable journal. A companion control binary, `nockchain-bridge-sequencer-ctl`, manages manual withdrawal approvals.

## Key Components

- `main` — boots the Nockchain app, the confirmed-Base-height watcher, and the withdrawal sequencer RPC/loops via `tokio::select!`
- `NockchainBridgeSequencerCli` — clap CLI flattening `nockchain::NockchainCli` plus sequencer flags (Base WS URL, confirmation depth, handoff/retry windows, sequencer config path)
- `build_sequencer_journal` — assembles an optional R2/S3-compatible durable journal from CLI flags, env vars, and sequencer config
- `start_withdrawal_sequencer` — opens the withdrawal-state SQLite store, recovers from the journal, and spawns the RPC, confirmation, and orphan-retry tasks
- `bin/nockchain-bridge-sequencer-ctl` — operator CLI: `pending-approvals`, `show-approval`, `export-tx`, `approve-withdrawal`, and related manual-approval commands
- Features: `jemalloc` (jemalloc allocator), `tracing-heap` (Tracy heap profiling)

## Usage

```sh
cargo run -p nockchain-bridge-sequencer -- \
  --bind-public-grpc-addr <addr> \
  --base-ws-url <wss-url> \
  --sequencer-config-path <path/to/sequencer.toml> \
  [nockchain node flags...]

# Operator tooling
cargo run -p nockchain-bridge-sequencer --bin nockchain-bridge-sequencer-ctl -- pending-approvals
```
