//! Creusot proves the laws of the production L3 lattice on its shipped
//! source. This file is the library root of `tests/formal/creusot/lattice`
//! and is not part of `vize_l3`'s module tree: the modules below are the ones
//! the product compiles, found by ordinary module discovery.
//!
//! Each shipped function carries its contract in the shipped source, stated
//! from the P3-2 rule table of `docs/davinci/plan/phase-3.md`, never from its
//! body: `ReactivityClass::join` returns `join_logic`, the higher position on
//! the chain; `EffectSet::class_floor` returns `floor`, the join of every
//! member's demand; the set operations state membership (`has`); `classify`
//! returns `demanded`, the join of every demand the rule table makes. The
//! lemmas below are the laws over those contracts, for every value. See
//! docs/davinci/decisions/2026-11-17-creusot.md.

mod axis;
mod class;
mod effect;

pub use axis::{BindingOrigin, EscapeKind, classify, demanded};
pub use class::ReactivityClass;
pub use effect::{EffectKind, EffectSet};

use creusot_std::prelude::*;
use BindingOrigin::{Local, ProvideInject};
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

/// Lean `effectsFloor_eq_tier` (Kani
/// `the_effect_floor_is_the_join_of_the_rule_table`). The tiered
/// `EffectSet::class_floor` returns `floor`, the join of every member's
/// demand: that is its contract, which Why3 proves against the tiers. Over
/// it, every member's demand is below the floor, and a set without members
/// is static.
#[logic]
#[ensures(forall<k: EffectKind> s.has(k) ==> k.demand().position() <= s.floor().position())]
#[ensures((forall<k: EffectKind> !s.has(k)) ==> s.floor() == Static)]
pub fn the_effect_floor_is_the_join_of_the_rule_table(s: EffectSet) {}

/// Lean `effectsFloor_cons` and `floor_le_effectsFloor` (Kani
/// `adding_an_effect_joins_its_floor`). `EffectSet::one(e)` has exactly `e`
/// and `s.with(e)` has `s`'s members and `e` (their contracts): the floor of
/// the first is `e`'s demand, the floor of the second joins `e`'s demand into
/// `s`'s, and a member's demand is already below the floor.
#[logic]
#[ensures((forall<k: EffectKind> t.has(k) == (k == e)) ==> t.floor() == e.demand())]
#[ensures((forall<k: EffectKind> t.has(k) == (s.has(k) || k == e))
    ==> t.floor() == e.demand().join_logic(s.floor()))]
#[ensures(s.has(e) ==> e.demand().position() <= s.floor().position())]
pub fn adding_an_effect_joins_its_floor(s: EffectSet, t: EffectSet, e: EffectKind) {}

/// Lean `effectsFloor_append` and `effectsFloor_mono` (Kani
/// `effect_union_preserves_the_class_floor`): the floor is a join
/// homomorphism from `EffectSet::union` (whose contract is membership in
/// either set), so more effects never make a binding more stable.
#[logic]
#[ensures((forall<k: EffectKind> u.has(k) == (a.has(k) || b.has(k)))
    ==> u.floor() == a.floor().join_logic(b.floor()))]
#[ensures((forall<k: EffectKind> a.has(k) ==> b.has(k))
    ==> a.floor().position() <= b.floor().position())]
pub fn effect_union_preserves_the_class_floor(a: EffectSet, b: EffectSet, u: EffectSet) {}

/// Lean `effectsFloor_le` (Kani `a_bound_on_every_effect_bounds_the_set`): a
/// class bounds the floor exactly when it bounds every member's demand.
#[logic]
#[ensures((s.floor().position() <= c.position())
    == (forall<k: EffectKind> s.has(k) ==> k.demand().position() <= c.position()))]
pub fn a_bound_on_every_effect_bounds_the_set(s: EffectSet, c: ReactivityClass) {}

