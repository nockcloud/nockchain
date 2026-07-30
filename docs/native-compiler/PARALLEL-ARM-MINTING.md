# Dependency-aware parallel arm minting

## Status

This campaign is quarantined on `codex/honk-parallel-arm-minting`. The deterministic planning work and profiling are valid, but Linux process-isolated arm execution is experimental and disabled unless `HONK_ARM_JOBS` is greater than one. No parallel configuration tested across the production Wallet, Roswell, and Dumbnet kernels produced a fleet-wide positive build-time yield, so parallel execution must not be enabled in production or CI by default.

## Determinism model

Arm map decoding, expected-goal derivation, and output topology are performed serially in canonical preorder. Expensive arm mint calls are represented by an indexed plan. Results are reduced into the battery in the original map topology and task order, independent of completion order. Linux workers receive copy-on-write compiler snapshots, return a JAM-encoded formula, and the parent cues each result into its own slab in canonical task order. A worker never shares `NounSlab`, `Rc`-backed type IR, lazy-resolver state, fan context, or memo tables with another worker. This prevents data races and completion-order-dependent noun allocation from changing output bytes.

## Dependency model

`HONK_ARM_PROFILE=1` records `HONK_ARM_GROUP_START`, `HONK_ARM_EDGE`, `HONK_ARM_TIME`, and `HONK_ARM_GROUP_END` events. Every arm completion reports inclusive and exclusive time. Nested lazy-resolver mint time is charged to the parent's child total, so the scheduler can distinguish an independently expensive arm from a cheap arm waiting on an expensive semantic dependency. The existing AST signature walk also records whether an arm contains a `^~` fold point, which is used as a conservative first-seen dependency-risk signal.

## Production profile

The accepted Dumbnet artifact was `2b5b0f77937f5162ed0e6c8f8ebd2651761576798f79dfa95a079d490423bb01`. The profiled build contained 8,923 arm mint completions and 416 dynamic lazy-resolver edges. One nested `$` arm consumed 45.703 seconds of exclusive time, and `first-from-hash` consumed 5.493 seconds. The top enclosing 55-arm battery took 55.880 seconds, but one task accounted for 52.103 seconds inclusive. This establishes a hard critical-path limit: broad sibling parallelism cannot remove the dominant single-arm semantic work.

## Benchmark results

All completed candidates reproduced the accepted artifact hashes exactly. Dumbnet jobs=1 clean runs were 84.35s, 83.34s, and 84.82s before later scheduler experiments, averaging 84.17s. Inclusive-cost scheduling at four jobs and a 25ms threshold regressed to 94.61s. A 100ms inclusive threshold appeared positive on Dumbnet at 80.69s and 79.87s, but regressed Wallet from 92.99s to 97.43s and Roswell from 140.90s to 153.28s; it duplicated nested resolver work and is rejected. Two jobs at that threshold took 95.88s and eight jobs took 95.17s. Unbounded first-seen speculation was stopped after exceeding three minutes and multiplying long dependency work. An early bounded-speculation run of eight tasks in batteries with at least 48 arms launched 32 workers and took 97.03s, also rejected. The exact final source reproduced the accepted serial hashes in 87.92s for Wallet, 128.53s for Roswell, and 85.86s for Dumbnet. Forced four-worker bounded speculation remained byte-identical where completed but regressed Wallet to 89.37s and Dumbnet to 86.95s; Roswell was terminated after exceeding four minutes, roughly 1.9 times its adjacent serial duration.

## Controls

`HONK_ARM_JOBS=N` enables Linux process-isolated execution and runs the CLI on a single OS thread with an unconditional large stacker fiber before any fork. `HONK_ARM_PARALLEL_MIN_COST_US=N` sets the minimum previously observed exclusive cost, defaulting to 100,000 microseconds. `HONK_ARM_PARALLEL_SPECULATE_ARMS=N` enables first-seen speculation only for batteries at least that wide; the default is `usize::MAX`, which disables speculation. `HONK_ARM_PARALLEL_SPECULATE_LIMIT=N` caps speculative tasks per battery and defaults to eight. `HONK_ARM_PROFILE=1` emits the dependency and timing trace. These controls are experimental diagnostics, not recommended build settings.

## Why this did not win

The current compiler's apparent arm-level width is mostly nested semantic dependency work. Lazy resolver requests need formulas synchronously, and `Ut` owns mutable slab pointers, non-`Send` `Rc` type nodes, recursion guards, fan scope, evaluator state, and context-sensitive memos. Forking preserves correctness but loses cross-arm memo sharing and pays page-table, allocator copy-on-write, JAM, cue, and process scheduling costs. The measured independent work is too small and too irregular to amortize those costs across the production kernel set.

## Viable follow-up

The next principled parallel attempt should wait for a frozen worker input boundary: an owned `Send + Sync` type/formula DAG, canonical resolver descriptors, immutable Hoon arena IDs, and worker-local memo/evaluator state. Dependency discovery should produce strongly connected components, schedule only ready components, and return owned formula IR rather than JAM bytes. Canonical reduction must remain serial and indexed. That design can use a persistent thread pool and preserve shared immutable interning without process isolation; it is the credible route to a larger parallel yield.
