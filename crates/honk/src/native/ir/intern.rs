//! Hash-cons intern table for [`super::ty::Type`] (plan §3.3) — the keystone of
//! the memory win.
//!
//! Returns the canonical `Rc<Type>` for a structurally-equal type, so equal
//! subtrees — including pointer-distinct ones produced by minting — collapse to
//! ONE shared `Rc`. This is what fixes subject-deepening: the repeated embedded
//! subjects become a single shared node instead of O(N²) duplicated structure.
//!
//! Interning is BOTTOM-UP: children are interned first, so a node's hash/eq use
//! the children's canonical `Rc` IDENTITY (`Rc::ptr_eq`) plus its own leaf
//! content — O(1) per node (no deep recursion in the compare), O(total) overall.
//!
//! NOTE: this operates on the Phase-1 boundary `Type` (native skeleton + carried
//! leaves). It already dedups the native skeleton — crucially the recursive
//! payload/cell/inner/subject chains where subject-deepening lives. Phase 2
//! nativizes the carried leaves (coil/treap/gene) so the deduplication reaches
//! inside cores/forks too; the table mechanism here is unchanged by that.
#![allow(dead_code)]

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use nockapp::noun::slab::NounSlab;
use nockvm::noun::{Noun, NounSpace};

use super::leaf::Leaf;
use super::ty::Type;
use crate::errors::{CompilerError, Result};
use crate::native::noun::noun_pair;

/// Decode a type noun into the native IR AND intern it in one O(n) pass, using a
/// persistent pointer-identity `memo` so the noun DAG (and anything carried over
/// from a prior call) is walked at most once. Structurally-equal-but-pointer-
/// distinct subtrees — the duplicated embedded subjects of subject-deepening —
/// are collapsed by the table to one shared `Rc`. This is the O(n) construction
/// primitive the native-mint port is built on (the combinator O(n²) trap was
/// re-parsing per call with no shared memo; this shares one).
pub fn intern_type_noun(
    table: &mut TypeTable,
    memo: &mut HashMap<u64, Rc<Type>>,
    noun: Noun,
    space: &NounSpace,
) -> Result<Rc<Type>> {
    // %void / %noun are bare atom cords — directs, no memo needed.
    if let Ok(atom) = noun.in_space(space).as_atom() {
        if atom.eq_bytes(b"void") {
            return Ok(table.intern_shallow(Type::Void));
        }
        if atom.eq_bytes(b"noun") {
            return Ok(table.intern_shallow(Type::Noun));
        }
        return Err(CompilerError::Decode(
            "native type IR: unknown atom type tag".into(),
        ));
    }
    // SAFETY: `noun` is a live, in-`space` slab noun; `as_raw` only reads its
    // identity word (used purely as a memo key, never dereferenced).
    let raw = unsafe { noun.as_raw() };
    if let Some(rc) = memo.get(&raw) {
        return Ok(Rc::clone(rc));
    }
    let (tag, tail) = pair(noun, space)?;
    let tag = tag
        .in_space(space)
        .as_atom()
        .map_err(|_| CompilerError::Decode("native type IR: type tag not atom".into()))?;
    let node = if tag.eq_bytes(b"atom") {
        let (aura, bits) = pair(tail, space)?;
        Type::Atom {
            aura: Leaf::from_noun(aura, space),
            bits: Leaf::from_noun(bits, space),
        }
    } else if tag.eq_bytes(b"cell") {
        let (h, t) = pair(tail, space)?;
        Type::Cell(
            intern_type_noun(table, memo, h, space)?,
            intern_type_noun(table, memo, t, space)?,
        )
    } else if tag.eq_bytes(b"core") {
        let (payload, coil) = pair(tail, space)?;
        Type::Core {
            payload: intern_type_noun(table, memo, payload, space)?,
            coil: Leaf::from_noun(coil, space),
        }
    } else if tag.eq_bytes(b"face") {
        let (tool, inner) = pair(tail, space)?;
        Type::Face {
            tool: Leaf::from_noun(tool, space),
            inner: intern_type_noun(table, memo, inner, space)?,
        }
    } else if tag.eq_bytes(b"hint") {
        let (head, payload) = pair(tail, space)?;
        Type::Hint {
            head: Leaf::from_noun(head, space),
            payload: intern_type_noun(table, memo, payload, space)?,
        }
    } else if tag.eq_bytes(b"fork") {
        Type::Fork {
            set: Leaf::from_noun(tail, space),
        }
    } else if tag.eq_bytes(b"hold") {
        let (subject, gene) = pair(tail, space)?;
        Type::Hold {
            subject: intern_type_noun(table, memo, subject, space)?,
            gene: Leaf::from_noun(gene, space),
        }
    } else {
        return Err(CompilerError::Decode(
            "native type IR: unknown type tag".into(),
        ));
    };
    let interned = table.intern_shallow(node);
    memo.insert(raw, Rc::clone(&interned));
    Ok(interned)
}

