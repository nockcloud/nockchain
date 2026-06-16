//! Provenanced owned leaves for the native IR (plan §3.9, RT-04).
//!
//! Formulas (and, for typed Dynock, types) carry noun leaves: quoted constants
//! (`[1 const]`), hint clues, and dbug spot tuples. A **bare `Noun`** leaf is an
//! alien-pointer hazard — nouns depend on a private slab lifetime, `set_root`
//! panics on foreign-provenance roots, and release-mode resolution skips range
//! checks — so the IR forbids bare nouns and carries provenance instead. The
//! concrete owner representation is finalized in
//! `docs/native-compiler/PHASE0-PROVENANCE-DESIGN.md`; the safe default is owned
//! jam bytes.
#![allow(dead_code)]

use std::sync::Arc;

/// An owned, provenance-safe noun leaf.
#[derive(Clone)]
pub enum Leaf {
    /// Owned jam bytes — provenance-free and always safe to cue into a
    /// destination slab. The default representation for the skeleton.
    Jammed(Arc<[u8]>),
    /// A small direct atom value that needs no slab allocation.
    Direct(u64),
    // Phase-0 design (PHASE0-PROVENANCE-DESIGN.md) may add a
    // `Provenanced { root, owner }` variant once the SourceSpace/Brand owner
    // token is defined; until then a leaf that must hold an allocated noun is
    // represented as `Jammed` so it carries no foreign pointer.
}
