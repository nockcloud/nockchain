# Nockchain Documentation Map

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (navigation index; canonical authority is the Tier-0 spine, starting at [`START_HERE.md`](../START_HERE.md))

This is the single place to find any document in the repository. It is a map,
not an authority: trust policy and canonical sources are defined by the spine.
If anything here conflicts with a Tier-0 doc, the Tier-0 doc wins.

> New here? Read [`START_HERE.md`](../START_HERE.md) first. It defines the docs
> trust contract and the canonical read order.

## 1. Start Here / Canonical Spine (Tier 0)

| Doc                                                  | Purpose                                                     |
| ---------------------------------------------------- | ---------------------------------------------------------- |
| [`START_HERE.md`](../START_HERE.md)                  | Docs trust contract, read order, canonical tier policy     |
| [`PROTOCOL.md`](../PROTOCOL.md)                      | Protocol authority and upgrade index                       |
| [`ARCHITECTURE.md`](../ARCHITECTURE.md)              | System boundaries and global invariants                    |
| [`WORKFLOWS.md`](../WORKFLOWS.md)                    | Operational routing: pick the right golden path            |
| [`DECISIONS/README.md`](../DECISIONS/README.md)      | Architecture Decision Records (ADR) index                  |
| [`README.md`](../README.md)                          | Quickstart: setup, build, run, wallet, FAQ                 |

## 2. Protocol & Consensus

- [`PROTOCOL.md`](../PROTOCOL.md) — canonical index of every protocol upgrade.
- [`changelog/protocol/SPECIFICATION.md`](../changelog/protocol/SPECIFICATION.md) — required spec format and lifecycle.
- [`changelog/protocol/`](../changelog/protocol/) — per-upgrade specs (legacy checkpoints through Bythos, Nous, Aletheia).

## 3. Architecture

- [`ARCHITECTURE.md`](../ARCHITECTURE.md) — canonical boundaries and invariants.
- [`docs/architecture/`](./architecture/README.md) — explanatory deep dives.
  - [`tx-engine/`](./architecture/tx-engine/README.md) — 16-part transaction
    engine series (UTXO/note model, witness separation, lock Merkle proofs,
    validation pipeline, Tip5, Schnorr/Cheetah, Goldilocks field, STARK stack).

## 4. Runtime: NockVM & the Persistent Memory Arena (PMA)

- [`crates/nockvm/README.md`](../crates/nockvm/README.md) — the Nock virtual machine.
- [`crates/nockvm/DEVELOPERS.md`](../crates/nockvm/DEVELOPERS.md) — NockVM developer guide.
- [`crates/nockvm/docs/`](../crates/nockvm/docs/) — VM internals (stack, heap, persistence, b-trees, codegen, pills, LLVM).
- [`PMA-FAQ.md`](../PMA-FAQ.md) — operator FAQ for upgrading to a PMA-enabled release.
- [`docs/pma/`](./pma/README.md) — PMA design, durability operations, GC, and provenance notes.

## 5. Modules by Subsystem

Every crate has a README. The spine promotes a few of these to Tier-1 canonical
satellites (marked **Tier 1**); the rest are crate-level reference.

### Node & Consensus Core

| Crate                                                                       | What it is                                                          |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| [`nockchain`](../crates/nockchain/README.md)                                | The full-node binary: kernel + networking + mining + gRPC drivers  |
| [`nockchain-types`](../crates/nockchain-types/README.md)                    | Shared blockchain types and the versioned transaction engine       |
| [`nockchain-math`](../crates/nockchain-math/README.md)                      | Finite-field and crypto math primitives (Goldilocks, Tip5, zoon)   |
| [`zkvm-jetpack`](../crates/zkvm-jetpack/README.md)                          | Native Rust jets and proof-system forms for the zkVM               |
| [`nockchain-peek`](../crates/nockchain-peek/README.md)                      | CLI to query a running node's chain state over gRPC                 |

### Runtime, Compiler & Kernels

| Crate                                                                       | What it is                                                          |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| [`nockvm`](../crates/nockvm/README.md)                                      | The Nock VM and runtime (home of the PMA)                          |
| [`nockapp`](../crates/nockapp/README.md) **(Tier 1)**                       | NockApp runtime interface and kernel integration                   |
| [`nockapp-grpc`](../crates/nockapp-grpc/README.md)                          | gRPC servers/clients/drivers for NockApp and public node services  |
| [`nockapp-grpc-proto`](../crates/nockapp-grpc-proto/README.md)             | `.proto` schemas + generated protobuf types                        |
| [`hoonc`](../crates/hoonc/README.md)                                        | The Hoon compiler                                                  |
| [`hoon`](../crates/hoon/README.md)                                          | CLI to compile/run a single Hoon/Nock script                       |
| [`kernels`](../crates/kernels/README.md)                                    | Embeds prebuilt Hoon kernel `.jam` artifacts as constants          |
| [`chaff`](../crates/chaff/README.md)                                        | Experimental jam/cue noun serializer (`Jammer` impl)               |
| [`habit`](../crates/habit/README.md)                                        | Bit-level reader/writer primitives used by `chaff`                 |

