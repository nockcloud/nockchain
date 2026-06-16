# honk: Native-Types Migration Plan

Migrate honk's *working* representation of the Hoon type system and Nock output
from **Nouns** to **native Rust data structures** (`Type`, `Formula` enums with
`Rc` sharing + hash-consing), emitting Nouns only at the output boundary via
`ToNoun`. This is a dedicated branch effort.

Status: PLAN (pre-implementation). Author: native-compiler track.
Reviewed: adversarially reviewed against the honk source (3 critics — arch
soundness + code verification, parity/completeness, sequencing/sizing); their
findings are folded in (notably: the three distinct hint kinds, the play-path
formula sites, the mack/fold interpreter as a second `to_noun` boundary, the fact
that laziness cannot be deferred past Phase 2, and the precise jam-dedup
byte-safety argument). See §3.9, §7 R9–R12.

---

## 0. Why

honk reimplements `++ut` (Hoon's type/compile system) in Rust, but represents
compiler **types** and **formulas** as `Noun` values allocated into a grow-only
`NounSlab`, mirroring hoon-138's noun encodings. That choice is the root of the
problems this branch exists to fix:

- **Memory.** Native hoon-138 mint OOMs: peak RSS ~32 GB on a 128 GB machine
  with no convergence (linear ~3 GB/min). Diagnosed root cause = honk builds many
  structurally-equal type nouns **without sharing** (no hash-consing), compounded
  by **subject-deepening** (`mint_core` embeds the whole subject per core, O(N²))
  and **resolver-id churn** (a fresh `lazy_resolver_next_id` per core defeats any
  structural dedup), all accumulating in a never-freed slab. The H7 frame-arena
  proved per-arm scratch reclamation cannot bound this: the mass is *preserved,
  shared, un-interned* structure, not transient scratch.
- **A persistent bug class.** Pointer-keyed caches that dangle, mug collisions,
  `noun_eq` deep walks, the `as_raw()` cache machinery, dbug-spot preservation,
  the frame-arena `copy_to_base` duplication — all are artifacts of using nouns
  as the *working* representation.
- **Maintainability.** ~12.7k-line `ut/mod.rs` dispatches on noun tags via
  string compares and hand-decodes noun shapes; the type structure is not
  enforced by the compiler.

**The bet:** native Rust `enum`s give us structural sharing (`Rc`), hash-consing
(an intern table with a cached structural hash and `Rc::ptr_eq` short-circuit —
*real* hashing, not mugs, so the mug-collision problem disappears), and automatic
reclamation (`Drop`). That directly dissolves the memory wall, removes the bug
class, and makes the algorithm legible. We only pay a `ToNoun` at the output
boundary.

### The enabling fact

honk's output is **formula-only in the normal modes**:

- **Standard / Arbitrary** (kernels + parity): output jam is **only the Nock
  formula** — a `[battery payload]` trap. The Hoon *type* never appears in the
  bytes. (`honk.rs:2335-2349`, `2047-2058`.) Types are pure internal scaffolding.
- **Dynock** (typed builds, e.g. the octs-type probe): output is `[type formula]`
  — the type *is* in the bytes (`honk.rs:2732-2738`).

So for the dominant path, **types can be 100% native with zero byte-exactness
constraint and no `ToNoun` at all.** Only the **formula** needs `ToNoun`, plus a
narrow **type `ToNoun`** for Dynock.

---

## 1. Goals, non-goals, success criteria

### Goals
1. Bound the memory of native hoon-138 minting so self-hosting is *feasible*
   (target: completes within a small multiple of the embedded path's working
   set; concretely aim < ~8–16 GB and converging, vs current 32 GB diverging).
2. Make honk meaningfully faster (eliminate mug walks, noun decode, deep
   `noun_eq`; recover the roswell <60s gate).
3. Preserve **byte-exact output parity** with hoonc at every step (the contract).
4. Eliminate the noun-representation bug class.
5. Improve maintainability: exhaustive `match` over a small, enforced type
   algebra.

### Non-goals
- Changing `++ut` **semantics** (nest/mint/mull/find/… behavior is preserved;
  this is a representation change, not an algorithm change).
- Changing the **output contract** (the Nock the user gets is unchanged).
- Rewriting nockvm / `NounSlab` — they remain for I/O, output jamming, quoted
  constants, and any noun leaves.
- Fixing the functional-parity algorithm tail (roswell wet polymorphism,
  find/fend matrices) *as part of* the migration — that is orthogonal work (see
  §7 R2), though it benefits from the cleaner representation.

### Success criteria (acceptance)
- **A1.** Every currently-passing kernel (dumb, wal, miner, peek, bridge)
  compiled by native honk is **byte-identical** to the noun honk / hoonc
  reference. (roswell tracked separately — see R2.)
- **A2.** `--native-parity` hoon-138 mint **completes** with bounded RSS
  (record the number; the bar is "converges in available RAM", the stretch is
  "competitive with the embedded path").
- **A3.** `cargo nextest run` workspace-green; `just honk-parity` green; debug
  build green (validation regime).
- **A4.** roswell native compile recovers under the 60s gate (or the regression
  is understood and accepted, as today).
- **A5.** The noun-based `ut` is retired; `NounSlab` use in honk is output-only.

---

## 2. The contract (what must remain byte-exact)

| Surface | Constraint | Needs ToNoun? |
|---|---|---|
| Nock **formula** (all modes) | byte-exact incl. quoted constants, jet hints (`%fast`), and **dbug spots** | **Yes — `Formula::to_noun`** |
| Hoon **type** (Standard/Arbitrary) | none — internal only | No |
| Hoon **type** (Dynock only) | byte-exact `[%tag …]` per hoon's `++ut` type encoding | Yes — `Type::to_noun` (Dynock path only) |
| Cold state / wrapper assets | unchanged (already honk-produced nouns) | n/a (boundary) |

The unforgiving oracle is byte-equality of the output jam. Because interning and
native representation produce *structurally equal* outputs, jam bytes are
unchanged — that is the invariant every phase gate checks.

---

## 3. Target architecture

### 3.1 The native/noun split (key design decision)

- **Types are fully native** — no noun leaves. Children are `Rc<Type>`; auras and
  face names are interned symbols (`Rc<str>` or a symbol id); `%hold` genes are
  the **native hatch AST** (`Rc<Hoon>`), not noun-encoded AST. This makes types
  cleanly hash-consable.
- **Formulas are native with noun leaves.** A `Formula` enum maps to Nock
  opcodes, but its leaves — quoted constants (`[1 noun]`), hint clues, dbug spot
  tuples — are `Noun`s. `to_noun` assembles the byte-exact Nock noun.
- **The AST round-trip is dropped (target state).** hatch parses to a Rust
  `Hoon` enum, but **today** `ut` re-encodes it to a noun (`hoon_to_noun`,
  `ut/mod.rs:6761`) and re-decodes (`hoon_ast_lookup_result`, `decode_hold_hoon_ast`)
  — a round-trip with its own caches. The migration's *target* is for `ut` to
  operate on `&Hoon`/`Rc<Hoon>` directly, removing that layer. This is real work
  in Phases 1–2 (every `hoon_ast_lookup_result` / `repo` / `play` site that
  consumes the noun-AST must be ported), not a free given.
- **Nouns remain** for: final output (jammed from a fresh `NounSlab` via
  `to_noun`), quoted constants embedded in formulas, hint clues, dbug spot
  tuples, cold-state / wrapper assets.

### 3.2 Formula IR

```
enum Formula {
    Slot(u64),                         // [0 axis]
    Quote(Noun),                       // [1 const]      (noun leaf)
    Eval(Rc<Formula>, Rc<Formula>),    // [2 subj form]
    Cell(Rc<Formula>, Rc<Formula>),    // autocons [f g]
    Op(u8, ...),                       // 3/4/5/7/8/9/10/12 as needed
    Cond(Rc<Formula>, Rc<Formula>, Rc<Formula>),  // [6 p q r]
    // Hint kinds are NOT interchangeable — they have different to_noun encodings
    // AND different compile-time semantics. Do not collapse into one variant:
    JetHint  { clue: Noun, body: Rc<Formula> },    // [11 [%fast …] body] — registers a battery in the warm/cold jet registry
    NoteHint { note: Noun, body: Rc<Formula> },    // [11 [%hint @] body] / [12 …] type/typo notes from the play path
    Dbug     { spot: Noun, body: Rc<Formula> },    // [11 spot body] — debug location
}
impl Formula { fn to_noun(&self, slab: &mut NounSlab) -> Noun { /* exact Nock */ } }
```

- Replaces `cons`/`comb`/`cond`/`T(...)` formula construction in `formula.rs`.
  These functions have **zero type coupling** today (confirmed), so the leaf
  builders slot in cleanly — BUT formulas are emitted from **both** the `mint`
  path **and** the `play` path (e.g. `play_note`/`mint_note` build `[12 …]`/`[11 …]`
  directly, `ut/mod.rs:5759,6168`). The Formula-IR port must cover the play-path
  emit sites too, not just `mint*`.
- **Three distinct hint surfaces** (a review finding — the original single `Hint`
  conflated them): `%fast` jet hints feed the **jet battery registry** (warm/cold)
  and their clue is a `[%fast [%clls …]]` chain (`ut/mod.rs:2751`); `%spot`/dbug
  carry source locations; `%note`/`%typo` are type-level notes. `to_noun` and any
  registry side-effects must be handled per-kind.
- **dbug spot stack.** Spots are sequenced via a context stack (`dbug_locations`,
  `include_dbug_spot`; `mint_dbug` push/pop at `ut/mod.rs:7672`). The `Dbug` node
  must capture the spot **at mint time in stack order** so the `[11 spot …]`
  nesting in the emitted formula is byte-exact (it is not merely error
  decoration).
- `Rc` + a small intern table dedups shared subformulas (battery sharing). Note:
  quoted-constant noun leaves need **not** be deduped by `to_noun` for
  byte-exactness — honk's jam re-deduplicates structurally at emit (see §3.9).

### 3.3 Type representation + hash-consing

```
enum Type {
    Void,
    Noun_,                              // %noun
    Atom { aura: Sym, constant: Option<Rc<BigUint>> }, // %atom
    Cell(Rc<Type>, Rc<Type>),          // %cell
    Face { name: Sym, inner: Rc<Type> },               // %face (+ %face tool variant)
    Fork(BTreeSet<Rc<Type>>),          // %fork  (canonical-ordered set)
    Hint { /* %hint */ },
    Core(Rc<Core>),                    // %core
    Hold(Rc<Hold>),                    // %hold
}
```

- **Intern table** (`TypeTable`): `HashMap<&Type, Rc<Type>>` (hashbrown raw entry)
  returning the canonical `Rc<Type>`. Each `Rc<Type>` caches its structural hash
  (computed once at intern, O(children) since children are already interned).
  `Eq`/`Hash` short-circuit on `Rc::ptr_eq` of children → O(1) amortized
  bottom-up interning. The noun path only deduplicates at **jam time** (the
  `NounMap` mug+`noun_eq`); it does **no construction-time sharing**, so equal
  type subtrees are re-allocated all through the mint — that is the memory cost we
  remove. The earlier *top-down* mug-interning attempt regressed because it
  walked whole cores and fell to full `noun_eq` on mug collisions; **bottom-up**
  interning with cached per-node hashes + `ptr_eq` short-circuit is what makes it
  cheap (children are already canonical, so each parent intern is O(1) amortized).
- After interning, **structural equality == pointer equality** → `nest`/`find`
  fast paths become `Rc::ptr_eq`; the boundary memos key on `Rc` identity.

### 3.4 Core, battery, coil; lazy cores (the resolver replacement)

```
struct Core { payload: Rc<Type>, garb: Garb, context: Coil, battery: Battery }
enum Battery { Full(Rc<...>), Lazy(Rc<LazyBattery>) }
struct LazyBattery {
    context: Rc<Type>, poly: Poly,
    arms: Rc<ArmMap>,                                  // axis -> &Hoon (native AST)
    cache: RefCell<HashMap<u64 /*axis*/, Rc<Formula>>>,// memoized compiled arms
}
```

- **No `lazy_resolver_next_id`.** Sharing is by `Rc<LazyBattery>` identity, so
  structurally-identical / re-minted cores share one battery instead of forking a
  new integer id. This is the specific mechanism that defeated dedup in the noun
  path; it disappears.
- Arm resolution caches the compiled `Rc<Formula>` in the `LazyBattery` (lives as
  long as the core's `Rc`); cross-arm/cross-core resolution is a lookup, not a
  scope-sensitive re-mint, so the H7-C "wrong %hold/fan scope on re-mint" hazard
  also disappears.
- **Lazy is not optional and likely cannot be deferred to Phase 3.** hoon cores
  reference their own/sibling arms; **eager** minting would not terminate — that
  is *why* the noun path has the lazy seminoun in the first place. So the native
  type path cannot run end-to-end without a laziness mechanism. Practical
  consequence (see §5): either the lazy battery lands together with the type enum
  (Phases 2+3 partly merge), or Phase 2 temporarily **bridges** to the existing
  noun lazy-resolver while only leaf/type *shapes* are native. Do not assume a
  clean "eager cores first, lazy later" split.
- **Seminoun lifetime / output.** Today `[%lazy 1 id]` seminoun masks are embedded
  in *type* nouns and the resolver tables are deliberately retained for the whole
  Ut (`clear_build_memos` preserves them, `ut/mod.rs:2031`). In native, the
  `Rc<LazyBattery>` must outlive any type that references it. Critically: a type
  carrying an unresolved lazy battery must **never reach `to_noun` for output**
  (Dynock) with a dangling/absent battery — it must be fully resolved (or its
  seminoun faithfully re-encoded) before emit. Phase 3 must define this contract.

### 3.5 `%hold` recursive types — model as finite lazy nodes, NOT Rc cycles

`Hold { subject: Rc<Type>, gene: Rc<Hoon> }` is a **finite** node. Recursion is
expressed by `repo`/expansion producing the unrolled type on demand (lazily,
memoized on `Rc<Hold>` identity) — exactly as hoon-138's `%hold` is a finite
encoding expanded by `++repo`. **We deliberately do not build cyclic `Rc`
graphs** (which would leak, since `Rc` cycles never drop). This is the single
most important correctness/leak guardrail of the design.

### 3.6 fan / `%rest` scope

Native: a stack of active legs keyed by `(Rc<Type> inner, Rc<Hoon> gene)`
identity (pointer equality after interning) instead of mug buckets + interned
integer leg-ids + signature XOR. The fan-context key becomes a cheap set of
`Rc` identities.

**Why `Rc` identity is sufficient here (it is not in noun-land).** Today
`hold_repo_fan_leg_lookup_id` (`ut/mod.rs:2100-2119`) dual-keys: raw pointer
first, then mug bucket, then `noun_eq` fallback — *because nouns are not
canonical* (structurally-equal nouns have different pointers). Hash-consing makes
the type representation **canonical**: structurally-equal interned types ARE
pointer-identical, so `Rc::ptr_eq` is exact and the raw+mug+noun_eq triple
collapses to a single pointer compare. This canonicality is a hard invariant the
implementation must uphold (every type that enters the algebra is interned);
**add a debug assertion that no two live `Rc<Type>` are structurally-equal but
pointer-distinct** — a violation silently breaks every identity-keyed memo.

### 3.7 Caches

Most current caches **shrink or disappear**:
- `boundary_memo` (mint/mull/redo/rest/fish/nest/crop/fuse), `lookup_memo`
  (find/cool/chip/…), `hold_memo` → keyed on `Rc<Type>` identity + small context,
  O(1) hash, no mugs, no `as_raw`, no dangling, no clear-on-frame-pop.
- The raw pointer caches (`*_raw`, `mack_*_raw`, `hold_repo_fan_leg_raw_ids`),
  `hoon_cache_*`, `hoon_identity_cache_*`, `hoon_ast_ptr_cache`,
  `decoded_hold_*`, the mug machinery → **deleted** (their reason for existing is
  the noun representation).
- The H7 frame arena (`push_frame`/`pop_frame_preserving`/`copy_to_base`,
  `invalidate_frame_caches`, region stack) → **removed**; `Rc`/`Drop` replaces it.

### 3.8 Output boundary

`Compiled` carries `Rc<Formula>` (+ optionally `Rc<Type>` for Dynock). At emit
time, a fresh `NounSlab` is filled by `formula.to_noun(slab)` (and
`type.to_noun(slab)` for Dynock), then jammed. Quoted constants / clues / spots
are copied in.

**Byte-exactness of output is independent of internal sharing.** honk's jam
(`NockJammer` in `slab.rs`) deduplicates with a `NounMap` keyed by **mug +
`noun_eq`** (structural), emitting backrefs for any *structurally*-equal
subnoun regardless of pointer identity (`slab.rs` backref insert/get). So the
output bytes depend only on the *logical* tree `to_noun` produces, not on how
much the native representation shared internally. This is why hash-consing is
byte-safe for output — but note it also means the memory win comes purely from
**construction-time** sharing (the native side), a *separate* property from
jam-time dedup; do not conflate the two in the rationale.

### 3.9 The in-compile interpreter (mack/fold) — a SECOND noun boundary

honk executes Nock **during** compilation — `^~` folds and similar run through
the noun-based interpreter (`musk_interpret_mack_in_context`, the `MuskRuntime`,
`ut/mod.rs:5848+`): it builds a **noun** formula (`T(stack, [D(9) …])`) and calls
`interpret(context, core, formula)` on the live eval stack, then copies the
result back. This is a real `to_noun` boundary the output section does not cover:

- The fold path must `formula.to_noun(...)` (into the eval stack / a scratch
  slab) before calling `interpret`, and the folded *value* result comes back as a
  noun (it is data, not a type — it stays noun, possibly re-interned if it feeds
  a `%atom` constant).
- The interpreter (nockvm) stays noun-based — we are **not** rewriting it. The
  boundary is: native `Formula` → `to_noun` → `interpret` → noun value.
- Fold-result caching (`ktsg_fold_cache`, keyed today on `mug(bran)+mug(formula)`)
  must be re-keyed on native identity (`Rc<Formula>` / `Rc<Type>`), not deleted.

---

## 4. Migration strategy: dual representation, noun honk as oracle

The type algebra is interconnected; we cannot run "half native". So:

- Build the native core (`Type`, `Formula`, the algebra) as a **new module set**
  alongside the existing noun `ut`, behind a flag (`HONK_NATIVE_IR` /
  `Ut::native_ir`), defaulting OFF.
- The **noun `ut` is the oracle.** A **dual-run harness** compiles each corpus
  item with both paths and asserts the output jams are byte-identical. The native
  path is grown until it passes the whole corpus; then the noun path is retired.
- Migrate in dependency order (leaves → composites → algorithm → lazy/hold/fan →
  caches), each phase gated by the oracle. Within a phase the change may be
  "big-bang" inside the native module, but it is always validated against the
  noun reference.

Corpus (the oracle's input set, expand aggressively):
- the 6 kernels; `compiler_mint.rs` fixtures; a broad sweep of `hoon/` library
  files and standalone expressions; hoonc-oracle fixtures (deferred item #11 —
  worth building here as the parity net).

---

## 5. Phases (units + gates)

### Phase 0 — Scaffolding & oracle (small)
- New crate module layout: `native_ir/{formula.rs, ty.rs, intern.rs, core.rs}`.
- Dual-run harness + `just native-parity-dual` (compile corpus both ways,
  jam-diff). Wire into CI as the gate for all later phases.
- **Concrete corpus** (don't leave "expand aggressively" vague — under-coverage
  makes A2/parity unprovable): (a) all 6 kernels; (b) every `compiler_mint.rs`
  fixture; (c) a broad sweep of `hoon/` library files, weighted toward files with
  **wet/`|*` arms, `|-`/recursion, and `=>`/core chains** (to stress fire/redo,
  holds, and subject-deepening); (d) targeted fixtures with **structurally-equal
  cores** (to prove canonical interning / dedup); (e) Dynock fixtures (for
  Phase 5). Stand up hoonc-oracle fixtures (#11) as the external net.
- **Gate:** harness runs; noun path still green (no behavior change yet).

### Phase 1 — Formula IR (de-risking first slice; NO type changes)
- `enum Formula` + `to_noun` (byte-exact incl. quoted constants, the **three hint
  kinds** `JetHint`/`NoteHint`/`Dbug`, and **stack-ordered dbug spots**). Build it
  producible **in parallel** with the existing noun formula so
  `to_noun(F) == current noun formula` can be asserted in isolation.
- Refactor `formula.rs` + **all** formula-emit sites — the `mint*` path **and**
  the `play*` path (`play_note`, etc.) — to build `Rc<Formula>`. Note this is a
  larger site set than the ~476 *type* sites and is its own scope (see §9).
- Wire `to_noun` at **both** boundaries: final output **and** the mack/fold
  in-compile interpreter (§3.9).
- **Gate:** all passing kernels still byte-exact; a fixture compiling 100+
  expressions both ways asserts `formula.to_noun jam == noun-formula jam`
  byte-for-byte — catching hint-tag encoding, `%fast` registry, spot order, and
  fold-path differences **before** any type work.

### Phase 2 — Type representation + hash-consing (NO algorithm change)
- `enum Type` + intern table + `Rc`. Replace the 11 `ty_*` constructors with
  interning builders and the 11 `type_*_parts` decoders with `match` arms.
- Port the ~476 type-noun call sites (147 `T()` + 117 `type_*_parts` in mod.rs,
  plus find.rs/test.rs/etc.). This is the bulk mechanical work; do it
  module-by-module under the flag, dual-run after each.
- Operations to port (representation only, semantics unchanged): `mint`,
  `mint_core`, `mint_tsgr`, `play`/`play_inner`/`play_cnts`, `nest`, `mull`,
  `find`/`fond`/`fend`/`fund`, `fish`, `crop`, `fuse`, `gain`/`lose`, `peek`,
  `bran`/`ride`, `repo`/`rest`.
- **Laziness cannot be deferred** (review finding): eager core minting does not
  terminate on self/sibling-referential cores, so the native type path cannot run
  end-to-end without a lazy mechanism. Choose one bridge for Phase 2:
  (i) keep the **existing noun lazy-resolver** as a temporary backend while only
  type *shapes* are native (a hybrid with a conversion seam), or (ii) bring a
  minimal native `LazyBattery` forward into Phase 2. Plan for (ii) if the seam in
  (i) proves as costly as just doing the native lazy battery.
- **Gate:** native path produces byte-identical formulas to the noun path on the
  full corpus (using whichever lazy bridge was chosen).

### Phase 3 — Lazy cores / hold / fan native (the hard part; the memory win)
- `LazyBattery` (`Rc` + per-arm memo), drop `resolver_id`. `%hold` as finite
  lazy nodes (§3.5). fan scope via `Rc` identity (§3.6).
- **This is where the giant-core mint should bound.** Measure RSS + time on the
  `--native-parity` hoon-138 mint.
- **Gate:** corpus byte-exact **AND** A2 (hoon-138 mint converges, bounded RSS).

### Phase 4 — Cache redesign + perf
- Replace remaining noun caches with `Rc`-identity memos; delete the dead noun
  caches and the H7 frame arena. Profile and tune.
- **Gate:** corpus byte-exact; roswell <60s gate (A4); benches at/under noun
  baseline.

### Phase 5 — Dynock type `to_noun`
- `Type::to_noun` matching hoon's `[%tag …]` encoding; validate Dynock builds
  (octs-type probe path) byte-exact.
- **Gate:** Dynock parity.

### Phase 6 — Retire the noun `ut`
- Delete `ut` (noun) once native passes the full corpus; reduce `NounSlab` use in
  honk to output-only; remove the dual-run flag.
- **Gate:** A1–A5 all met; full suite green; docs updated.

---

## 6. Validation (cross-cutting)

- **Dual-run on every change** until Phase 6: `native_jam == noun_jam` (and both
  `== hoonc_jam` where applicable) over the corpus. CI gate.
- **Debug-build validation** stays on (catches representation bugs early).
- **Expand the corpus** beyond the 6 kernels to stress functional parity — this
  is also how we make progress on the algorithm tail safely.
- **Memory/time tracking** from Phase 3: record RSS + wall for hoon-138 native
  mint at each gate; regression = blocker.

---

## 7. Risk register

- **R1 — Parity regression during the port.** *Mitigation:* dual-run oracle,
  module-by-module under a flag, never delete the noun path until native is at
  full corpus parity.
- **R2 — The algorithm tail is orthogonal and must not be conflated.** roswell
  wet polymorphism (redo/mull/nest) and find/fend matrices are *algorithm* gaps,
  not representation. *Decision:* fix them on the **noun honk first** so the
  oracle is 6/6 before/while migrating (a buggy oracle is worse than a slow one),
  OR explicitly track them as native-only follow-ups. Recommend: close roswell on
  noun honk first so the oracle is complete.
- **R3 — `Rc` cycles in recursive types → leaks.** *Mitigation:* §3.5 — model
  `%hold`/recursion as finite lazy nodes expanded on demand, never cyclic `Rc`.
  Add a debug assertion / leak check (e.g. count live `Rc<Type>` across a mint).
- **R4 — Hash-consing soundness with noun-ish leaves.** Auras/names → interned
  symbols; genes → native AST `Rc<Hoon>` (must be hashable/eq — derive on the
  hatch AST or intern it); `Fork` must use a **canonical ordering** (BTreeSet of
  interned children) so equal forks intern equal. Quoted constants live only in
  formulas (noun leaves there), not types.
- **R5 — Dynock type encoding drift.** *Mitigation:* Phase 5 dedicated, fixtures
  against hoon's type-noun format; low traffic so contained.
- **R6 — Scope/time.** Large (≈476 sites + algorithm + lazy/hold/fan). *Mitigation:*
  each phase is independently valuable and gated; Phase 1 (formula IR) lands real
  value and de-risks fast; Phase 3 is the long pole.
- **R7 — Performance of the intern table.** A bad hash or non-amortized interning
  could regress. *Mitigation:* cache per-node hash, intern bottom-up, `ptr_eq`
  short-circuit; bench against the noun baseline at each gate.
- **R8 — Two representations coexisting balloons complexity/build size.**
  *Mitigation:* keep the flag short-lived; aim to retire the noun path promptly
  after Phase 3 corpus parity rather than carrying both indefinitely. Note the
  interconnected algebra means the flag effectively forces near-duplication of
  `ut` while both live — budget for that, and minimize the window.
- **R9 — The mack/fold in-compile interpreter is a second noun boundary** (§3.9).
  honk runs nockvm during compile; native `Formula` must `to_noun` before
  `interpret`. *Mitigation:* land this in Phase 1 with the formula IR; test fold-
  heavy inputs (`^~`) in the corpus; re-key the fold cache on native identity.
- **R10 — Eager cores don't terminate; laziness can't be cleanly deferred**
  (§3.4, Phase 2). *Mitigation:* pick a Phase-2 lazy bridge up front; treat the
  native `LazyBattery` as possibly Phase-2 work, not strictly Phase-3.
- **R11 — Hint-kind conflation breaks `%fast`/`%spot`/`%note`.** Distinct
  encodings and the `%fast` jet-registry side effect. *Mitigation:* split the
  Formula variants (done in §3.2); fixtures per hint kind in the Phase-1 gate.
- **R12 — Auras / face names / fork ordering must intern canonically.** Auras and
  names → interned symbols; `Fork` → canonical-ordered set so equal forks intern
  equal; genes → `Rc<Hoon>` that derives `Hash`/`Eq` (or is itself interned).
  *Mitigation:* the §3.6 debug assertion (no structurally-equal-but-pointer-
  distinct live `Rc<Type>`) catches canonicality violations.

---

## 8. Interaction with existing work / branch hygiene

- **Branch:** cut fresh from the integration point (current branch tip or master,
  per the team's base). This is a parallel track.
- **Carry over:** the parity harness (`jam_diff --kernel-parity`,
  `just honk-parity`), the timing harness, the cache-context correctness fixes
  (H2), the nockvm overhead cleanup (N1–N6), the embedded-prelude path (unchanged
  shipping route).
- **Drop / supersede on the new branch:** the H7 frame-arena (region stack,
  `pop_frame_preserving`, `copy_to_base`, `invalidate_frame_caches`) and the
  arena profiling counters — `Rc`/`Drop` replaces them; keep them only as
  reference in git history (they remain committed on the prior branch). The
  uncommitted Step-2 chunked-prelude WIP in `honk.rs` is obsolete — drop it.
- **Embedded prelude stays** as the shipping path throughout; native self-hosting
  (dropping the hoonc-built prelude asset) only becomes feasible *after* Phase 3
  bounds the giant-core mint, and is a separate decision even then.

---

## 9. Sizing (rough, calibrate after Phase 1)

| Phase | Scope | Rough size |
|---|---|---|
| 0 | scaffolding + oracle + corpus | ~0.5–1 wk |
| 1 | Formula IR + to_noun + **all mint & play emit sites** + fold boundary | ~1–2 wks |
| 2 | Type enum + intern + ~476 type sites + AST round-trip removal + lazy bridge | ~3–4 wks |
| 3 | lazy/hold/fan native, resolver-id removal (long pole; decides A2) | ~2–4 wks |
| 4 | cache redesign + perf (recover 60s gate) | ~1 wk |
| 5 | Dynock type to_noun | ~days |
| 6 | retire noun ut + cleanup | ~0.5–1 wk |

Total order-of-magnitude: **~8–12 weeks** (revised up after review — Phase 1's
formula-site port and the AST round-trip removal in Phase 2 were under-counted,
and the lazy bridge blurs the 2/3 boundary). Phase 1 is the cheap proof of the
byte-exact boundary; **Phase 3 decides success (A2)** — front-load risk there.

---

## 10. First concrete steps (week 1)

1. Cut the branch; add `native_ir` module skeleton.
2. Build the dual-run harness + `just native-parity-dual`; wire the corpus.
3. Implement `enum Formula` + `to_noun`; assert `to_noun == current noun formula`
   on the formula fixtures and the 5 passing kernels.
4. (In parallel, on the *base* branch or first on this one) close the roswell
   wet-polymorphism gap so the oracle is 6/6 (R2).

The decision gate for the whole effort is **Phase 3 / A2**: does native + lazy
cores + hash-consing bound the hoon-138 mint? Everything before it is valuable
regardless (parity preserved, bug class reduced, faster), but A2 is the prize.
