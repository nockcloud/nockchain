# equix-latency

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

A command-line microbenchmark that measures EquiX solve and verify latency.

## Role in the Workspace

`equix-latency` is a binary-only crate that repeatedly generates pseudo-random challenges, solves them with the `equix` proof-of-work library, and verifies the resulting solutions, reporting latency and attempt statistics. It is a standalone profiling/diagnostic tool for the EquiX runtime backends and has no other dependencies in the workspace.

## Key Components

- `run` — solves and verifies `equix` challenges across the configured number of samples
- `RuntimeChoice` — selects the EquiX runtime: `default`, `compile-only`, or `interpret-only`
- `Summary` / `DurationStats` / `U64Stats` — min/p50/p95/max/mean statistics for solve time, verify time, and attempt counts
- `fill_challenge` / `splitmix64` — deterministic challenge generation from a seed
- text and JSON output (`print_text` / `print_json`)

## Usage

```sh
cargo run -p equix-latency --release -- \
  [--samples N] [--challenge-bytes N] [--seed N] \
  [--runtime default|compile-only|interpret-only] [--json]
```
