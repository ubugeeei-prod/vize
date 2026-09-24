//! TS-34 spec for the reactivity lattice's join.
//!
//! The floors, the rank join and the provide/inject cap are data here.
//! Production is [`evaluate_binding`](vize_impeto::lattice::evaluate_binding).
//! A kind's effects and verdict are a second table; the constructor in
//! `vize_impeto` is not called to build it.

use super::agreement::Agreement;
use crate::Croquis;
use crate::facts::{reactivity_losses, reactivity_sources};
use crate::reactivity::ReactiveKind;
use vize_carton::{CompactString, Span, cstr};
use vize_impeto::lattice::{
    BindingFact, BindingId, BindingInput, BindingOrigin, EffectKind, EffectSet, EscapeKind,
    ReactivityClass, SourceKind, Verdict, evaluate_binding,
};

/// Every origin, effect subset, escape and verdict against the lattice evaluator.
#[must_use]
pub fn join_matrix() -> Agreement {
    let origins = [
        BindingOrigin::Local,
        BindingOrigin::Prop,
        BindingOrigin::ProvideInject,
        BindingOrigin::TemplateRef,
    ];
    let escapes = [
        EscapeKind::None,
        EscapeKind::Returned,
        EscapeKind::Stored,
        EscapeKind::Global,
    ];
    let verdicts = [Verdict::Proven, Verdict::Refuted, Verdict::Unknown];
    let mut facts = 0usize;
    let mut bad = None;
    for origin in origins {
        for bits in 0..256u16 {
            for escape in escapes {
                for verdict in verdicts {
                    let input = BindingInput::local(BindingId::new(0), Span::new(0, 0))
                        .with_origin(origin)
                        .with_effects(effect_set(bits))
                        .with_escape(escape)
                        .with_verdict(verdict);
                    let production = evaluate_binding(input);
                    let spec = spec_fact(input);
                    facts += 1;
                    if production != spec && bad.is_none() {
                        bad = Some(cstr!(
                            "lattice input {input:?} production {production:?} != spec {spec:?}"
                        ));
                    }
                }
            }
        }
    }
    let mut run = Agreement::default();
    run.compare(facts, bad);
    run
}

/// The kind constructor and this table name the same effects and verdict.
pub fn kind_table_agrees() -> Result<(), CompactString> {
    for tag in 0..9u8 {
        let kind = match tag {
            0 => ReactiveKind::Ref,
            1 => ReactiveKind::ShallowRef,
            2 => ReactiveKind::Reactive,
            3 => ReactiveKind::ShallowReactive,
            4 => ReactiveKind::Computed,
            5 => ReactiveKind::Readonly,
            6 => ReactiveKind::ShallowReadonly,
            7 => ReactiveKind::ToRef,
            _ => ReactiveKind::ToRefs,
        };
        let Some((effects, verdict)) = kind_input(kind) else {
            return Err(cstr!("kind {kind:?} has no spec input"));
        };
        let input = SourceKind::binding_input(
            source_kind(kind),
            BindingId::new(tag.into()),
            Span::new(0, 0),
        );
        if input.effects != effects
            || input.verdict != verdict
            || input.origin != BindingOrigin::Local
            || input.escape != EscapeKind::None
        {
            return Err(cstr!(
                "kind {kind:?} constructor effects {:?} verdict {:?} != spec effects {effects:?} verdict {verdict:?}",
                input.effects,
                input.verdict
            ));
        }
        let fact = evaluate_binding(input);
        let class = classify(BindingOrigin::Local, effects, EscapeKind::None);
        if fact.class != class || fact.verdict != verdict {
            return Err(cstr!(
                "kind {kind:?} lattice {:?} != spec class {class:?}",
                fact.class
            ));
        }
    }
    Ok(())
}

/// Compare one artifact's lattice rows with the kind table and the join.
pub fn compare_croquis(name: &str, croquis: &Croquis, agreement: &mut Agreement) {
    let sources = reactivity_sources(croquis);
    if sources.is_empty() {
        agreement.skip("no-reactive-source");
        return;
    }
    let mut bad = None;
    for source in &sources {
        let Some((effects, verdict)) = kind_input(source.kind) else {
            bad = Some(cstr!("{name}: unmapped kind {:?}", source.kind));
            break;
        };
        let class = classify(BindingOrigin::Local, effects, EscapeKind::None);
        if source.class != class || source.verdict != verdict || source.effects != effects {
            bad = Some(cstr!(
                "{name}: {} kind {:?} class {:?} verdict {:?} effects {:?} != spec class {class:?} verdict {verdict:?} effects {effects:?}",
                source.name,
                source.kind,
                source.class,
                source.verdict,
                source.effects
            ));
            break;
        }
    }
    if bad.is_none() && reactivity_losses(croquis) != croquis.reactivity.losses() {
        bad = Some(cstr!("{name}: losses diverged from the tracker"));
    }
    if bad.is_none() && sources.len() != croquis.reactivity.sources().len() {
        bad = Some(cstr!("{name}: source count diverged from the tracker"));
    }
    agreement.compare(sources.len(), bad);
}

