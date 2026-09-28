//! Kani checks the production L3 lattice implementation directly.

#[path = "../../../crates/vize_l3/src/lattice/class.rs"]
mod class;
#[path = "../../../crates/vize_l3/src/lattice/effect.rs"]
mod effect;

pub use class::ReactivityClass;
use effect::{EffectKind, EffectSet};

#[cfg(kani)]
mod proofs {
    use super::{EffectKind, EffectSet, ReactivityClass};

    fn class(rank: u8) -> ReactivityClass {
        match rank {
            0 => ReactivityClass::Static,
            1 => ReactivityClass::PropsStable,
            2 => ReactivityClass::Reactive,
            _ => ReactivityClass::Unstable,
        }
    }

    fn effects(mask: u8) -> EffectSet {
        let mut set = EffectSet::empty();
        if mask & 1 != 0 {
            set = set.with(EffectKind::Freeze);
        }
        if mask & 2 != 0 {
            set = set.with(EffectKind::Capture);
        }
        if mask & 4 != 0 {
            set = set.with(EffectKind::ReadProp);
        }
        if mask & 8 != 0 {
            set = set.with(EffectKind::ReadReactive);
        }
        if mask & 16 != 0 {
            set = set.with(EffectKind::MutateLocal);
        }
        if mask & 32 != 0 {
            set = set.with(EffectKind::MutateGlobal);
        }
        if mask & 64 != 0 {
            set = set.with(EffectKind::CallUnknown);
        }
        if mask & 128 != 0 {
            set = set.with(EffectKind::Allocate);
        }
        set
    }

    #[kani::proof]
    fn join_is_a_least_upper_bound() {
        let a = class(kani::any());
        let b = class(kani::any());
        let c = class(kani::any());
        let joined = a.join(b);
        assert!(joined >= a && joined >= b);
        assert_eq!(joined, b.join(a));
        assert_eq!(a.join(a), a);
        assert_eq!(joined.join(c), a.join(b.join(c)));
        if c >= a && c >= b {
            assert!(c >= joined);
        }
    }

    #[kani::proof]
    fn effect_union_preserves_the_class_floor() {
        let left = effects(kani::any());
        let right = effects(kani::any());
        let union = left.union(right);
        assert_eq!(union.class_floor(), left.class_floor().join(right.class_floor()));
        assert!(union.class_floor() >= left.class_floor());
        assert!(union.class_floor() >= right.class_floor());
    }
}
