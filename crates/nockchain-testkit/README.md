# nockchain-testkit

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

Library defining the YAML scenario schema for Nockchain end-to-end tests: node specs, step actions, and assertions, with loading and validation.

## Role in the Workspace

This is a library crate (no binary). It is the shared data model consumed by the `nockchain-e2e` runner, which executes the scenarios these types describe. It deserializes scenario YAML (via serde/serde_yaml) into strongly typed structures and validates basic invariants (non-empty name, unique node ids) before a run begins.

## Key Components

- `scenario::Scenario` — top-level scenario: name, seed, optional protocol version, binaries map, `nodes`, `steps`, and `asserts`; `load_from_path` reads and validates a YAML file.
- `scenario::NodeSpec` — per-node configuration (gRPC ports/addresses, data dir, fakenet/mining flags, peers and `peer_from`, env, extra args, binary override).
- `scenario::Action` — step variants such as `StartNodes`, `StopNodes`, `WaitForGrpc`, `WaitForHeight`, `WaitForHeadsEqual`, `WaitForTxAccepted`/`InBlock`, `SubmitTx`, `InjectBlock`, `Partition`, `Upgrade`, `Wallet`, `CloneWallet`, and `Command`.
- `scenario::Assert` — assertions such as `GrpcReady`, `HeadsEqual`/`HeadsNotEqual`, `HeightAtLeast`, `TxAccepted`/`TxInBlock`/`TxNotAccepted`, and `ReqResGeneration`.
- `scenario::WalletCapture` / `WalletCaptureSource` / `SubmitTxExpect` / `PeerFrom` / `ReqResGenerationExpectation` — supporting enums and structs for steps and asserts.
- `error::TestkitError` — error type for invalid scenarios and YAML/IO failures.
