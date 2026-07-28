# Candidate rebuild liveness

## Problem

After accepting a new tip, the miner rebuilt its next candidate by synchronously replaying every retained transaction before returning to the event loop. A backlog containing large multi-input transactions could therefore delay block, transaction, and solution handling for minutes. Repeated chain-timer ticks could also queue overlapping work behind the same slow event.

Large v1 transactions amplified the stall through repeated output normalization, serial signature verification, and validation work that had already been established for the exact raw transaction.

## Fix

The miner now publishes a valid candidate for the new tip immediately and refills retained transactions at a bounded rate. One-way preflight checks reject only work that is already provably impossible, and overlapping chain-timer ticks are coalesced rather than queued. Admitted timer pokes carry a contiguous logical sequence, so coalescing a slow poke cannot repeatedly skip the same retained transactions.

The retained work is made cheaper by grouping output construction by lock root, verifying independent signatures with bounded parallel speculation, and reusing exact raw-validation results while preserving all context-dependent validation.

Pool adapters can derive multiple coinbase templates from one canonical candidate body with `restamp-candidate`. The arm preserves the selected transaction IDs and validated accumulator, changes only the v1 coinbase, and rejects a customer split that would make the resulting block exceed the consensus size limit.

Honk now allocates indirect atoms when a parsed or generated axis exceeds the direct-atom limit. Its native `++comb` implementation also composes axes with arbitrary-precision `peg`, matching the Hoon compiler instead of skipping the optimization for big axes.

## Safety

The patch does not change the persisted state schema or relax transaction validity. Signature results are consumed in canonical order, grouped output construction is checked against the legacy implementation, and the candidate preflight cannot approve a transaction. Regression coverage includes large multi-input transactions, invalid-signature ordering, retained-transaction refill, logical-tick fairness, candidate capacity, timer coalescing, coinbase-only restamping, and a 674-input v1 transaction that spends legacy v0 notes.

The legacy-note fixture is executed through the real Roswell kernel: the transaction is admitted to the mining candidate, customer and house coinbases are restamped, and the transaction body plus validated accumulator remain unchanged.
