# Candidate rebuild liveness

## Problem

After accepting a new tip, the miner rebuilt its next candidate by synchronously replaying every retained transaction before returning to the event loop. A backlog containing large multi-input transactions could therefore delay block, transaction, and solution handling for minutes. Repeated chain-timer ticks could also queue overlapping work behind the same slow event.

Large v1 transactions amplified the stall through repeated output normalization, serial signature verification, and validation work that had already been established for the exact raw transaction.

## Fix

The miner now publishes a valid candidate for the new tip immediately and refills retained transactions at a bounded rate. One-way preflight checks reject only work that is already provably impossible, and overlapping chain-timer ticks are coalesced rather than queued.

The retained work is made cheaper by grouping output construction by lock root, verifying independent signatures with bounded parallel speculation, and reusing exact raw-validation results while preserving all context-dependent validation.

## Safety

The patch does not change the persisted state schema or relax transaction validity. Signature results are consumed in canonical order, grouped output construction is checked against the legacy implementation, and the candidate preflight cannot approve a transaction. Regression coverage includes large multi-input transactions, invalid-signature ordering, retained-transaction refill, candidate capacity, and timer coalescing.
