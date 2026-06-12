### find_jet runs a full formula traversal on every warm-table miss — warm.rs:664

This is almost certainly your main regression. On master, a warm miss was one HAMT lookup returning NoJet. On this branch, every miss falls through to normalize_transparent_hints (warm.rs:526), which recursively walks the entire arm formula stripping %hand/%hunk/%lose/%mean/%spot hints, then does a second HAMT lookup. Three compounding costs, paid on every Nock 9 dispatch of every unjetted arm (i.e., the vast majority of calls in hoonc):

- O(formula-size) tree walk per call, with no memoization — the same arm re-normalizes on every invocation.
- If the formula contains any transparent hint (hint-laden compiler code does, pervasively), the normalized copy is re-allocated on the NockStack every call via T(stack, …), churning the current interpreter frame.
- The second HAMT lookup mugs the freshly allocated normalized tree, so the mug cache never helps — another O(n) pass per call.

This turns the per-call dispatch overhead from ~O(1) into ~O(arm formula size), several times over. For big arms (+mint:ut etc.) that's thousands of nodes per call.

### Batteries::matches falls back to structural formula equality — cold.rs:278,464,488

battery_eq_ignoring_hints now runs nock_formula_eq_ignoring_transparent_hints whenever plain unification fails. That function calls unifying_equality at every recursion node, so a failed battery match is roughly O(n²) in battery size — and batteries are huge. Worse, the case it's designed for (hinted vs. hint-stripped battery variants) can never unify, so the full structural compare re-runs on every jet dispatch for such cores; it never gets cheaper. For hoonc this fires on warm-chain candidates that don't unify; for honk it's on the hot path by design. It also runs inside cold.register during %fast replay at boot.

### log_hint_event string allocation on every hint push/pop — interpreter.rs:1907,2061,2077,2149

Every %slog, and every %hand/%hunk/%lose/%mean/%spot push and pop now does UTF-8 validation plus two String allocations (atom_text + format!) before write_behavior_event_safe checks whether tracing is even enabled (trace_info is None in normal hoonc runs, but the strings are built unconditionally). Compiler code is dense with ~|/~_ mean hints, so this is a steady tax across the whole run.

### Warm::init normalizes and double-inserts every jet, per %fast registration — warm.rs:618

insert_with_transparent_hints runs normalize_transparent_hints on each registered arm formula and inserts the formula twice (original + normalized) when they differ. Warm::init already rebuilds the whole table on every successful cold.register, so boot replay is O(registrations × jets) — now with normalization and stack allocation added per entry, and warm chains are up to 2× longer, which feeds back into the per-call cost of #2. Mostly a boot/startup cost, which hoonc pays on every run.

### unifying_equality rework — unifying_equality.rs:229

The change you suspected, but it's a constant-factor cost, not the dominant one: EqualityWork is ~40 bytes vs master's 16-byte (*mut Noun, *mut Noun) (2.5× work-stack traffic), there's an extra FinishCell item pushed/popped per cell with unequal children, and two readonly-range checks per Compare (these short-circuit to nothing since replace_extra_noun_ptr_ranges has no production callers — the range machinery is dormant). I'd estimate 10–30% on equality-heavy paths in isolation — but note its call count went up substantially via #2, which calls it at every formula node.

### dor now does deep non-unifying equality — sort.rs:86, ext.rs:141

The head comparison in util::dor (reached from gor/mor mug ties and direct dor/sort use) falls back to noun_equality, which heap-allocates a Vec worklist and an IntMap per call and never unifies, so equal-but-unshared keys pay the full deep compare every time. This is a real correctness fix (master's raw_equals-only check could mis-order), so it likely needs to survive the rebase in some cheaper form (e.g., unifying equality, or no per-call allocs).

### Small per-op taxes

- op_budget check at the top of the interpreter loop, every work item (interpreter.rs:730) — well-predicted branch when None, ~1%.
- is_in_frame now does an explicit bounds check in release builds (mem.rs) where master had only debug asserts — small constant on every preserve/copy decision.
- Offsetting speedup: resolve_stack_ptr/classify_ptr got a release-mode identity fast path when no PMA is installed.

### Standard batch output recomputes directory mug by walking and reading the tree per entry

- Evidence: batch loops entries and calls jam_product at src/bin/honk.rs:1021-1045; standard jam_product calls exact_directory_mug at src/bin/honk.rs:2320-2324; directory_mug_with_files walks deps_dir, reads target and every valid file into memory at src/bin/honk.rs:3383-3435.
- Impact: O(entries × directory size) I/O and allocation.
- Fix: Cache directory manifests/content hashes per batch; avoid WalkDir when manifest file list is supplied.

### Hoon sources are read/scanned/parsed multiple times

- Evidence: content key reads bytes at src/bin/honk.rs:804-806, 1688-1691; import resolution reads source at pipeline.rs:221-224; leaf parse reads again at src/bin/honk.rs:1743-1786, 635-654.
- Impact: Avoidable I/O and parse overhead on every cache miss.
- Fix: Introduce SourceFile { canonical, text, hash, imports, ast } cache.

### Batch cache/slab lifetime is unbounded

- Evidence: shared NounSlab is leaked for build context at src/bin/honk.rs:848-849, 951-952; cache and content_cache store NativeVase nouns at src/bin/honk.rs:1121-1138, inserted at 1738-1739, never evicted in batch loop 1021-1048.
- Impact: Long batch compiles retain type/trap/eval graphs for the process lifetime.
- Fix: Split reusable prelude state from per-entry arenas; add cache budgets/eviction.

### Cached standard JAM is cloned before write

- Evidence: NativeBuildProduct.standard_jam: Option<Vec<u8>> at src/bin/honk.rs:1113-1118; jam_product returns jam.clone() at src/bin/honk.rs:2320-2323.
- Impact: Large artifacts are duplicated just to write/pad.
- Fix: Consume/take the Vec<u8> or write borrowed bytes.

### Stack guard abstraction is a no-op

- Evidence: redo_dext/redo_sint call with_stack_guard at ut/wet.rs:230-232, 301-309; with_stack_guard only increments a test counter and calls directly at ut/mod.rs:10035-10041; nearby comment claims stacker safety for mull but calls directly at ut/mod.rs:10055-10068.
- Impact: Deep nested types can still overflow Rust stack.
- Fix: Use stacker::maybe_grow in the guard or convert worst recursion to explicit stacks.

### Cache surfaces are scaffolded but disabled

- Evidence: cool_cache_lookup/store and chip_cache_lookup/store ignore inputs and return Ok(None)/Ok(()) at ut/mod.rs:3474-3505.
- Impact: Maintainers may believe hot-path memoization exists; runtime still pays lookup plumbing without hits.
- Fix: Implement bounded structural caches or delete the dead plumbing.

### AST-to-noun conversion is repeatedly materialized for cache keys

- Evidence: mull calls hoon_noun_for_node at ut/mod.rs:10055-10058; fallback always calls hoon_to_noun at ut/mod.rs:7743-7754; AST caching also clones AST/noun at ut/mod.rs:6734-6779.
- Impact: Repeated recursive allocation on hot semantic paths.
- Fix: Use stable AST signatures/pointer-guarded cache as primary keys; materialize noun once per parsed node only when required for parity.

### Wing/fond recursion allocates heavily

- Evidence: fund clones gen before reek at ut/find.rs:69-73; fond_name uses BigUint axes and cloned vein vectors through recursive descent at ut/find.rs:147-185, 263-299.
- Impact: Hot wing lookup pays repeated heap allocation.
- Fix: Keep axes as u64 until overflow, use push/pop vein stacks, and add borrowed wing extraction.
