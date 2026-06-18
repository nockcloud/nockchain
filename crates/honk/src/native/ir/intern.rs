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

use super::ty::Type;

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
