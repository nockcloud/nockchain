# Atomic-flip execution tracker (native-types migration)

Resumption anchor for the atomic replace (the FLIP): types become native
`Rc<Type>` as the working representation of `mint`/`play`; nouns are materialized
only at boundaries via `Type::to_noun`. This survives context resets — **update
the STATE section every turn.**

## Strategy: monotonic native-region expansion (compiles at every step)

Flip producers one connected step at a time to RETURN `Rc<Type>` (the type slot;
the formula slot stays `Noun`). At the current boundary, convert:
- incoming noun types → native via `intern::native_of(noun, space)` (a not-yet-
  flipped caller still hands us a noun `sut`);
- outgoing native types → noun via `native.to_noun(slab)` (a not-yet-flipped
  caller still expects a noun).

Each step compiles (boundary conversions bridge the gap) and the native region
grows; the boundary (and thus the transient noun (re)builds) shrinks. The memory
win grows monotonically — internal deepened subjects become shared `Rc`. At
completion the boundary is the OUTPUT only, and noun type construction is gone.

VALIDATION each step: `crates/honk/test-assets/native-parity/shadow_gate.sh`
(fast fixtures, byte-identical output). Full kernel byte-parity at completion.
Do NOT run full-kernel flag-on as a routine gate (O(n^2) until flipped).

## Conventions

- Type slot: `Rc<Type>` (alias `NRc<NTy>` in ut/mod.rs). Formula slot: `Noun`.
- mint family returns `(Rc<Type>, Noun)` = (type, formula); play family returns
  `Rc<Type>`.
- Construction: use the `_n` constructors' native (`ty_*_n(...).1`,
  `cell_type_n(...)?.1`) for now (transient double-build of the discarded noun;
  add native-only `cons_*` constructors later as an optimization). Collapses are
  already mirrored in the `_n` ctors.
- Boundary bridges: `native_of(noun, &slab.noun_space())?` (noun→native),
  `native.to_noun(self.slab)` (native→noun).
- Type consumers (nest/fond/repo/type_*_parts/wrap_type decoders) still take
  nouns for now; a flipped producer feeding a consumer `to_noun`s at the call.
  Consumers convert to read `Rc<Type>` in a later pass (drops those `to_noun`s).

## Ordered checklist (leaf producers → spine → consumers → boundary)

1. [DONE] play_core -> Rc<Type>  (callers: play_inner BarCen/BarPat)
2. [DONE] play_inner / play -> Rc<Type>. Arms: delegating forward native; leaf
   arms cons_*/native_of; helper arms bridge via `pb`. The ~40 external self.play
   callers + play_* helpers' internal self.play renamed to `play_noun` (= play +
   to_noun) so they keep compiling unchanged. Gate PASS, tests green.
3. [ ] play_* helpers -> Rc<Type>  (drop `pb`/`play_noun` bridges as each flips)
4. [DONE] mint_core -> (Rc<Type>, Noun). nice/core_mint_cache stay noun (cache
   bridged via native_of on hit); native built bottom-up via ty_core_n.
5. [DONE] mine -> (Rc<Type>, Noun). wrap_type/nice bridged (to_noun->...->native_of).
   Its 2 callers (mint_inner BarCen/BarPat) to_noun the type slot.
6. [ ] mint_inner / mint -> (Rc<Type>, Noun)  (78 self.mint callers -> mint_noun)
7. [ ] mint_* helpers -> (Rc<Type>, Noun)
8. [ ] nice / wrap_type -> Rc<Type>
9. [ ] type consumers (nest/fond/repo/type_*_parts) read Rc<Type>; drop to_noun shims
10. [ ] boundary: emit nouns only at output + typed-Dynock; delete noun ty_* ctors
11. [ ] full kernel byte-parity; delete _n duplicates / dead noun paths

## STATE (update every turn)

- Branch: feature branch (non-compiling intermediate accepted, but kept compiling
  so far via the boundary-bridge technique).
- Done: native IR boundaries (Type/Formula to_noun+from_noun), intern table,
  `_n` constructor + wrapper vocabulary, intern accessors
  (live_intern/native_of/assert_native_eq).
- Gate: `shadow_gate.sh` now compares fixture output vs FIXED `flip-baselines/*.jam`
  (the flag no longer changes output once producers return native). PASS.
- Steps 1+2 DONE: `play_core` + `play`/`play_inner` return `Rc<Type>`. Bridges:
  `cons_cell`/`cons_void`/`cons_noun` (native leaf ctors), `pb` (helper noun ->
  native), `play_noun` (native play -> noun for not-yet-flipped callers). Byte
  parity PASS, native tests green, lib + bin compile.
- Steps 4-5 DONE: `mint_core` + `mine` return `(Rc<Type>, Noun)`; nice/cache stay
  noun (bridged), callers (mint_inner BarCen/BarPat) to_noun the type slot.
