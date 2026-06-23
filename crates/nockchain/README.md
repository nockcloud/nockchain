# nockchain

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

The main Nockchain full-node binary. It boots a NockApp around the Nockchain kernel, wires up the libp2p networking, mining, tracing, and gRPC drivers, and runs the node event loop.

## Role in the Workspace

This crate is the primary executable users run to participate in the network. It assembles a `NockApp` from a precompiled kernel jam (`kernels-open-dumb` / `kernels-open-miner`) plus the prover hot state from `zkvm-jetpack`, then attaches I/O drivers for peer-to-peer sync (`nockchain-libp2p-io`), in-kernel mining, trace export, and public/private gRPC servers (`nockapp-grpc`). It depends on `nockchain-types`, `nockchain-math`, `zkvm-jetpack`, and `nockapp`. It also exposes a library surface (`lib.rs`) so test/benchmark binaries and embedders can boot a node programmatically.

## Key Components

- `main` (`src/main.rs`) — binary entrypoint; parses the CLI, installs the optional jemalloc/Tracy allocator, produces the prover hot state, and runs the node.
- `init_with_kernel` (`src/lib.rs`) — boots a `NockApp` from a kernel jam, loads/persists the libp2p identity, applies connection/memory limits, configures fakenet vs realnet genesis, and registers all I/O drivers.
- `run_nockchain_app` — convenience wrapper that boots and runs the node with a given `NockchainAPIConfig`.
- `NockchainAPIConfig` — toggles whether the public gRPC server is enabled and on which address.
- `driver_init::DriverInitSignals` — coordinates per-driver initialization so the "born" poke is sent only after drivers (mining, libp2p) are ready.
- `config::NockchainCli` (`src/config.rs`) — clap-based command-line argument surface, including fakenet overrides, peer/bind options, and gRPC addresses.
- `mining` (`src/mining.rs`) — in-kernel mining driver and mining-key/PKH config.
- `backbone` (`src/backbone.rs`) — built-in realnet backbone peer set used for initial dialing.
- `setup`, `traces`, `colors` — genesis/setup pokes, trace export driver, and terminal banner output.

## Usage

```sh
# Build and run a node
cargo run --release --bin nockchain -- --help

# Enable mining (in-kernel) on the default network
cargo run --release --bin nockchain -- --mine

# Run a local fakenet node
cargo run --release --bin nockchain -- --fakenet
```

The crate also ships developer benchmark/utility binaries under `src/bin/`
(`bench_dumb_validation`, `bench_nockchain_kernel`, `bench_nockchain_checkpoint_block`,
`vet_chain_signatures`). Optional features: `jemalloc`, `tracing-heap`, `bazel_build`.
