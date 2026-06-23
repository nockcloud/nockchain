# nockapp-grpc

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

The gRPC server and client implementation for NockApp, providing a modern RPC interface (replacing the old socket interface) for cross-language interaction with a running node.

## Role in the Workspace

This crate hosts the tonic-based gRPC services that front a NockApp node. It defines the private NockApp service (peeks/pokes for local clients) and the public Nockchain services (v1 and v2, including block-explorer endpoints), plus matching clients and NockApp `IODriverFn` drivers that bind the servers into the runtime. It builds on `nockapp-grpc-proto` for the generated protobuf types and wire conversions, uses `nockchain-libp2p-io`/`nockchain-types`/`nockchain-math` for node data, and is consumed by node binaries such as `nockchain-api`.

## Key Components

- `services::private_nockapp` — the private (local) NockApp gRPC service: `PrivateNockAppGrpcServer`, `PrivateNockAppGrpcClient`, and `grpc_server_driver`/`grpc_listener_driver` drivers.
- `services::public_nockchain::v1` — the public Nockchain v1 service: server, client, NockApp driver (`grpc_server_driver`, `grpc_listener_driver`), the `PublicNockchainEffect` type, an in-memory cache, and metrics.
- `services::public_nockchain::v2` — the public Nockchain v2 service, adding block-explorer endpoints, CORS, an IP blocklist, caching, and metrics.
- `driver` — backcompat re-export of the v1 public server/listener drivers.
- `wire_conversion` — conversions between protobuf messages and node/noun types.
- `v1` / `v2` — request/response pagination helpers for each API version.
- `error` — `NockAppGrpcError` and the crate `Result` alias.

Cargo features `server` (default) and `client` select which side(s) are compiled.