- Compiles: YES.

KEY FINDING (2026-06-19, corrected): the migration is CONSUMER-DOMINATED, and the
producer-output flips have reached the boundary of what's cheap.
- "nice-first" was WRONG: `nice` has ~52 callers (nearly every mint_* helper +
  inline arm), all passing/consuming NOUN. Flipping nice alone cascades to 52
  bridge sites with no benefit (reverted). nice can only flip together with its
  callers.
- ROOT shape: the type is consumed pervasively by NOUN-based code — nice, nest,
  fond, fish, peek, gain, lose, fuse, crop, the whole type-algebra, and the
  type_*_parts decoders. ANY native type bridges back to a noun (to_noun) for
  these. So nouns are still BUILT (transiently, per consumer call) until the
  CONSUMERS read native. => no memory win, and every further producer flip just
  adds bridges, until the consumer subsystem is native.
- Therefore the remaining BULK = rewrite the type-consuming subsystem to operate
  on `Rc<Type>` (match the enum) instead of decoding nouns: the type_*_parts
  decoders first (the leaves of consumption), then peek/gain/lose/nest/fond/fish/
  fuse/crop/wrap_type, then nice. Once consumers are native, the producer natives
  (play/mint_core/mine, already done) flow straight in with no to_noun, the sut
  input can thread native (shared subject => the O(N^2)->O(N) win), and the noun
  ty_* ctors can be deleted. This is the large core of the migration (~500+ lines
  of type algebra), best done decoder-leaves-first, each validated by shadow_gate.

DONE producer spine: play_core, play/play_inner, mint_core, mine -> native
(committed, compiling, byte-parity). play_* and mint_* helpers + mint_inner/mint
dispatch remain noun (bridged) and are best flipped AFTER the consumer subsystem
so they don't double-bridge.

CONSUMER FLIP (mapped 2026-06-19 by consumer-flip-map workflow; leaves-first,
flip-in-place + bridge; decoders become enum matches retired bottom-up). Plan
steps and status:
  C1 [DONE] repo/repo_hold -> native (repo_noun bridge for 27 callers).
  C2 [DONE] peek -> native (Core lowers coil leaf to coil_parts/garb_vair; Fork lowers
         set to fork_set_options; Hold -> repo native). ~4 callers.
  C3 [DONE] wrap_type -> native (needs collapse-aware cons_core/cons_face/cons_hint).
  C4 [DONE] fuse/fuse_inner -> native (fitz stays noun; nest bridged; caches keep
         to_noun mug keys until C-final).
  C5 [DONE] crop/crop_inner/crop_sint -> native.
  C5b [DONE] miss family (miss/miss_dext/miss_dext_uncached/miss_sint, mod.rs ~9612)
         -> native (returns bool; type params sut/ref_ -> NRc; uses type_*_parts +
         repo + nest — same pattern as crop; nest bridged via lowering).
  C8 [ ] NEST SCC (nest/nest_inner/nest_inner_impl/nest_sint/nest_core/nest_meet/
         nest_deep_tomes/nest_deep_arms) — ONE atomic step (~600 lines, 8 mutually
         recursive fns -> must flip together; non-compiling until all done).
         DO THIS NEXT: independent (deps peek/repo/wrap_type already native; hot
         path), unlike gain/lose. nest_* take sut/ref_ (+ dom/dab/hem/dox) -> NRc;
         seen/gil/interner/memo keyed by native pointer; nest_deep_* use native
         play (its result drives nest, can stay native). nest_noun bridge for the
         many callers (nice/fuse/crop/miss/gain/lose/mull/...). nest_core lowers
         the core coil leaf for coil_parts/garb_*/rest_tomes.
  C6 [ ] gain/lose skin families (~12 fns) + cool/chip -> native. ENTANGLED with
         find/take/Port/Palo (chip/cool drive `take`, whose duz closure passes
         type NOUNS to the skins) -> flip TOGETHER with the find/take + fond
         batch (C9), not standalone. gain_skin builds via cons_face/cons_hint/
         fork; uses fuse/crop (native), nest (native after C8), play (bridge).
  C7 [ ] mull + type_test_formula_on_axis glue -> native.
  C9 [ ] find/take/Port/Palo + fond family -> native (the wing-nav subsystem;
         gain/lose/cool/chip fold in here).
  C-final [ ] BOUNDARY CLOSE = the memory win: thread sut native (play/mint sut
         param Noun->Rc<Type>); delete play_noun/pb/repo_noun + all transient
         native_of/to_noun bridges; retire decoders (type_*_parts/type_tag*/coil_*)
         + noun ty_* ctors; re-key boundary caches (nest_mug/fuse/crop/rest/hold)
         on native identity. This stops the grow-only NounSlab accumulation.
