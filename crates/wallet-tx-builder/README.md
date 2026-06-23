# wallet-tx-builder

Status: Active
Owner: Nockchain Maintainers
Last Reviewed: 2026-06-23
Canonical/Legacy: Legacy (crate-level reference; canonical docs spine starts at [`START_HERE.md`](../../START_HERE.md))

Deterministic transaction-planning library for the Nockchain wallet: it normalizes balance snapshots, selects candidate notes, resolves spend locks, estimates witness/seed word counts and fees, and produces a transaction plan.

## Role in the Workspace

This is a library crate (no binary). It provides the host-side planning logic that turns a consistent wallet balance view plus a recipient/withdrawal request into a `PlanResult` describing which notes to spend, the outputs to create, and the fee. It works against `nockchain-types` transaction structures (v0 and v1 notes, locks, spend conditions) and is designed to be deterministic so the same inputs always yield the same plan.

## Key Components

- `adapter` — normalizes paged balance snapshots into a `NormalizedSnapshot` of deduplicated `CandidateNote`s, with consistency checks (`SnapshotConsistencyError`, `CandidateNormalizationError`).
- `planner` — core planning entry points `plan_create_tx` and `plan_withdrawal_tx` (both generic over a `LockMatcher`), candidate selection/admission, and conservation checks; errors via `PlanError`.
- `types` — request/result and selection types: `PlanRequest`, `PlanResult`, `WithdrawalPlanRequest/Result`, `SelectionMode`, `SelectionOrder`, `CandidateVersionPolicy`, `ChainContext`, `CandidateNote`, `PlannedOutput`.
- `lock_resolver` — `LockMatcher` trait and `LockResolution`/`LockResolutionSource` for resolving each note's effective spend condition (note-data, lock-root first-name, reconstructed PKH, etc.).
- `note_data` — encodes/decodes typed note-data entries (`%lock`, bridge deposit/withdrawal) to and from nouns.
- `fee` — minimum-fee and bridge-fee computation (`compute_minimum_fee`, `compute_bridge_fee`, `FeeInputs`, `FeeBreakdown`).
- `word_count` — `WordCountEstimator` for seed/witness word counts used as fee inputs.
- `determinism` — stable ordering helpers (lexical name keys, candidate sorting, spend-condition canonicalization) that keep planning reproducible.
