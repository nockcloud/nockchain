### Nouns in `honk` co-mingle PMA and NounSlabs

Cf. `docs/native-compiler/DOR-DEEP-EQUALITY.md`, "honk's nouns live in NounSlabs and the PMA where unification never happens"

This isn't allowed in the post-PMA nockvm. The burden of and reasons for copying/referencing PMA nouns (cold state? something else?) need to be analyzed and `honk` made to conform to PMA `nockvm` memory-safety constraints.

- Phase 0: commit to copy-at-boundary as the one regime; delete the dormant readonly-range machinery from NockStack/EqualityWork (TODOS-PERF #5). DONE (2026-06-12): readonly ranges, replace_extra_noun_ptr_ranges, and the EqualityWork rework are deleted; unifying_equality.rs is back to the merge-base implementation.
- Phase 1: make the boundary mechanical — handle/branded-handle APIs on honk's eval wrappers so raw slab nouns can't reach interpret; replace Compiled::formula()+noun_space() with a scoped with_formula (subsumes the "raw Noun without lifetime binding" TODO); corral the musk raw-pointer juggling (subsumes the "unsafe borrow workarounds" TODO).
- Phase 2: restore a release-mode net where it's free — copy_into already walks every node, so validate provenance there.
- Phase 3: slab generations instead of address-as-identity for the mack caches, explicit context epochs instead of frame_identity, then un-leak the slab (subsumes the unbounded-slab-lifetime TODO; also the right moment to fix mack's cached-panic conflation).
- Phase 4: only then cue the cold state once into a shared PMA arena for both contexts — at which point honk is a genuine post-PMA citizen and the co-mingling is gone by construction.

### Missing canonical hoonc-octs-type-138.jam silently downgrades /* data-import parity

- Evidence: crates/honk/assets/ only has honc-type-138.jam and honc-formula-138.jam; build.rs:12-25 writes an empty placeholder if assets/hoonc-octs-type-138.jam is absent; src/bin/honk.rs:907-915, 993-1001 only loads the canonical octs type when non-empty; data_vase falls back to local_octs_type at src/bin/honk.rs:1924-1941.
- Problem: A required parity input is optional. Data imports can compile with a local [p=@ud q=@] type instead of hoonc’s canonical $octs hold.
- Fix: Make the asset mandatory for parity/release builds. If absent/empty, fail before compiling any /* import.

### Artifact build leaves parse with docs disabled even though source spots are artifact data

- Evidence: parse_build_leaf calls parse_native_hoon_source_without_docs at src/bin/honk.rs:635-654; default public parser paths enable docs at pipeline.rs:86-101; policy says source spots/dbug are parity data in docs/native-compiler/source-spots.md:5-10.
- Problem: CLI artifact path normalizes parser input differently from parity policy. Doc-comment anchoring can change emitted dbug/spot nouns.
- Fix: Use doc-enabled parsing for artifact-producing parses unless there's a reason we needed this. If there is, skip this and make a note requesting clarification.

### Batch mode has known state-dependent miscompile risk

- Evidence: compile_entry_with_miss_persistence disables persistent miss memo for batches because shared state “drifts” and can “miscompile” at src/bin/honk.rs:1649-1658; README presents native_hoon_batch as normal cache-reuse mode at README.md:95-98; miss comments document drift and cached/fresh mismatches at ut/mod.rs:9349-9398.
- Problem: Correctness depends on caller-specific memo discipline, not complete cache keys.
- Fix: Include all semantic/rest/redo/nest/fan context in memo keys. Add batch-vs-single byte-for-byte parity tests. Do not reset all unsafe state per entry or it defeats the purpose of batch mode.

### Several semantic cache keys omit active %rest/fan context

- Evidence: semantic_context_key includes fan_context_key at ut/mod.rs:2076-2080; mint_cache_key only uses sut mug, gol mug, vet, gen signature at ut/mod.rs:2988-2996; redo, rest, and nest keys similarly use only mugs + vet at ut/mod.rs:3311-3346, 3377-3412, 3514-3544.
- Problem: The code documents fan context as semantic state but does not key many boundary caches by it. Equal-looking inputs under different active hold/rest state can reuse invalid results.
- Fix: Centralize typed cache-key builders that include full CacheContextKey; add tests where same nouns are evaluated under different fan scopes.

### Native parity is masked by embedded hoonc artifacts and oracle paths

- Evidence: canonical hoon-138 is detected by byte equality and loads embedded hoonc cold/type/formula assets at src/bin/honk.rs:850-889; canonical arbitrary wrapper path swaps in embedded hoonc formula at src/bin/honk.rs:1219-1225; pipeline::build_jam delegates directly to hoonc::build_jam at pipeline.rs:43-49.
- Problem: Primary “native” paths can succeed by importing hoonc-produced nouns or bypassing honk entirely.
- Fix: Split oracle/bootstrap mode from native parity mode. Native parity tests should fail if embedded hoonc substitutions are disabled and native output diverges.

### exact_swet_vase_trap is misleadingly not exact

- Evidence: mode plumbing distinguishes exact artifact swet for Arbitrary at src/bin/honk.rs:115-117, 1660-1663, 1804-1809; implementation simply calls native_swet_vase_trap at src/bin/honk.rs:1999-2007.
- Problem: The architecture suggests a hoonc-exact +swet path, but production uses native minting. That hides what is actually proven.
- Fix: Rename/remove the “exact” path unless it really calls hoonc-exact mint/swet; cover all output modes with raw byte fixtures.

### mack fold failures cache panics as ordinary “no fold”

- Evidence: musk_mack_constant_core caches Option<Noun> results at ut/mod.rs:5763-5794; musk_interpret_mack_in_context catches all unwinds and returns None at ut/mod.rs:5829-5851.
- Problem: Stack exhaustion, interpreter bugs, and expected non-folds collapse to the same cached value. That can silently de-optimize artifact shape/performance instead of surfacing a parity-critical failure.
- Fix: Distinguish expected interpreter Err, resource exhaustion, and internal panic. Cache only deterministic successes/expected failures.

### Import resolution is a partial handwritten hoonc model

- Evidence: imports are parsed only from the leading slash block at pipeline.rs:399-470; malformed /= and /* clauses return None silently at pipeline.rs:473-522; ScopedImport.mark is parsed but not used by resolve_imports_for at pipeline.rs:124-143, 221-244; hoonc documents mark-sensitive /* at crates/hoonc/README.md:76-86.
- Problem: Native importer approximates hoonc/ford semantics. Unsupported or malformed build runes can be ignored or treated as generic data.
- Fix: Hard-error malformed imports, explicitly support/deny each mark, and fixture every supported build rune.

### softed-constraints.hoon is a filename-based source bypass

- Evidence: native_value_override special-cases filename softed-constraints.hoon at src/bin/honk.rs:2537-2545; it loads fixed JAMs at src/bin/honk.rs:2548-2554.
- Problem: Source semantics are replaced by a path-name shortcut. Any source/import change can be bypassed unless the shortcut is updated.
- Fix: Compile normally, or key the optimization on content hash + import graph and prove byte parity.

### Rust oracle tests are semantic, not byte-artifact parity

- Evidence: only compare mode is NounSlabRejam at tests/compiler_mint.rs:274-289; comparison normalizes both sides and accepts structural equality at tests/compiler_mint.rs:296-319; docs require byte equality at docs/native-compiler/README.md:20-26 and artifact-parity.md:3-6.
- Problem: Byte layout, source metadata, and exact JAM encoding differences can pass tests.
- Fix: Add raw-byte compare mode for artifact tests. Keep structural diff diagnostic-only.

### Several “strict semantic parity” tests do not run hoonc

- Evidence: tests explicitly say they “intentionally do not run hoonc” and only run honk at tests/compiler_mint.rs:2659-2666.
- Problem: Hand-selected expectations can encode honk’s current behavior instead of canonical ++mint/++fire/++mull.
- Fix: Generate hoonc acceptance/artifact fixtures or add an oracle runner for these cases.

### Wrapper parity relies on hardcoded source positions and partial field population

- Evidence: native wrapper constructors hardcode hoonc spot lines/columns at src/bin/honk.rs:2634-2656; several ExactWrapperBatteries fields are D(0) at src/bin/honk.rs:2658-2674; dump tests cover a curated subset at src/bin/honk.rs:2365-2446.
- Problem: Future use of currently-zero fields can silently produce invalid artifacts; source-line drift breaks parity.
- Fix: Assert every mode-required wrapper is non-zero and fixture all wrapper batteries against source hashes/spans.

### Public API exposes raw Noun without lifetime binding to its slab

- Evidence: Compiled owns private NounSlab and formula: Noun at lib.rs:57-61; formula(&self) -> Noun and noun_space(&self) -> NounSpace expose handles separately at lib.rs:69-75.
- Problem: Safe API returns allocation-backed handles whose validity depends on Compiled’s private slab lifetime.
- Fix: Replace with scoped APIs like with_formula(|noun, space| ...) or a borrow-tied wrapper.

### Core ut implementation is too large and duplicates semantic dispatch

- Evidence: ut/mod.rs is ~12.7k lines; mint_inner and play have separate large rune dispatches around ut/mod.rs:3551-3792 and 3864-4145.
- Problem: Every rune change must stay synchronized across parallel dispatches. Divergence is likely and hard to review.
- Fix: Split by canonical boundary and share lowering/dispatch tables where possible.

### Partial parity markers remain in production core paths

- Evidence: find, fend, fund, fond marked status=partial / “full parity review is still in progress” at ut/find.rs:38-81; repo has the same at ut/repo.rs:174-176; fend returns generic fend-fragment at ut/find.rs:56-65.
- Problem: Production paths still declare incomplete parity review.
- Fix: Convert each marker into executable hoon-138 parity matrices; remove marker only when covered.

### Unsafe borrow workarounds live in high-level compiler logic

- Evidence: play uses raw pointer VetGuard to mutate self.vet at ut/mod.rs:3864-3885; musk interpretation takes raw pointers to slab/context at ut/mod.rs:5797-5805.
- Problem: Unsafe lifetime/state invariants are mixed into semantic logic.
- Fix: Isolate unsafe in a small module with documented invariants; use safe RAII for vet restoration.