### Networking

| Crate                                                                       | What it is                                                          |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| [`nockchain-libp2p-io`](../crates/nockchain-libp2p-io/README.md)            | libp2p networking driver (Kademlia/identify/request-response/QUIC) |

### Wallet, Transactions & Public API

| Crate                                                                       | What it is                                                          |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| [`nockchain-wallet`](../crates/nockchain-wallet/README.md) **(Tier 1)**     | Wallet CLI: keys, transaction construction, balances               |
| [`wallet-tx-builder`](../crates/wallet-tx-builder/README.md)               | Deterministic wallet transaction-planning engine                   |
| [`raw-tx-checker`](../crates/raw-tx-checker/README.md)                      | CLI to compute the Tip5 hash of a raw transaction                  |
| [`nockchain-api`](../crates/nockchain-api/README.md) **(Tier 1)**           | Public API runtime/deployment surface                              |

### Serialization

| Crate                                                                       | What it is                                                          |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| [`noun-serde`](../crates/noun-serde/README.md)                              | `NounEncode`/`NounDecode` traits for Rust <-> noun conversion      |
| [`noun-serde-derive`](../crates/noun-serde-derive/README.md)               | Derive macros for `NounEncode`/`NounDecode`                       |

### Bridge

| Crate / Doc                                                                  | What it is                                                          |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| [`bridge`](../crates/bridge/README.md)                                       | Cross-chain bridge crate                                           |
| [`bridge/docs/`](../crates/bridge/docs/README.md)                            | Bridge architecture, runbook, governance, signatures, withdrawals  |
| [`bridge-dev`](../crates/bridge-dev/README.md)                              | Dev CLI that boots a full local bridge stack for E2E testing       |
| [`nockchain-bridge-sequencer`](../crates/nockchain-bridge-sequencer/README.md) | Node colocated with the bridge withdrawal sequencer service     |

### Tooling & Developer Experience

| Crate                                                                       | What it is                                                          |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| [`nockup`](../crates/nockup/README.md)                                       | Developer support framework for NockApp development (+ templates)  |
| [`nockchain-explorer-tui`](../crates/nockchain-explorer-tui/README.md)      | Terminal UI block explorer over the gRPC API                       |
| [`equix-latency`](../crates/equix-latency/README.md)                        | EquiX solve/verify latency microbenchmark                          |

### Testing & Proof Harness

| Crate                                                                       | What it is                                                          |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| [`roswell`](../crates/roswell/README.md)                                    | NockApp proof/test harness (suites + proof generation/verification) |
| [`nockchain-testkit`](../crates/nockchain-testkit/README.md)               | YAML scenario schema for end-to-end tests                          |
| [`nockchain-e2e`](../crates/nockchain-e2e/README.md)                       | Multi-node E2E harness and diagnostics                             |

## 6. Operations & Runbooks

- Node setup, run, mine, wallet, logging, profiling, FAQ: [`README.md`](../README.md)
- Protocol upgrade awareness: [`PROTOCOL.md`](../PROTOCOL.md)
- Bridge operations: [`crates/bridge/docs/node-runbook.md`](../crates/bridge/docs/node-runbook.md)
- PMA durability operations: [`docs/pma/DURABILITY-OPERATIONS.md`](./pma/DURABILITY-OPERATIONS.md)
- Incident/debug triage routing: [`WORKFLOWS.md`](../WORKFLOWS.md)

## 7. Decisions (ADRs)

- [`DECISIONS/README.md`](../DECISIONS/README.md) — ADR index and process.

## 8. Fix Notes & History

- [`docs/fixes/`](./fixes/README.md) — engineering fix / hardening write-ups.
- [`crates/nockvm/docs/proposal/`](../crates/nockvm/docs/proposal/) and [`status/`](../crates/nockvm/docs/status/) — historical NockVM design notes.

## 9. Doc Governance

- Trust contract and promotion gate: [`START_HERE.md`](../START_HERE.md)
- Audit coverage tracker: [`DOC_INVENTORY.md`](../DOC_INVENTORY.md)
- CI doc checks: `make docs-check` (see [`scripts/docs/`](../scripts/docs/))