fn pair(n: Noun, space: &NounSpace) -> Result<(Noun, Noun)> {
    noun_pair(n, space).map_err(|_| CompilerError::Decode("native type IR: bad cell".into()))
}

// ---------------------------------------------------------------------------
// Live native-mint construction-port harness (flag-gated by HONK_NATIVE_TYPES).
//
// This is the first real step of the construction port: as `mint` builds each
// core's type noun, we build the corresponding interned native type into one
// PERSISTENT table (shared pointer-memo across the whole compile). It runs
// alongside the noun path (which stays the live oracle), so it is additive and
// safe, and it measures the thing the whole migration turns on: how much the
// intern table collapses the mint-time type duplication (subject-deepening).
//
// Single-thread, single-compile harness — call `live_reset` at compile start.
// ---------------------------------------------------------------------------

struct LiveIntern {
    table: TypeTable,
    memo: HashMap<u64, Rc<Type>>,
    cores: u64,
    next_report: u64,
}

impl LiveIntern {
    fn new() -> Self {
        LiveIntern {
            table: TypeTable::new(),
            memo: HashMap::new(),
            cores: 0,
            next_report: 100_000,
        }
    }
}

std::thread_local! {
    static LIVE: std::cell::RefCell<Option<LiveIntern>> = const { std::cell::RefCell::new(None) };
}

static LIVE_ENABLED: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Whether the live native-type harness is on (`HONK_NATIVE_TYPES`), cached.
pub fn live_enabled() -> bool {
    use std::sync::atomic::Ordering;
    match LIVE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => {
            let on = std::env::var_os("HONK_NATIVE_TYPES").is_some();
            LIVE_ENABLED.store(if on { 1 } else { 2 }, Ordering::Relaxed);
            on
        }
    }
}

/// Reset the live table (call at the start of each top-level compile so stale
/// noun pointers from a prior compile can't alias). Unconditional: the native
/// construction helpers (`live_intern`/`native_of`) use the table regardless of
/// the measurement flag, so the table must be reset every compile.
pub fn live_reset() {
    LIVE.with(|cell| *cell.borrow_mut() = None);
}

/// Intern a single native node through the persistent thread-local table — the
/// ONE canonical pointer-identity universe shared by all native-shadow
/// construction (so `intern_shallow`'s children-by-`Rc`-pointer hashing stays
/// valid). Children must already be canonical `Rc<Type>` from this same table.
/// Works regardless of `HONK_NATIVE_TYPES` (the flag only gates the measurement
/// hook); it is only ever CALLED on the native-shadow path.
pub fn live_intern(node: Type) -> Rc<Type> {
    LIVE.with(|cell| {
        let mut slot = cell.borrow_mut();
        let st = slot.get_or_insert_with(LiveIntern::new);
        st.table.intern_shallow(node)
    })
}

/// Native-only cell constructor (collapse-aware): `cell(void,_)`/`cell(_,void)` ->
/// void, else `Cell`. For flipped producers that hold native children directly.
pub fn cons_cell(head: Rc<Type>, tail: Rc<Type>) -> Rc<Type> {
    if matches!(&*head, Type::Void) || matches!(&*tail, Type::Void) {
        return live_intern(Type::Void);
    }
    live_intern(Type::Cell(head, tail))
}

/// Native `%void` / `%noun`.
pub fn cons_void() -> Rc<Type> {
    live_intern(Type::Void)
}
pub fn cons_noun() -> Rc<Type> {
    live_intern(Type::Noun)
}

/// Collapse-aware native `%core`: `core(void,_)` -> void (mirrors ty_core_n).
pub fn cons_core(payload: Rc<Type>, coil: Leaf) -> Rc<Type> {
    if matches!(&*payload, Type::Void) {
        return live_intern(Type::Void);
    }
    live_intern(Type::Core { payload, coil })
}

/// Collapse-aware native `%face`: `face(_,void)` -> void (mirrors ty_face_tool_n).
pub fn cons_face(tool: Leaf, inner: Rc<Type>) -> Rc<Type> {
    if matches!(&*inner, Type::Void) {
        return live_intern(Type::Void);
    }
    live_intern(Type::Face { tool, inner })
}

/// Collapse-aware native `%hint`: `hint(_,void)` -> void, `hint(_,noun)` -> noun
/// (mirrors ty_hint_n).
pub fn cons_hint(head: Leaf, payload: Rc<Type>) -> Rc<Type> {
    match &*payload {
        Type::Void => live_intern(Type::Void),
        Type::Noun => live_intern(Type::Noun),
        _ => live_intern(Type::Hint { head, payload }),
    }
}

