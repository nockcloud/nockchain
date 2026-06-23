# nockapp-grpc-proto

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

The protobuf/gRPC schema crate for NockApp: it owns the `.proto` definitions, compiles them at build time with tonic/prost, and re-exports the generated message and service types plus conversions to and from native node types.

## Role in the Workspace

This is the build-time proto layer shared across the Nockchain gRPC stack. Its `build.rs` runs `protoc` (via `tonic-prost-build`) over every `.proto` under `proto/` and emits Rust code plus a file-descriptor set for reflection. The generated types are surfaced under `pb`, and hand-written conversion code maps protobuf primitives to `nockchain-types`/`nockchain-math` values. The `nockapp-grpc` crate (and downstream node binaries) depend on this crate for all wire types.

## Key Components

- `pb` — generated protobuf modules: `common::{v1,v2}`, `monitoring::v1`, `private::v1`, `public::{v1,v2}`, plus `FILE_DESCRIPTOR_SET` for gRPC reflection.
- `v1::convert` / `v2::convert` — `From`/`TryFrom` implementations converting between protobuf types (belts, points, hashes, names, spends, signatures, etc.) and native node types.
- `common` — shared helpers for the generated types.
- `proto/nockchain/**` — the source `.proto` files (common, monitoring, private, public) compiled by `build.rs`.

Building requires `protoc` on `PATH` or via the `PROTOC` env var (the Nix flake and Docker build provide it). Cargo features `server` (default) and `client` mirror the consuming crate's configuration.
