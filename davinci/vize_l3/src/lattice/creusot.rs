//! Creusot proves the laws of the production L3 lattice on its shipped
//! source. This file is the library root of `tests/formal/creusot/lattice`
//! and is not part of `vize_l3`'s module tree, as `kani.rs` beside it is not:
//! the modules below are the ones the product compiles, found by ordinary
//! module discovery.
//!
//! `ReactivityClass::join` carries its contract in the shipped source: it
//! returns `join_logic`, the higher position on the chain. The lemmas below
//! are the value-axis laws over that contract, for every class. See
//! docs/davinci/decisions/2026-11-17-creusot.md.

mod class;

pub use class::ReactivityClass;

use creusot_std::prelude::*;
use ReactivityClass::{PropsStable, Reactive, Static, Unstable};

/// Lean `Class.le_refl`, `le_trans`, `le_antisymm`, `le_total`, `static_le`,
/// `le_unstable` and `chain` (Kani `the_value_axis_is_a_strict_chain`): the
/// positions are `0..=3`, distinct classes have distinct positions, and the
/// four classes form the chain `static < props-stable < reactive < unstable`.
#[logic]
#[ensures(0 <= a.position() && a.position() <= 3)]
#[ensures(a.position() == b.position() ==> a == b)]
#[ensures(Static.position() <= a.position() && a.position() <= Unstable.position())]
#[ensures(Static.position() < PropsStable.position())]
#[ensures(PropsStable.position() < Reactive.position())]
#[ensures(Reactive.position() < Unstable.position())]
pub fn the_value_axis_is_a_strict_chain(a: ReactivityClass, b: ReactivityClass) {}

/// Lean `Class.join_comm`, `join_assoc`, `join_idem`, `le_join_left`,
/// `le_join_right` and `join_le` (Kani `join_is_a_least_upper_bound`).
#[logic]
#[ensures(a.join_logic(b).position() >= a.position())]
#[ensures(a.join_logic(b).position() >= b.position())]
#[ensures(a.join_logic(b) == b.join_logic(a))]
#[ensures(a.join_logic(a) == a)]
#[ensures(a.join_logic(b).join_logic(c) == a.join_logic(b.join_logic(c)))]
#[ensures(c.position() >= a.position() && c.position() >= b.position()
    ==> c.position() >= a.join_logic(b).position())]
pub fn join_is_a_least_upper_bound(a: ReactivityClass, b: ReactivityClass, c: ReactivityClass) {}

/// Lean `Class.static_join`, `join_static`, `unstable_join`, `join_unstable`,
/// `join_le_iff`, `join_eq_right_iff`, `rank_join` and `join_mono` (Kani
/// `join_has_its_units_and_is_monotone`).
#[logic]
#[ensures(Static.join_logic(a) == a && a.join_logic(Static) == a)]
#[ensures(Unstable.join_logic(a) == Unstable && a.join_logic(Unstable) == Unstable)]
#[ensures((a.join_logic(b).position() <= c.position())
    == (a.position() <= c.position() && b.position() <= c.position()))]
#[ensures((a.join_logic(b) == b) == (a.position() <= b.position()))]
#[ensures(a.join_logic(b).position()
    == if a.position() >= b.position() { a.position() } else { b.position() })]
#[ensures(a.position() <= c.position() && b.position() <= d.position()
    ==> a.join_logic(b).position() <= c.join_logic(d).position())]
pub fn join_has_its_units_and_is_monotone(
    a: ReactivityClass,
    b: ReactivityClass,
    c: ReactivityClass,
    d: ReactivityClass,
) {
}
