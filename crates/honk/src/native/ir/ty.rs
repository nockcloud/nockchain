//! Native Hoon type IR (plan §3.3).
//!
//! Fully native (no noun leaves): children are `Rc<Type>`, auras/face-names are
//! interned symbols, `%hold` genes are native AST. Canonicalized by the
//! hash-cons [`super::intern::TypeTable`] so structural equality == pointer
//! equality. Constructors are NORMALIZING smart constructors (face(void)→void,
//! hint collapses, core(void)→void, fork flatten/omit) — a naive 1:1 enum would
//! change behavior (RT-06); those invariants land with the `TypeTable` in
//! Phase 2. This is the data skeleton.
#![allow(dead_code)]

use std::rc::Rc;

use num_bigint::BigUint;

use super::core::{Core, ForkSet, Hold};

/// An interned symbol (aura, face name). `Rc<str>` for the skeleton; may become
/// a small interned id in Phase 2.
pub type Sym = Rc<str>;

/// A Hoon compiler type. The nine variants mirror honk's current noun tags
/// (`%void %noun %atom %cell %core %face %fork %hint %hold`).
pub enum Type {
    Void,
    Noun,
    Atom {
        aura: Sym,
        constant: Option<Rc<BigUint>>,
    },
    Cell(Rc<Type>, Rc<Type>),
    Face {
        name: Sym,
        inner: Rc<Type>,
    },
    /// `%hint` wrapper — exact payload modeling (and its collapse rules) lands
    /// with the smart constructors in Phase 2.
    Hint {
        inner: Rc<Type>,
    },
    /// `%fork` — internally a canonical-ordered set; `to_noun` rebuilds the exact
    /// Hoon mug-ordered treap for typed Dynock (plan §3.4, RT-07).
    Fork(ForkSet),
    Core(Rc<Core>),
    Hold(Rc<Hold>),
}