/// The O(n) fallback for a not-yet-threaded child: decode `noun` to its canonical
/// native `Rc<Type>` via the shared memoized walk. One shared `(table, memo)`
/// means each noun node is walked at most once per compile.
pub fn native_of(noun: Noun, space: &NounSpace) -> Result<Rc<Type>> {
    LIVE.with(|cell| {
        let mut slot = cell.borrow_mut();
        let st = slot.get_or_insert_with(LiveIntern::new);
        intern_type_noun(&mut st.table, &mut st.memo, noun, space)
    })
}

/// Live byte-exact oracle: panic unless `to_noun(native)` jams identically to the
/// `noun` it shadows. The per-node validation for the construction port.
pub fn assert_native_eq(noun: Noun, native: &Rc<Type>, space: &NounSpace) {
    let mut a: NounSlab = NounSlab::new();
    a.copy_into(noun, space);
    let ja = a.jam();
    let mut b: NounSlab = NounSlab::new();
    let rebuilt = native.to_noun(&mut b);
    b.set_root(rebuilt);
    let jb = b.jam();
    assert!(
        ja == jb,
        "native shadow mismatch: to_noun(native)={} bytes != noun={} bytes",
        jb.len(),
        ja.len()
    );
}

/// Build the interned native type for one minted core type noun.
pub fn live_intern_core_type(noun: Noun, space: &NounSpace) {
    if !live_enabled() {
        return;
    }
    LIVE.with(|cell| {
        let mut slot = cell.borrow_mut();
        let st = slot.get_or_insert_with(LiveIntern::new);
        if intern_type_noun(&mut st.table, &mut st.memo, noun, space).is_ok() {
            st.cores += 1;
            if st.table.interned_calls >= st.next_report {
                let t = &st.table;
                eprintln!(
                    "[native-types] cores={} walked={} distinct={} hits={} ({:.1}x dedup)",
                    st.cores,
                    t.interned_calls,
                    t.distinct,
                    t.hits,
                    t.interned_calls as f64 / t.distinct.max(1) as f64
                );
                st.next_report = t.interned_calls + 100_000;
            }
        }
    });
}

/// Emit the final dedup summary for the compile.
pub fn live_report_final() {
    if !live_enabled() {
        return;
    }
    LIVE.with(|cell| {
        if let Some(st) = cell.borrow().as_ref() {
            let t = &st.table;
            eprintln!(
                "[native-types] FINAL cores={} walked={} distinct={} hits={} ({:.1}x dedup)",
                st.cores,
                t.interned_calls,
                t.distinct,
                t.hits,
                t.interned_calls as f64 / t.distinct.max(1) as f64
            );
        } else {
            eprintln!("[native-types] FINAL: no cores interned (mint_core hook never fired on this thread)");
        }
    });
}

#[derive(Default)]
pub struct TypeTable {
    buckets: HashMap<u64, Vec<Rc<Type>>>,
    /// Total node-constructions seen by `intern` (the un-shared structural size).
    pub interned_calls: u64,
    /// Distinct canonical nodes retained (the hash-consed size).
    pub distinct: u64,
    /// Dedup hits (a structurally-equal node already existed).
    pub hits: u64,
}

impl TypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a type tree bottom-up, returning its canonical `Rc`.
    pub fn intern(&mut self, t: &Type) -> Rc<Type> {
        let node = match t {
            Type::Void => Type::Void,
            Type::Noun => Type::Noun,
            Type::Atom { aura, bits } => Type::Atom {
                aura: aura.clone(),
                bits: bits.clone(),
            },
            Type::Cell(h, tl) => Type::Cell(self.intern(h), self.intern(tl)),
            Type::Core { payload, coil } => Type::Core {
                payload: self.intern(payload),
                coil: coil.clone(),
            },
            Type::Face { tool, inner } => Type::Face {
                tool: tool.clone(),
                inner: self.intern(inner),
            },
            Type::Hint { head, payload } => Type::Hint {
                head: head.clone(),
                payload: self.intern(payload),
            },
            Type::Fork { set } => Type::Fork { set: set.clone() },
            Type::Hold { subject, gene } => Type::Hold {
                subject: self.intern(subject),
                gene: gene.clone(),
            },
        };
        self.intern_node(node)
    }

    /// Intern a single node whose children are ALREADY canonical (interned).
    /// O(1) amortized. Used by the memoized decode-and-intern walk
    /// ([`intern_type_noun`]) which interns bottom-up itself.
    pub fn intern_shallow(&mut self, node: Type) -> Rc<Type> {
        self.intern_node(node)
    }

    fn intern_node(&mut self, node: Type) -> Rc<Type> {
        self.interned_calls += 1;
        let h = node_hash(&node);
        if let Some(bucket) = self.buckets.get(&h) {
            for existing in bucket {
                if node_eq(existing, &node) {
                    self.hits += 1;
                    return Rc::clone(existing);
                }
            }
        }
        let rc = Rc::new(node);
        self.buckets.entry(h).or_default().push(Rc::clone(&rc));
        self.distinct += 1;
        rc
    }
}

