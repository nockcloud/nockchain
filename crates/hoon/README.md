# hoon

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

A command-line tool for compiling and executing a Hoon/Nock script against the Nockchain runtime.

## Role in the Workspace

`hoon` is both a library and a binary. The binary parses CLI arguments, initializes tracing, produces the prover hot state, and runs the supplied Nock script; the library exposes `HoonCli` and `run`, which set up a `nockvm` interpreter `Context` and use `hoonc`'s `kick_and_save_generator` to compile/evaluate the script and optionally persist the kicked jam. It depends on `hoonc`, `nockapp`, `nockvm`, and `zkvm-jetpack`. This is the developer-facing entry point referenced in the workspace root README for running Hoon code.

## Key Components

- `HoonCli` — clap CLI: positional `nock_script` and `dep_dir`, optional `--out-dir`, plus flattened `nockapp` boot flags
- `run` — initializes the interpreter context (with `URBIT_HOT_STATE` plus extra hot state) and invokes `hoonc::kick_and_save_generator`
- `init_context` — builds a `nockvm` `Context` with a fresh `NockStack`, cold state, and optional trace info
- `main` (binary) — boots tracing, produces prover hot state, and calls `hoon::run`

## Usage

```sh
cargo run -p hoon -- <nock_script> <dep_dir> [--out-dir <dir>]
```

If `--out-dir` is not provided, the kicked jam output is not saved.
