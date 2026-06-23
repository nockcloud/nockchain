# nockchain-libp2p-io

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

The libp2p networking driver for Nockchain: it wires a configured libp2p swarm into the NockApp runtime, handling peer discovery, request/response message exchange, and connection gating between Nockchain nodes.

## Role in the Workspace

This crate is the network I/O layer for a Nockchain node. It builds the node's libp2p `Behaviour` (Kademlia, identify, ping, request-response over QUIC/TLS, peer store, connection limits) and exposes it as a NockApp `IODriverFn` so that the node kernel can send and receive blockchain messages over the wire. It depends on `nockapp`, `nockvm`, and `noun-serde` for kernel integration and noun (de)serialization, and is consumed by the `nockchain` node binary as its peer-to-peer transport.

## Key Components

- `driver` — builds and runs the Nockchain libp2p driver; `make_libp2p_driver` is the entry point, and `NockchainWire`/`Libp2pWire` describe the wire identifiers. Submodules cover gen1/gen2 request-response protocols (`driver/gen1`, `driver/gen2`), kernel I/O, and a connection watchdog.
- `config` — `LibP2PConfig` plus `from_env`/`from_env_or_default` constructors and protocol-version/timeout accessors that tune swarm, Kademlia, identify, and request-response behavior.
- `behaviour` — the composed Nockchain libp2p `NetworkBehaviour`.
- `catch_up` — catch-up sync-mode signalling (phase 1 of catch-up prefetch).
- `ip_block` — IP-level connection gating to deny banned addresses.
- `key_fair_queue` / `traffic_cop` — fair queueing and traffic prioritization for outbound work.
- `metrics` — gnort-based network metrics.
- `peer_stats` — read-only per-peer request/response statistics snapshots.
- `p2p_state` / `p2p_util` / `tip5_util` — driver state, helper utilities, and tip5<->string conversion.
- `test_support` (doc-hidden) — reusable request/response integration harness.
