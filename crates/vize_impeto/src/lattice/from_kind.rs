//! Constructor from a Vue reactive-source kind onto one lattice input.
//!
//! P4-3d's only edit in this crate. The class is still [`super::evaluate_binding`]'s
//! join: this file chooses the origin, effects, escape and verdict a kind
//! implies, and does not classify on its own.

use vize_s0::Span;

use super::{BindingId, BindingInput, EffectKind, EffectSet, Verdict};

/// A Vue reactive-source kind, in the same order as
/// `vize_croquis::reactivity::ReactiveKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SourceKind {
    /// `ref()`
    Ref = 0,
    /// `shallowRef()`
    ShallowRef = 1,
    /// `reactive()`
    Reactive = 2,
    /// `shallowReactive()`
    ShallowReactive = 3,
    /// `computed()`
    Computed = 4,
    /// `readonly()`
    Readonly = 5,
    /// `shallowReadonly()`
    ShallowReadonly = 6,
    /// `toRef()`
    ToRef = 7,
    /// `toRefs()`
    ToRefs = 8,
}

impl SourceKind {
    /// The lattice input for one source of this kind.
    ///
    /// Local bindings. `ref` / `shallowRef` / `computed` / `toRef` / `toRefs`
    /// read reactive state. `reactive` / `shallowReactive` also mutate that
    /// local state. `readonly` / `shallowReadonly` track reads and freeze
    /// writes, but the wrapped value is not visible here, so the verdict
    /// stays unknown.
    #[must_use]
    pub const fn binding_input(self, id: BindingId, span: Span) -> BindingInput {
        let (effects, verdict) = match self {
            Self::Ref | Self::ShallowRef | Self::Computed | Self::ToRef | Self::ToRefs => {
                (EffectSet::one(EffectKind::ReadReactive), Verdict::Proven)
            }
            Self::Reactive | Self::ShallowReactive => (
                EffectSet::one(EffectKind::ReadReactive).with(EffectKind::MutateLocal),
                Verdict::Proven,
            ),
            Self::Readonly | Self::ShallowReadonly => (
                EffectSet::one(EffectKind::ReadReactive).with(EffectKind::Freeze),
                Verdict::Unknown,
            ),
        };
        BindingInput::local(id, span)
            .with_effects(effects)
            .with_verdict(verdict)
    }
}