fn source_kind(kind: ReactiveKind) -> SourceKind {
    match kind {
        ReactiveKind::Ref => SourceKind::Ref,
        ReactiveKind::ShallowRef => SourceKind::ShallowRef,
        ReactiveKind::Reactive => SourceKind::Reactive,
        ReactiveKind::ShallowReactive => SourceKind::ShallowReactive,
        ReactiveKind::Computed => SourceKind::Computed,
        ReactiveKind::Readonly => SourceKind::Readonly,
        ReactiveKind::ShallowReadonly => SourceKind::ShallowReadonly,
        ReactiveKind::ToRef => SourceKind::ToRef,
        ReactiveKind::ToRefs => SourceKind::ToRefs,
    }
}

/// Effects and verdict a kind implies. Independent of [`SourceKind::binding_input`].
fn kind_input(kind: ReactiveKind) -> Option<(EffectSet, Verdict)> {
    let read = EffectSet::one(EffectKind::ReadReactive);
    Some(match kind {
        ReactiveKind::Ref
        | ReactiveKind::ShallowRef
        | ReactiveKind::Computed
        | ReactiveKind::ToRef
        | ReactiveKind::ToRefs => (read, Verdict::Proven),
        ReactiveKind::Reactive | ReactiveKind::ShallowReactive => {
            (read.with(EffectKind::MutateLocal), Verdict::Proven)
        }
        ReactiveKind::Readonly | ReactiveKind::ShallowReadonly => {
            (read.with(EffectKind::Freeze), Verdict::Unknown)
        }
    })
}

fn spec_fact(input: BindingInput) -> BindingFact {
    BindingFact {
        id: input.id,
        class: classify(input.origin, input.effects, input.escape),
        verdict: input.verdict,
        origin: input.origin,
        effects: input.effects,
        escape: input.escape,
        span: input.span,
    }
}

fn classify(origin: BindingOrigin, effects: EffectSet, escape: EscapeKind) -> ReactivityClass {
    let class = join(
        join(origin_floor(origin), effect_floor(effects)),
        escape_floor(escape),
    );
    if origin == BindingOrigin::ProvideInject {
        join(class, ReactivityClass::Reactive)
    } else {
        class
    }
}

fn join(left: ReactivityClass, right: ReactivityClass) -> ReactivityClass {
    if rank(left) >= rank(right) {
        left
    } else {
        right
    }
}

fn rank(class: ReactivityClass) -> u8 {
    match class {
        ReactivityClass::Static => 0,
        ReactivityClass::PropsStable => 1,
        ReactivityClass::Reactive => 2,
        ReactivityClass::Unstable => 3,
    }
}

fn origin_floor(origin: BindingOrigin) -> ReactivityClass {
    match origin {
        BindingOrigin::Local => ReactivityClass::Static,
        BindingOrigin::Prop => ReactivityClass::PropsStable,
        BindingOrigin::ProvideInject | BindingOrigin::TemplateRef => ReactivityClass::Reactive,
    }
}

fn escape_floor(escape: EscapeKind) -> ReactivityClass {
    match escape {
        EscapeKind::None => ReactivityClass::Static,
        EscapeKind::Returned => ReactivityClass::PropsStable,
        EscapeKind::Stored => ReactivityClass::Reactive,
        EscapeKind::Global => ReactivityClass::Unstable,
    }
}

fn effect_floor(effects: EffectSet) -> ReactivityClass {
    if effects.contains(EffectKind::MutateGlobal) || effects.contains(EffectKind::CallUnknown) {
        ReactivityClass::Unstable
    } else if effects.contains(EffectKind::ReadReactive)
        || effects.contains(EffectKind::MutateLocal)
        || effects.contains(EffectKind::Capture)
    {
        ReactivityClass::Reactive
    } else if effects.contains(EffectKind::ReadProp)
        || effects.contains(EffectKind::Freeze)
        || effects.contains(EffectKind::Allocate)
    {
        ReactivityClass::PropsStable
    } else {
        ReactivityClass::Static
    }
}

fn effect_set(bits: u16) -> EffectSet {
    let mut set = EffectSet::empty();
    let kinds = [
        EffectKind::Freeze,
        EffectKind::Capture,
        EffectKind::ReadProp,
        EffectKind::ReadReactive,
        EffectKind::MutateLocal,
        EffectKind::MutateGlobal,
        EffectKind::CallUnknown,
        EffectKind::Allocate,
    ];
    for kind in kinds {
        if bits & (1u16 << (kind as u8)) != 0 {
            set = set.with(kind);
        }
    }
    set
}
