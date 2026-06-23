# nockchain-peek

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

A command-line tool for querying ("peeking" at) a running Nockchain node's state over gRPC, such as the heaviest block, individual blocks, and block pages.

## Role in the Workspace

This crate is both a binary and a thin library. It boots a small NockApp around the `nockchain-peek` kernel, connects to a node's private gRPC endpoint via `nockapp-grpc`, issues a single peek poke built from the chosen subcommand, and prints the result. It uses `zkvm-jetpack` for the prover hot state and `nockvm` for noun construction. It is a developer/operator inspection tool, not part of the consensus path.

## Key Components

- `main` (`src/main.rs`) — binary entrypoint; installs the rustls crypto provider, parses the CLI, and runs the peek.
- `NockchainPeekCli` (`src/lib.rs`) — clap CLI with a `--grpc-address` option (default `http://localhost:5555`) and a `command` subcommand.
- `PeekCommand` — the available peeks: `Heavy`, `Block <id>`, `Blocks`, `HeaviestBlock`, `HeavyN <page>`, `SmallBlocks`, and `CheckNotes <id>`; each maps to a kernel peek path via `PeekCommand::to_noun`.
- `init_with_kernel` — boots the NockApp, attaches the one-punch poke, markdown/file output, and gRPC listener drivers.

## Usage

```sh
# Show the heaviest block ID
cargo run --release --bin nockchain-peek -- heavy

# Inspect a specific block by base58 ID against a custom gRPC address
cargo run --release --bin nockchain-peek -- \
    --grpc-address http://localhost:5555 block <BLOCK_ID>

# Page lookup by height
cargo run --release --bin nockchain-peek -- heavy-n <PAGE_NUMBER>
```
