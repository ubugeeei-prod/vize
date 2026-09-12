use vize_davinci::folio::{Folio, FolioMode};
use vize_impeto::lattice::{
    BindingId, BindingInput, BindingOrigin, EffectKind, EffectSet, EscapeKind, ReactivityClass,
    ReactivityFolio, Verdict, evaluate, evaluate_binding,
};
use vize_s0::{Allocator, Span};

const CANONICAL: &str = "\
[s3-reactivity-folio]

[s3-reactivity-folio.bindings]
id=0 class=static verdict=proven origin=local effects=- escape=none span=0:3
id=1 class=reactive verdict=unknown origin=prop effects=capture,read-reactive escape=none span=4:9
id=2 class=unstable verdict=proven origin=provide-inject effects=mutate-global escape=stored span=10:20

";

fn input(index: u32) -> BindingInput {
    BindingInput::local(BindingId::new(index), Span::new(index * 10, index * 10 + 3))
}

#[test]
fn class_join_moves_only_toward_less_stable_values() {
    assert_eq!(
        ReactivityClass::Static.join(ReactivityClass::PropsStable),
        ReactivityClass::PropsStable
    );
    assert_eq!(
        ReactivityClass::Reactive.join(ReactivityClass::PropsStable),
        ReactivityClass::Reactive
    );
    assert_eq!(
        ReactivityClass::Unstable.join(ReactivityClass::Static),
        ReactivityClass::Unstable
    );
}

#[test]
fn effect_vocabulary_maps_to_the_lattice_floor() {
    assert_eq!(EffectSet::empty().class_floor(), ReactivityClass::Static);
    assert_eq!(
        EffectSet::one(EffectKind::ReadProp).class_floor(),
        ReactivityClass::PropsStable
    );
    assert_eq!(
        EffectSet::one(EffectKind::Capture)
            .with(EffectKind::ReadReactive)
            .class_floor(),
        ReactivityClass::Reactive
    );
    assert_eq!(
        EffectSet::one(EffectKind::MutateGlobal).class_floor(),
        ReactivityClass::Unstable
    );
    assert_eq!(
        EffectSet::one(EffectKind::CallUnknown).class_floor(),
        ReactivityClass::Unstable
    );
}

#[test]
fn origins_and_escape_analysis_demote_classifications() {
    assert_eq!(evaluate_binding(input(0)).class, ReactivityClass::Static);
    assert_eq!(
        evaluate_binding(input(1).with_origin(BindingOrigin::Prop)).class,
        ReactivityClass::PropsStable
    );
    assert_eq!(
        evaluate_binding(input(2).with_escape(EscapeKind::Stored)).class,
        ReactivityClass::Reactive
    );
    assert_eq!(
        evaluate_binding(input(3).with_escape(EscapeKind::Global)).class,
        ReactivityClass::Unstable
    );
}

#[test]
fn provide_inject_is_never_more_stable_than_reactive() {
    let inert = evaluate_binding(input(0).with_origin(BindingOrigin::ProvideInject));
    assert_eq!(inert.class, ReactivityClass::Reactive);

    let mutating = evaluate_binding(
        input(1)
            .with_origin(BindingOrigin::ProvideInject)
            .with_effects(EffectSet::one(EffectKind::MutateGlobal)),
    );
    assert_eq!(mutating.class, ReactivityClass::Unstable);
}

#[test]
fn verdict_axis_is_orthogonal_and_consumers_fire_only_on_proven_values() {
    let unknown = evaluate_binding(
        input(0)
            .with_effects(EffectSet::one(EffectKind::ReadReactive))
            .with_verdict(Verdict::Unknown),
    );
    assert_eq!(unknown.class, ReactivityClass::Reactive);
    assert_eq!(unknown.proven_class(), None);
    assert!(!unknown.fires_as(ReactivityClass::Reactive));

    let proven = evaluate_binding(input(1).with_effects(EffectSet::one(EffectKind::ReadReactive)));
    assert_eq!(proven.proven_class(), Some(ReactivityClass::Reactive));
    assert!(proven.fires_as(ReactivityClass::Reactive));
}

#[test]
fn fact_group_lookup_keeps_binding_ids_exact() {
    let arena = Allocator::default();
    let facts = evaluate(
        &arena,
        [
            input(0),
            input(7).with_effects(EffectSet::one(EffectKind::ReadProp)),
        ],
    );
    assert_eq!(facts.bindings.len(), 2);
    assert_eq!(
        facts.get(BindingId::new(7)).map(|fact| fact.class),
        Some(ReactivityClass::PropsStable)
    );
    assert_eq!(facts.get(BindingId::new(1)), None);
}

#[test]
fn reactivity_folio_roundtrips_canonical_text() {
    let folio = ReactivityFolio::parse(CANONICAL).expect("canonical lattice folio parses");
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
}

#[test]
fn fact_group_print_is_structural_identity() {
    let arena = Allocator::default();
    let facts = evaluate(
        &arena,
        [
            input(0),
            BindingInput::local(BindingId::new(1), Span::new(4, 9))
                .with_origin(BindingOrigin::Prop)
                .with_effects(EffectSet::one(EffectKind::Capture).with(EffectKind::ReadReactive))
                .with_verdict(Verdict::Unknown),
            BindingInput::local(BindingId::new(2), Span::new(10, 20))
                .with_origin(BindingOrigin::ProvideInject)
                .with_effects(EffectSet::one(EffectKind::MutateGlobal))
                .with_escape(EscapeKind::Stored),
        ],
    );
    let folio = ReactivityFolio::of(&facts);
    let printed = folio.print_to_string(FolioMode::Full);
    assert_eq!(printed.as_str(), CANONICAL);
    assert_eq!(ReactivityFolio::parse(printed.as_str()).unwrap(), folio);
}
