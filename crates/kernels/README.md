# kernels

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

A library crate that embeds the prebuilt Hoon kernel `.jam` artifacts so other crates can load them as in-binary byte slices.

## Role in the Workspace

`kernels` exposes the compiled Nockchain kernels (jammed Hoon) as `&[u8]` constants by including the `.jam` files from the repo `assets/` directory at build time. Its `build.rs` resolves the asset paths and sets `DUMB_JAM_PATH`, `WALLET_JAM_PATH`, and `MINER_JAM_PATH` env vars (with `rerun-if-changed` tracking), and each module then `include_bytes!`s the corresponding jam. Modules are feature-gated so consumers pull in only the kernels they need. The crate has no Rust dependencies; its job is purely to bundle kernel artifacts. Sibling `kernels-*` crates (e.g. `kernels-open-dumb`, `kernels-open-wallet`, `kernels-open-miner`, `kernels-open-bridge`, `kernels-open-nockchain-peek`, `kernels-roswell`) follow the same pattern, each embedding a single kernel and supporting a `KERNEL_JAM_PATH` override plus a `bazel_build` feature.

## Key Components

- `dumb::KERNEL` (feature `dumb`) — bytes of `assets/dumb.jam`
- `wallet::KERNEL` (feature `wallet`) — bytes of `assets/wal.jam`
- `miner::KERNEL` (feature `miner`) — bytes of `assets/miner.jam`
- `build.rs` — sets `*_JAM_PATH` env vars from `assets/` and emits `rerun-if-changed` for each jam
- Features: `dumb`, `wallet`, `miner`, and `bazel_build`
