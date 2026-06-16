//! Provenanced owned leaves for the native IR (plan §3.9, RT-04).
//!
//! Formulas (and, for typed Dynock, types) carry noun leaves: quoted constants
//! (`[1 const]`), hint clues, dbug spot tuples. A **bare `Noun`** leaf is an
//! alien-pointer hazard, so the IR carries an owned, provenance-free
//! representation: a small direct atom inline, or owned jam bytes for anything
//! larger. `to_noun` materializes the leaf into a destination slab through a
//! checked copy (never splicing a foreign pointer). The production copy-cache
//! lives in `docs/native-compiler/PHASE0-PROVENANCE-DESIGN.md`; this is the
//! Phase-1 shadow form.
#![allow(dead_code)]

use std::sync::Arc;

use bytes::Bytes;
use nockapp::noun::slab::NounSlab;
use nockvm::noun::{Atom, Noun, NounAllocator, NounSpace};

/// An owned, provenance-safe noun leaf.
#[derive(Clone)]
pub enum Leaf {
    /// A direct (≤ 63-bit) atom, stored inline.
    Direct(u64),
    /// Anything larger (big atoms, cells) as owned jam bytes — provenance-free,
    /// cued into the destination slab by `to_noun`.
    Jammed(Arc<[u8]>),
}

impl Leaf {
    /// Capture `noun` (resolved in `space`) as an owned, provenance-free leaf.
    pub fn from_noun(noun: Noun, space: &NounSpace) -> Self {
        if let Ok(atom) = noun.in_space(space).as_atom() {
            if let Ok(v) = atom.as_u64() {
                return Leaf::Direct(v);
            }
        }
        // Larger / cell: jam through a scratch slab so we own the bytes.
        let mut scratch: NounSlab = NounSlab::new();
        scratch.copy_into(noun, space);
        Leaf::Jammed(Arc::from(&scratch.jam()[..]))
    }

    /// Materialize the leaf into `dst` via a checked copy (no foreign pointer).
    pub fn to_noun(&self, dst: &mut NounSlab) -> Noun {
        match self {
            Leaf::Direct(v) => Atom::new(dst, *v).as_noun(),
            Leaf::Jammed(bytes) => {
                let mut scratch: NounSlab = NounSlab::new();
                let cued = scratch
                    .cue_into(Bytes::copy_from_slice(bytes))
                    .expect("leaf jam bytes must cue");
                dst.copy_into(cued, &scratch.noun_space())
            }
        }
    }
}
