# bridge-dev

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

Developer orchestration CLI for bringing up and driving a local Nockchain/Base bridge stack (nodes, bridge processes, and the withdrawal sequencer) for end-to-end testing.

## Role in the Workspace

`bridge-dev` is a binary-only crate that automates a multi-process bridge deployment against a local fakenet Nockchain and a Base (Optimism/Ethereum) test network. It boots and supervises the node, bridge, and sequencer processes, then offers commands to inspect status, stream logs, and trigger deposits, withdrawals, and Base block advancement. It builds on the `bridge`, `nockchain-types`, and `nockchain-math` crates and talks to running components over gRPC (`nockapp-grpc`/`tonic`) and Unix sockets. The opt-in scenario suite under `tests/` exercises full end-to-end flows (see [`tests/README.md`](tests/README.md)).

## Key Components

- `Commands` (clap subcommands) — `up`, `down`, `status`, `watch`, `wait`, `info`, `logs`, `stop`, `start`, `restart`, `deposit`, `mint-for-burn`, `request-withdrawal`, `advance-base`
- `ControlRequest` / `ControlResponse` — Unix-socket control protocol used to supervise individual components (status, stop/start/restart)
- `WaitCommand` — `deposit` / `withdrawal` polling helpers that block until a target phase is reached
- Profile loading — reads a bridge-dev profile (default `<bridge>/scripts/environments/bridge-dev.toml`) selected with `--profile`
- `BRIDGE_DEV_*` environment overrides — fakenet genesis/difficulty, port offsets, sequencer journal, and save-interval tuning

## Usage

```sh
# Boot a fresh local bridge stack
cargo run -p bridge-dev -- up --fresh

# Inspect status of nodes, bridges, and the sequencer
cargo run -p bridge-dev -- status --bridges --sequencer

# Trigger a deposit, then wait for it to succeed
cargo run -p bridge-dev -- deposit ...
cargo run -p bridge-dev -- wait deposit ...

# Tear everything down
cargo run -p bridge-dev -- down
```
