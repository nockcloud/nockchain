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