KEY RISKS (from the map): collapse-parity (add cons_core/face/hint, never bare
live_intern for branch rebuilds); fork RT-07 order (keep noun fork path til late);
boundary-cache key drift (keep to_noun mug keys until C-final); NEST SCC atomicity.

## RESUMPTION NOTES (2026-06-19) — read before continuing the grind

BRANCH: fwd/bitemyapp/native-compiler-pma-native-compiler-types (NOT pma-hell-4;
I mixed them up once — flip commits are here). HEAD = the latest "FLIP consumer-N"
commit.

USER DIRECTIVE: complete the ENTIRE flip (blind big-bang); do NOT ask per-step.
The branch is intentionally perf-broken (real-kernel compiles >180s) until C-final
removes all bridges — this is expected, not a regression to chase. Validate each
family ONLY with the fast fixture gate; dumb byte-parity+perf is checked at
C-final against /tmp/dumb_preflip.jam (the verified pre-flip golden, 19873112 B;
regenerate from commit e6e653e2 if /tmp lost it).

PERF (settled): the flip's slowness is the pervasive bridge machinery
(native_of/to_noun/ty_core_n double-build/mug-keyed caches) on every algebra call;
it vanishes only at C-final. Infra in place: Leaf carries a cached content hash
(leaf.rs) so interning leaf-carrying types is O(1)/leaf; intern::live_to_noun
(Type) + live_leaf_to_noun (coil/set) memoize lowering by interned/Arc pointer.

WORKFLOW: a linter reformats mod.rs constantly -> the Edit tool fails "modified
since read". Apply edits via `python3` text-replace (read current content, assert
count==1, replace, write). Pattern per consumer family:
  1. extract current fn text via awk to /tmp; python-replace with native body.
  2. native body: match &*sut (and &*ref_); Rc::ptr_eq for equality; cons_cell/
     cons_void/cons_noun/cons_core/cons_face/cons_hint for rebuilds (collapse-aware);
     self.repo(x) native; for leaf-carried parts (coil/set/atom) lower via
     live_leaf_to_noun + the existing noun decoder (coil_parts/fork_set_options/
     type_atom_parts); native-pointer seen-sets (HashSet<(usize,usize)>).
  3. not-yet-flipped callees (nest until C8) bridged by lowering args via
     live_to_noun.
  4. add `fn <name>_noun` bridge; python-rename `self.<name>(`/`ut.<name>(` ->
     `<name>_noun(` in mod.rs/wet.rs/find.rs/test.rs; then fix the bridge fn's OWN
     self-call back to native (the rename hits it).
  5. cargo build -p honk --lib; then release build + shadow_gate.sh (timeout 120).
     commit. mark C# DONE here.

DONE so far: C1 repo, C2 peek, C3 wrap_type (+cons_core/face/hint), C4 fuse,
C5 crop. NEXT: C5b miss, C6 gain/lose+cool, C7 mull glue, C8 NEST SCC, C9 fond,
C-final.


## NEST SCC impl plan (do as ONE atomic python pass — next step)

nest family (mod.rs ~8124-8620): nest, nest_inner, nest_inner_impl, nest_sint,
nest_core, nest_meet, nest_deep_tomes, nest_deep_arms + atom_nest (Atom arm).
All mutually recursive -> flip together (non-compiling until all done).

Helper structs (mod.rs, search NestTypeInterner/NestSeenSet/NestPairSet/
NestMemoKey): they assign ids to type NOUNS (id_for) and key the memo/seen/gil on
those ids. NATIVE: the canonical Rc pointer IS the id -> drop NestTypeInterner;
key NestSeenSet (HashSet<u64>), NestPairSet (HashSet<(u64,u64)>), NestMemoKey
{sut:u64, ref_:u64, seg, reg, gil} on NRc::as_ptr(&t) as u64. snapshot()/insert/
remove become trivial ptr ops (no `self`/interner needed).

Per-fn: sut/ref_ (+ dom/dab/hem/dox/vim in nest_core/meet/deep) -> NRc<NTy>;
noun_eq(sut,ref_) -> Rc::ptr_eq; type_tag_kind+type_*_parts -> match &*; repo ->
native (done); nest_deep_* play_noun -> native play (result drives nest); nest_core
lowers the core coil leaf via live_leaf_to_noun for coil_parts/garb_poly/garb_vair/
rest_tomes (those stay noun); atom_nest reads NTy::Atom leaves (lower small for
type_atom_parts/fitz). nest_mug_lookup/register (3542): keep noun-keyed (lower
sut/ref_ via live_to_noun) until C-final.

Then: add `fn nest_noun(sut: Noun, ref_: Noun) -> bool` bridge; python-rename the
19 self.nest(/ut.nest( callers -> nest_noun (in mod.rs/wet.rs/find.rs/test.rs);
fix nest_noun's own self-call back to native; cargo build -p honk --lib; release
build + shadow_gate.sh; commit; mark C8 DONE.
