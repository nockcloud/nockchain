//! Native Nock formula IR (plan §3.2).
//!
//! Maps to Nock opcodes. `Rc`-shared for battery/subformula sharing. The
//! builders (`cons`/`comb`/`cond` analogues) MUST reproduce honk's hoon-138
//! peephole rewrites (RT-09) and emit byte-exact Nock via [`super::ToNoun`].
//! Hint kinds are split because `%fast`/`%spot`/`%note` differ in both encoding
//! and compile-time semantics (RT-12). Axes are arbitrary atoms, not `u64`
//! (RT-08). The exact variant set is finalized in Phase 1 against the emit-site
//! inventory; this is the skeleton.
#![allow(dead_code)]

use std::rc::Rc;

use num_bigint::BigUint;

use super::leaf::Leaf;

/// A Nock axis. Axes can be arbitrary-size atoms (Nock 0/9/10), not just `u64`
/// (RT-08; honk has `slot_formula_axis_big` + BigUint helpers, the interpreter
/// stores axes as `Atom`).
#[derive(Clone, Debug)]
pub enum Axis {
    Small(u64),
    Big(Rc<BigUint>),
}

/// A Nock formula.
pub enum Formula {
    /// `[0 axis]`
    Slot(Axis),
    /// `[1 const]` — a provenanced constant leaf.
    Quote(Leaf),
    /// `[2 subject formula]`
    Eval(Rc<Formula>, Rc<Formula>),
    /// autocons `[f g]`
    Cell(Rc<Formula>, Rc<Formula>),
    /// `[6 p q r]`
    Cond(Rc<Formula>, Rc<Formula>, Rc<Formula>),
    /// `[9 axis core]`
    Kick { axis: Axis, core: Rc<Formula> },
    /// `[10 [axis value] target]`
    Edit {
        axis: Axis,
        value: Rc<Formula>,
        target: Rc<Formula>,
    },
    /// `[11 [%fast clue] body]` — jet registration; a runtime-state contract
    /// (plan §3.11, RT-12), not merely an encoding.
    JetHint { clue: Leaf, body: Rc<Formula> },
    /// `[11 note body]` / `[12 …]` — type/typo notes (also emitted by the play
    /// path, RT-09).
    NoteHint { note: Leaf, body: Rc<Formula> },
    /// `[11 spot body]` — source location; stack-ordered, byte-exact (RT-15).
    Dbug { spot: Leaf, body: Rc<Formula> },
    /// Remaining Nock opcodes (3/4/5/7/8/12 …) pending the Phase-1 emit-site
    /// inventory; modeled generically until each is given a typed variant.
    Op {
        code: u8,
        args: Vec<Rc<Formula>>,
    },
}