/// Shallow structural hash: variant + children by canonical `Rc` pointer + leaf
/// content. Valid only when children are already interned (bottom-up).
fn node_hash(t: &Type) -> u64 {
    let mut h = DefaultHasher::new();
    std::mem::discriminant(t).hash(&mut h);
    let p = |rc: &Rc<Type>, h: &mut DefaultHasher| (Rc::as_ptr(rc) as usize).hash(h);
    match t {
        Type::Void | Type::Noun => {}
        Type::Atom { aura, bits } => {
            aura.hash(&mut h);
            bits.hash(&mut h);
        }
        Type::Cell(a, b) => {
            p(a, &mut h);
            p(b, &mut h);
        }
        Type::Core { payload, coil } => {
            p(payload, &mut h);
            coil.hash(&mut h);
        }
        Type::Face { tool, inner } => {
            tool.hash(&mut h);
            p(inner, &mut h);
        }
        Type::Hint { head, payload } => {
            head.hash(&mut h);
            p(payload, &mut h);
        }
        Type::Fork { set } => set.hash(&mut h),
        Type::Hold { subject, gene } => {
            p(subject, &mut h);
            gene.hash(&mut h);
        }
    }
    h.finish()
}

/// Shallow structural equality (children by canonical `Rc` identity).
fn node_eq(a: &Type, b: &Type) -> bool {
    use Type::*;
    match (a, b) {
        (Void, Void) | (Noun, Noun) => true,
        (Atom { aura: a1, bits: b1 }, Atom { aura: a2, bits: b2 }) => a1 == a2 && b1 == b2,
        (Cell(h1, t1), Cell(h2, t2)) => Rc::ptr_eq(h1, h2) && Rc::ptr_eq(t1, t2),
        (Core { payload: p1, coil: c1 }, Core { payload: p2, coil: c2 }) => {
            Rc::ptr_eq(p1, p2) && c1 == c2
        }
        (Face { tool: t1, inner: i1 }, Face { tool: t2, inner: i2 }) => {
            t1 == t2 && Rc::ptr_eq(i1, i2)
        }
        (Hint { head: h1, payload: p1 }, Hint { head: h2, payload: p2 }) => {
            h1 == h2 && Rc::ptr_eq(p1, p2)
        }
        (Fork { set: s1 }, Fork { set: s2 }) => s1 == s2,
        (Hold { subject: s1, gene: g1 }, Hold { subject: s2, gene: g2 }) => {
            Rc::ptr_eq(s1, s2) && g1 == g2
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::ir::leaf::Leaf;

    fn atom() -> Type {
        Type::Atom {
            aura: Leaf::Direct(100),
            bits: Leaf::Direct(0),
        }
    }

    #[test]
    fn dedups_structurally_equal_to_one_rc() {
        let mut tab = TypeTable::new();
        let r1 = tab.intern(&atom());
        let r2 = tab.intern(&atom()); // distinct allocation, equal structure
        assert!(Rc::ptr_eq(&r1, &r2), "equal atoms intern to one Rc");
        assert_eq!(tab.distinct, 1);
        assert_eq!(tab.hits, 1);

        // Equal cells over equal children also collapse.
        let c1 = Type::Cell(Rc::new(atom()), Rc::new(atom()));
        let c2 = Type::Cell(Rc::new(atom()), Rc::new(atom()));
        let rc1 = tab.intern(&c1);
        let rc2 = tab.intern(&c2);
        assert!(Rc::ptr_eq(&rc1, &rc2), "equal cells intern to one Rc");
        assert_eq!(tab.distinct, 2, "only the atom and the cell are distinct");
    }

    // The subject-deepening fix in miniature: a fully-duplicated balanced cell
    // tree of depth D has 2^(D+1)-1 structural nodes but only D+1 distinct after
    // hash-consing — O(2^n) → O(n).
    #[test]
    fn hash_consing_collapses_duplicated_structure() {
        fn build(depth: u32) -> Type {
            if depth == 0 {
                atom()
            } else {
                Type::Cell(Rc::new(build(depth - 1)), Rc::new(build(depth - 1)))
            }
        }
        let depth = 12;
        let mut tab = TypeTable::new();
        let _root = tab.intern(&build(depth));
        assert_eq!(
            tab.distinct as u32,
            depth + 1,
            "duplicated tree collapses to O(depth) distinct nodes"
        );
        assert_eq!(
            tab.interned_calls,
            (1u64 << (depth + 1)) - 1,
            "the full duplicated tree was walked"
        );
    }
}
