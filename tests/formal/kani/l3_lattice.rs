//! Kani checks the production L3 lattice implementation directly: the value
//! axis (`class.rs`), the effect vocabulary (`effect.rs`) and the
//! declarative-rule evaluator over the origin and escape axes (`axis.rs`).
//!
//! These harnesses replace the Lean module `Impeto.LatticeLaws`. Each one
//! names the Lean theorems it replaces. The rule table below is written from
//! the declarative rules in `docs/davinci/plan/phase-3.md`, never from the
//! Rust source, so the evaluator is checked against an independent statement.

extern crate core;

#[path = "../../../davinci/vize_l3/src/lattice/axis.rs"]
mod axis;
#[path = "../../../davinci/vize_l3/src/lattice/class.rs"]
mod class;
#[path = "../../../davinci/vize_l3/src/lattice/effect.rs"]
mod effect;

pub use axis::{BindingOrigin, EscapeKind};
pub use class::ReactivityClass;
use effect::{EffectKind, EffectSet};

#[cfg(kani)]
mod proofs {
    use super::axis::classify;
    use super::{BindingOrigin, EffectKind, EffectSet, EscapeKind, ReactivityClass};
    use ReactivityClass::{PropsStable, Reactive, Static, Unstable};

    const EFFECTS: [EffectKind; 8] = [
        EffectKind::Freeze,
        EffectKind::Capture,
        EffectKind::ReadProp,
        EffectKind::ReadReactive,
        EffectKind::MutateLocal,
        EffectKind::MutateGlobal,
        EffectKind::CallUnknown,
        EffectKind::Allocate,
    ];

    fn class(rank: u8) -> ReactivityClass {
        match rank {
            0 => Static,
            1 => PropsStable,
            2 => Reactive,
            _ => Unstable,
        }
    }

    fn any_class() -> ReactivityClass {
        let rank: u8 = kani::any();
        kani::assume(rank < 4);
        class(rank)
    }

    fn any_effect() -> EffectKind {
        let index: usize = kani::any();
        kani::assume(index < EFFECTS.len());
        EFFECTS[index]
    }

    fn effects(mask: u8) -> EffectSet {
        let mut set = EffectSet::empty();
        let mut index = 0;
        while index < EFFECTS.len() {
            if mask & (1 << index) != 0 {
                set = set.with(EFFECTS[index]);
            }
            index += 1;
        }
        set
    }

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

    /// The position of a class on the chain (Lean `Class.rank`).
    fn rank(class: ReactivityClass) -> u8 {
        match class {
            Static => 0,
            PropsStable => 1,
            Reactive => 2,
            Unstable => 3,
        }
    }

