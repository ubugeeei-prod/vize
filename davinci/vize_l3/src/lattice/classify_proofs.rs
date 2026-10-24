//! The laws of the L3 declarative-rule evaluator, `classify` in `axis.rs`.
//!
//! This module is part of the `l3_lattice.rs` Kani harness, which includes
//! the production sources. It replaces the `classify_*` theorems of the Lean
//! module `Impeto.LatticeLaws`.

use super::ReactivityClass::{PropsStable, Reactive, Static, Unstable};
use super::axis::classify;
use super::proofs::{EFFECTS, any_class, any_effect, effect_demand, effects};
use super::{BindingOrigin, EffectSet, EscapeKind, ReactivityClass};

fn any_origin() -> BindingOrigin {
    match kani::any::<u8>() % 4 {
        0 => BindingOrigin::Local,
        1 => BindingOrigin::Prop,
        2 => BindingOrigin::ProvideInject,
        _ => BindingOrigin::TemplateRef,
    }
}

fn any_escape() -> EscapeKind {
    match kani::any::<u8>() % 4 {
        0 => EscapeKind::None,
        1 => EscapeKind::Returned,
        2 => EscapeKind::Stored,
        _ => EscapeKind::Global,
    }
}

/// Rule table: where the value enters the component.
fn origin_demand(origin: BindingOrigin) -> ReactivityClass {
    match origin {
        BindingOrigin::Local => Static,
        BindingOrigin::Prop => PropsStable,
        BindingOrigin::ProvideInject | BindingOrigin::TemplateRef => Reactive,
    }
}

/// Rule table: each escape demotes toward `unstable`.
fn escape_demand(escape: EscapeKind) -> ReactivityClass {
    match escape {
        EscapeKind::None => Static,
        EscapeKind::Returned => PropsStable,
        EscapeKind::Stored => Reactive,
        EscapeKind::Global => Unstable,
    }
}

/// One binding summary on the value axis (Lean `Input` without its verdict).
#[derive(Clone, Copy)]
struct Input {
    origin: BindingOrigin,
    mask: u8,
    escape: EscapeKind,
}

impl Input {
    fn any() -> Self {
        Self {
            origin: any_origin(),
            mask: kani::any(),
            escape: any_escape(),
        }
    }

    fn class(self) -> ReactivityClass {
        classify(self.origin, effects(self.mask), self.escape)
    }

    /// The class meets every lower bound the rule table demands for this
    /// binding (Lean `Admissible` over `Demand`).
    fn admits(self, bound: ReactivityClass) -> bool {
        let mut admitted = origin_demand(self.origin) <= bound
            && escape_demand(self.escape) <= bound
            && (!matches!(self.origin, BindingOrigin::ProvideInject) || Reactive <= bound);
        let mut index = 0;
        while index < EFFECTS.len() {
            if self.mask & (1 << index) != 0 {
                admitted &= effect_demand(EFFECTS[index]) <= bound;
            }
            index += 1;
        }
        admitted
    }

    /// The join of every floor, before the provide/inject cap (Lean `base`).
    fn base(self) -> ReactivityClass {
        origin_demand(self.origin)
            .join(effects(self.mask).class_floor())
            .join(escape_demand(self.escape))
    }
}

/// Lean `classify_admissible`, `classify_least` and `classify_spec`: the
/// evaluator returns exactly the least class that meets every demand of the
/// rule table.
#[kani::proof]
#[kani::unwind(10)]
fn classify_is_the_least_admissible_class() {
    let input = Input::any();
    let bound = any_class();
    assert!(input.admits(input.class()));
    assert_eq!(input.class() <= bound, input.admits(bound));
}

/// Lean `classify_eq`, `base_le_classify`, `provideInject_capped` and
/// `provideInject_cap_redundant`.
#[kani::proof]
#[kani::unwind(10)]
fn the_provide_inject_cap_never_lowers_and_is_already_implied() {
    let input = Input::any();
    let capped = if matches!(input.origin, BindingOrigin::ProvideInject) {
        input.base().join(Reactive)
    } else {
        input.base()
    };
    assert_eq!(input.class(), capped);
    assert!(input.base() <= input.class());
    if matches!(input.origin, BindingOrigin::ProvideInject) {
        assert!(Reactive <= input.class());
    }
    assert_eq!(input.class(), input.base());
    assert_eq!(input.origin.class_floor(), origin_demand(input.origin));
    assert_eq!(input.escape.class_floor(), escape_demand(input.escape));
}

/// Lean `classify_monotone`, `classify_add_effect` and
/// `classify_escape_demotes`: more effects, a weaker origin or a wider escape
/// never make a binding more stable.
#[kani::proof]
#[kani::unwind(10)]
fn classification_is_monotone() {
    let (weaker, stronger) = (Input::any(), Input::any());
    if origin_demand(weaker.origin) <= origin_demand(stronger.origin)
        && weaker.mask & stronger.mask == weaker.mask
        && escape_demand(weaker.escape) <= escape_demand(stronger.escape)
    {
        assert!(weaker.class() <= stronger.class());
    }

    let effect = any_effect();
    let with_effect = classify(
        weaker.origin,
        effects(weaker.mask).with(effect),
        weaker.escape,
    );
    assert!(weaker.class() <= with_effect);

    let escape = any_escape();
    if escape_demand(weaker.escape) <= escape_demand(escape) {
        assert!(weaker.class() <= classify(weaker.origin, effects(weaker.mask), escape));
    }
}

/// Lean `classify_effects_extensional`: effect sets are sets. Only membership
/// is observable, whatever order the effects were added in.
#[kani::proof]
#[kani::unwind(10)]
fn classification_reads_effect_membership_only() {
    let input = Input::any();
    let mut reversed = EffectSet::empty();
    let mut index = EFFECTS.len();
    while index > 0 {
        index -= 1;
        if input.mask & (1 << index) != 0 {
            reversed = reversed.with(EFFECTS[index]).with(EFFECTS[index]);
        }
    }
    assert_eq!(reversed, effects(input.mask));
    assert_eq!(
        classify(input.origin, reversed, input.escape),
        input.class()
    );
}

/// Lean `classify_static_iff`: `static` is reachable only from an inert local
/// binding.
#[kani::proof]
#[kani::unwind(10)]
fn only_an_inert_local_binding_is_static() {
    let input = Input::any();
    let inert_local = matches!(input.origin, BindingOrigin::Local)
        && input.mask == 0
        && matches!(input.escape, EscapeKind::None);
    assert_eq!(input.class() == Static, inert_local);
}