/// The declarative P3-2 spec of a binding's class (Lean `Admissible` over
/// `Demand`): `c` meets the lower bound that the origin, every effect and
/// the escape demand, and a provide/inject origin demands at least
/// `reactive`.
#[logic(open)]
pub fn admits(
    origin: BindingOrigin,
    effects: EffectSet,
    escape: EscapeKind,
    c: ReactivityClass,
) -> bool {
    pearlite! {
        origin.demand().position() <= c.position()
            && (forall<k: EffectKind> effects.has(k) ==> k.demand().position() <= c.position())
            && escape.demand().position() <= c.position()
            && (origin == ProvideInject ==> Reactive.position() <= c.position())
    }
}

/// Lean `classify_admissible`, `classify_least` and `classify_spec` (Kani
/// `classify_is_the_least_admissible_class`): `classify` returns `demanded`
/// (its contract), which meets every demand of the rule table and is below
/// every class that does.
#[logic]
#[ensures(admits(o, s, e, demanded(o, s, e)))]
#[ensures((demanded(o, s, e).position() <= c.position()) == admits(o, s, e, c))]
pub fn classify_is_the_least_admissible_class(
    o: BindingOrigin,
    s: EffectSet,
    e: EscapeKind,
    c: ReactivityClass,
) {
}

/// Lean `classify_eq`, `base_le_classify`, `provideInject_capped` and
/// `provideInject_cap_redundant` (Kani
/// `the_provide_inject_cap_never_lowers_and_is_already_implied`): the class
/// is never below the join of the origin, effect and escape floors, a
/// provide/inject binding is at least `reactive`, and that cap changes
/// nothing, because the origin floor already demands it.
#[logic]
#[ensures(o.demand().join_logic(s.floor()).join_logic(e.demand()).position()
    <= demanded(o, s, e).position())]
#[ensures(o == ProvideInject ==> Reactive.position() <= demanded(o, s, e).position())]
#[ensures(demanded(o, s, e) == o.demand().join_logic(s.floor()).join_logic(e.demand()))]
pub fn the_provide_inject_cap_never_lowers_and_is_already_implied(
    o: BindingOrigin,
    s: EffectSet,
    e: EscapeKind,
) {
}

/// Lean `classify_monotone`, `classify_add_effect` and
/// `classify_escape_demotes` (Kani `classification_is_monotone`): a worse
/// origin, more effects (a superset, such as `s.with(k)`) or a worse escape
/// never make a binding more stable.
#[logic]
#[ensures(o1.demand().position() <= o2.demand().position()
    && (forall<k: EffectKind> s1.has(k) ==> s2.has(k))
    && e1.demand().position() <= e2.demand().position()
    ==> demanded(o1, s1, e1).position() <= demanded(o2, s2, e2).position())]
pub fn classification_is_monotone(
    o1: BindingOrigin,
    s1: EffectSet,
    e1: EscapeKind,
    o2: BindingOrigin,
    s2: EffectSet,
    e2: EscapeKind,
) {
}

/// Lean `classify_effects_extensional` (Kani
/// `classification_reads_effect_membership_only`): effect sets are sets.
/// Only membership is observable, whatever order the effects were added in
/// and however often.
#[logic]
#[ensures((forall<k: EffectKind> a.has(k) == b.has(k)) ==> demanded(o, a, e) == demanded(o, b, e))]
pub fn classification_reads_effect_membership_only(
    o: BindingOrigin,
    a: EffectSet,
    b: EffectSet,
    e: EscapeKind,
) {
}

/// Lean `classify_static_iff` (Kani `only_an_inert_local_binding_is_static`):
/// `static` is reachable only from a local binding without effects or
/// escape.
#[logic]
#[ensures((demanded(o, s, e) == Static)
    == (o == Local && (forall<k: EffectKind> !s.has(k)) && e == EscapeKind::None))]
pub fn only_an_inert_local_binding_is_static(o: BindingOrigin, s: EffectSet, e: EscapeKind) {}
