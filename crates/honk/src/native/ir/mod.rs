//! Native IR for honk's Hoon type system and Nock output.
//!
//! This is the foundation of the native-types migration — see
//! `docs/native-compiler/NATIVE-TYPES-MIGRATION.md`. It replaces the noun-based
//! working representation of compiler **types** and **formulas** with native
//! Rust enums (`Rc`-shared, hash-consed), emitting nouns only at provenanced
//! boundaries via [`ToNoun`].
//!
//! STATUS: Phase 0 SKELETON. These are the type definitions and module layout
//! only — there is no algorithm yet (mint/nest/play/`to_noun` land in Phases
//! 1–2). The skeleton exists so the shapes are fixed, compile, and can be
//! reviewed before the large port begins. Nothing in `crate::native::ut`
//! depends on this yet.
#![allow(dead_code)]

pub mod core;
pub mod formula;
pub mod intern;
pub mod leaf;
pub mod ty;

use nockapp::noun::slab::NounSlab;
use nockvm::noun::Noun;

/// Emit a native IR node to a **byte-exact** Noun in `dst`.
///
/// Implemented for [`formula::Formula`] in Phase 1 (always needed) and for
/// [`ty::Type`] in Phase 5 (typed Dynock only). Leaves are deep-copied into
/// `dst` through a checked, provenance-aware API (never splicing a foreign
/// pointer) and via a destination-slab copy cache to avoid re-copying large
/// shared leaves — see `docs/native-compiler/PHASE0-PROVENANCE-DESIGN.md`
/// (plan §3.9, RT-04, RT-16). The cache parameter is added when Phase 1 lands;
/// this Phase-0 signature is intentionally minimal.
pub trait ToNoun {
    fn to_noun(&self, dst: &mut NounSlab) -> Noun;
}