    /// Rule table: a binding carrying this effect is at least this unstable.
    fn effect_demand(effect: EffectKind) -> ReactivityClass {
        match effect {
            EffectKind::Freeze | EffectKind::ReadProp | EffectKind::Allocate => PropsStable,
            EffectKind::Capture | EffectKind::ReadReactive | EffectKind::MutateLocal => Reactive,
            EffectKind::MutateGlobal | EffectKind::CallUnknown => Unstable,
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

    /// Lean `Class.le_refl`, `le_trans`, `le_antisymm`, `le_total`,
    /// `static_le`, `le_unstable` and `chain`.
    #[kani::proof]
    fn the_value_axis_is_a_strict_chain() {
        let (a, b, c) = (any_class(), any_class(), any_class());
        assert!(a <= a);
        if a <= b && b <= c {
            assert!(a <= c);
        }
        if a <= b && b <= a {
            assert_eq!(a, b);
        }
        assert!(a <= b || b <= a);
        assert!(Static <= a && a <= Unstable);
        assert!(Static < PropsStable && PropsStable < Reactive && Reactive < Unstable);
        assert_eq!(a <= b, rank(a) <= rank(b));
    }

    /// Lean `Class.join_comm`, `join_assoc`, `join_idem`, `le_join_left`,
    /// `le_join_right` and `join_le`.
    #[kani::proof]
    fn join_is_a_least_upper_bound() {
        let (a, b, c) = (any_class(), any_class(), any_class());
        let joined = a.join(b);
        assert!(joined >= a && joined >= b);
        assert_eq!(joined, b.join(a));
        assert_eq!(a.join(a), a);
        assert_eq!(joined.join(c), a.join(b.join(c)));
        if c >= a && c >= b {
            assert!(c >= joined);
        }
    }

    /// Lean `Class.static_join`, `join_static`, `unstable_join`,
    /// `join_unstable`, `join_le_iff`, `join_eq_right_iff`, `rank_join` and
    /// `join_mono`.
    #[kani::proof]
    fn join_has_its_units_and_is_monotone() {
        let (a, b, c, d) = (any_class(), any_class(), any_class(), any_class());
        assert_eq!(Static.join(a), a);
        assert_eq!(a.join(Static), a);
        assert_eq!(Unstable.join(a), Unstable);
        assert_eq!(a.join(Unstable), Unstable);
        assert_eq!(a.join(b) <= c, a <= c && b <= c);
        assert_eq!(a.join(b) == b, a <= b);
        assert_eq!(rank(a.join(b)), rank(a).max(rank(b)));
        if a <= c && b <= d {
            assert!(a.join(b) <= c.join(d));
        }
    }

    /// Lean `effectsFloor_eq_tier`: the tiered floor the evaluator ships is
    /// the join of the floor the rule table gives each effect in the set.
    #[kani::proof]
    #[kani::unwind(10)]
    fn the_effect_floor_is_the_join_of_the_rule_table() {
        let mask: u8 = kani::any();
        let mut joined = Static;
        let mut index = 0;
        while index < EFFECTS.len() {
            if mask & (1 << index) != 0 {
                assert!(effects(mask).contains(EFFECTS[index]));
                joined = joined.join(effect_demand(EFFECTS[index]));
            }
            index += 1;
        }
        assert_eq!(effects(mask).class_floor(), joined);
        assert_eq!(EffectSet::empty().class_floor(), Static);
        assert_eq!(effects(mask).is_empty(), mask == 0);
    }

    /// Lean `effectsFloor_cons` and `floor_le_effectsFloor`.
    #[kani::proof]
    #[kani::unwind(10)]
    fn adding_an_effect_joins_its_floor() {
        let set = effects(kani::any());
        let effect = any_effect();
        let floor = EffectSet::one(effect).class_floor();
        assert_eq!(floor, effect_demand(effect));
        assert_eq!(
            set.with(effect).class_floor(),
            floor.join(set.class_floor())
        );
        if set.contains(effect) {
            assert!(floor <= set.class_floor());
        }
    }

    /// Lean `effectsFloor_append` and `effectsFloor_mono`: the floor is a
    /// join homomorphism from set union, so more effects never make a
    /// binding more stable.
    #[kani::proof]
    #[kani::unwind(10)]
    fn effect_union_preserves_the_class_floor() {
        let (left_mask, right_mask): (u8, u8) = (kani::any(), kani::any());
        let (left, right) = (effects(left_mask), effects(right_mask));
        let union = left.union(right);
        assert_eq!(
            union.class_floor(),
            left.class_floor().join(right.class_floor())
        );
        assert!(union.class_floor() >= left.class_floor());
        assert!(union.class_floor() >= right.class_floor());
        if left_mask & right_mask == left_mask {
            assert!(left.class_floor() <= right.class_floor());
        }
    }

    /// Lean `effectsFloor_le`: a bound above every member's floor is above
    /// the floor of the set.
    #[kani::proof]
    #[kani::unwind(10)]
    fn a_bound_on_every_effect_bounds_the_set() {
        let mask: u8 = kani::any();
        let bound = any_class();
        let mut every_member_is_bounded = true;
        let mut index = 0;
        while index < EFFECTS.len() {
            if mask & (1 << index) != 0 {
                every_member_is_bounded &= effect_demand(EFFECTS[index]) <= bound;
            }
            index += 1;
        }
        assert_eq!(
            effects(mask).class_floor() <= bound,
            every_member_is_bounded
        );
    }

    /// Lean `classify_admissible`, `classify_least` and `classify_spec`: the
    /// evaluator returns exactly the least class that meets every demand of
    /// the rule table.
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
    /// `classify_escape_demotes`: more effects, a weaker origin or a wider
    /// escape never make a binding more stable.
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

    /// Lean `classify_effects_extensional`: effect sets are sets. Only
    /// membership is observable, whatever order the effects were added in.
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

    /// Lean `classify_static_iff`: `static` is reachable only from an inert
    /// local binding.
    #[kani::proof]
    #[kani::unwind(10)]
    fn only_an_inert_local_binding_is_static() {
        let input = Input::any();
        let inert_local = matches!(input.origin, BindingOrigin::Local)
            && input.mask == 0
            && matches!(input.escape, EscapeKind::None);
        assert_eq!(input.class() == Static, inert_local);
    }
}
