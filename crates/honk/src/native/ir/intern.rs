//! Hash-cons intern table for [`super::ty::Type`] (plan §3.3).
//!
//! Returns the canonical `Rc<Type>` for a structurally-equal type, giving
//! construction-time sharing (the memory win the noun path lacks — it dedups
//! only at jam time). Interning is BOTTOM-UP with a cached per-node structural
//! hash and an `Rc::ptr_eq` short-circuit, so each parent intern is O(1)
//! amortized (children are already canonical). The earlier top-down mug
//! interning regressed precisely because it walked whole cores and fell to full
//! compares on collisions — avoided here.
//!
//! Hard invariant: no two live `Rc<Type>` may be structurally-equal but
//! pointer-distinct (a debug assertion enforces this in Phase 2). A violation
//! silently breaks every identity-keyed memo and the fan scope.
//!
//! STATUS: Phase 0 skeleton — the table type and contract; `intern` lands in
//! Phase 2 with the normalizing smart constructors.
#![allow(dead_code)]

use std::collections::HashMap;
use std::rc::Rc;

use super::ty::Type;

#[derive(Default)]
pub struct TypeTable {
    /// Canonical types bucketed by cached structural hash (collisions resolved by
    /// structural compare, then shared by `Rc`).
    buckets: HashMap<u64, Vec<Rc<Type>>>,
}

impl TypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    // pub fn intern(&mut self, ty: Type) -> Rc<Type> { /* Phase 2 */ }
    // Smart constructors (interning + normalization invariants, RT-06) — Phase 2:
    //   fn cell(&mut self, h, t) -> Rc<Type>
    //   fn face(&mut self, name, inner) -> Rc<Type>   // face(void) -> void
    //   fn core(&mut self, payload, coil) -> Rc<Type> // core(void) -> void
    //   fn fork(&mut self, options) -> Rc<Type>       // flatten/omit void
    //   ...
}
