use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l0::{Allocator, Span};
use vize_l3::lattice::dump::Page as ReactivityPage;
use vize_l3::lattice::{
    BindingId, BindingInput, BindingOrigin, EffectKind, EffectSet, EscapeKind, ReactivityClass,
    Verdict, evaluate, evaluate_binding,
};

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

/// The derived order is the chain the Creusot value-axis laws state through
/// `join` (`davinci/vize_l3/src/lattice/creusot.rs`): `a <= b` exactly when
/// joining `a` into `b` leaves `b`. The derive is outside the proof crate,
/// so this oracle covers it for every pair.
#[test]
fn the_derived_order_is_the_join_order() {
    const CLASSES: [ReactivityClass; 4] = [
        ReactivityClass::Static,
        ReactivityClass::PropsStable,
        ReactivityClass::Reactive,
        ReactivityClass::Unstable,
    ];
    for (i, a) in CLASSES.into_iter().enumerate() {
        // The product's `rank` is the discriminant; the proof crate's `rank`
        // is a match returning the chain position. They agree only if each
        // discriminant is its position.
        assert_eq!(usize::from(a as u8), i, "{a:?} discriminant");
        for (j, b) in CLASSES.into_iter().enumerate() {
            assert_eq!(a <= b, a.join(b) == b, "{a:?} <= {b:?}");
            assert_eq!(a.cmp(&b), i.cmp(&j), "{a:?} cmp {b:?}");
        }
    }
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

const ORIGINS: [BindingOrigin; 4] = [
    BindingOrigin::Local,
    BindingOrigin::Prop,
    BindingOrigin::ProvideInject,
    BindingOrigin::TemplateRef,
];
/// The vocabulary in declaration order: index `i` is the kind whose
/// `repr(u8)` discriminant, and so whose set bit, is `i`.
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
const ESCAPES: [EscapeKind; 4] = [
    EscapeKind::None,
    EscapeKind::Returned,
    EscapeKind::Stored,
    EscapeKind::Global,
];

/// The set whose members are the kinds at the bits of `mask`.
fn subset(mask: u16) -> EffectSet {
    EFFECTS
        .into_iter()
        .enumerate()
        .filter(|(bit, _)| mask & (1 << bit) != 0)
        .fold(EffectSet::empty(), |set, (_, effect)| set.with(effect))
}

/// The product's `EffectKind::bit` shifts by the discriminant; the proof
/// crate's spells the bit at each kind's `offset`. They agree only if each
/// discriminant is the kind's position in the vocabulary.
#[test]
fn each_effect_discriminant_is_its_set_offset() {
    for (offset, kind) in EFFECTS.into_iter().enumerate() {
        assert_eq!(usize::from(kind as u8), offset, "{kind:?} discriminant");
        for (other_offset, other) in EFFECTS.into_iter().enumerate() {
            let member = EffectSet::one(kind).contains(other);
            assert_eq!(member, offset == other_offset, "{kind:?} has {other:?}");
        }
    }
}

/// The derives of `EffectSet` are outside the Creusot proof crate. A set is
/// its members: equal exactly when the members are, whatever the insertion
/// order or repetition, equal sets hash alike, and the default is empty.
#[test]
fn effect_set_derives_observe_membership_only() {
    use std::hash::{BuildHasher, RandomState};

    let hasher = RandomState::new();
    assert_eq!(EffectSet::default(), EffectSet::empty());
    for left in 0_u16..256 {
        let reversed = EFFECTS
            .into_iter()
            .enumerate()
            .rev()
            .filter(|(bit, _)| left & (1 << bit) != 0)
            .fold(EffectSet::empty(), |set, (_, effect)| {
                set.with(effect).with(effect)
            });
        assert_eq!(reversed, subset(left));
        assert_eq!(hasher.hash_one(reversed), hasher.hash_one(subset(left)));
        assert_eq!(subset(left).is_empty(), left == 0);
        for right in 0_u16..256 {
            assert_eq!(subset(left) == subset(right), left == right);
        }
    }
}

/// The folio spellings (`as_str`, `from_str`, `EffectSet::print` and
/// `parse`) are outside the Creusot proof crate. Every origin, escape and
/// effect kind spells and parses back, and a page with every origin, all 256
/// effect sets and every escape prints and parses back to itself.
#[test]
fn every_folio_spelling_roundtrips() {
    for origin in ORIGINS {
        assert_eq!(BindingOrigin::from_str(origin.as_str()), Some(origin));
    }
    for escape in ESCAPES {
        assert_eq!(EscapeKind::from_str(escape.as_str()), Some(escape));
    }
    for kind in EFFECTS {
        assert_eq!(EffectKind::from_str(kind.as_str()), Some(kind));
    }
    assert_eq!(EffectKind::from_str("-"), None);

    let arena = Allocator::default();
    let summaries = ORIGINS
        .into_iter()
        .flat_map(|origin| {
            (0_u16..256).flat_map(move |mask| {
                ESCAPES
                    .into_iter()
                    .map(move |escape| (origin, subset(mask), escape))
            })
        })
        .zip(0_u32..)
        .map(|((origin, effects, escape), id)| {
            input(id)
                .with_origin(origin)
                .with_effects(effects)
                .with_escape(escape)
        });
    let page = ReactivityPage::of(&evaluate(&arena, summaries));
    assert_eq!(page.bindings.len(), 4 * 256 * 4);
    let printed = page.print_to_string(DumpMode::Full);
    let parsed = ReactivityPage::parse(printed.as_str()).expect("printed folio parses");
    assert_eq!(parsed, page);
}

/// Every binding summary there is: 4 origins, 256 effect sets, 4 escapes and
/// 3 verdicts. The Creusot proofs in `davinci/vize_l3/src/lattice/creusot.rs`
/// verify the value axis; these are the laws that also read the verdict axis.
#[test]
fn the_verdict_never_changes_the_class_and_only_proven_facts_fire() {
    const VERDICTS: [Verdict; 3] = [Verdict::Proven, Verdict::Refuted, Verdict::Unknown];
    const CLASSES: [ReactivityClass; 4] = [
        ReactivityClass::Static,
        ReactivityClass::PropsStable,
        ReactivityClass::Reactive,
        ReactivityClass::Unstable,
    ];

    let mut checked = 0_u32;
    for origin in ORIGINS {
        for mask in 0_u16..256 {
            let effects = subset(mask);
            for escape in ESCAPES {
                let summary = input(0)
                    .with_origin(origin)
                    .with_effects(effects)
                    .with_escape(escape);
                let proven = evaluate_binding(summary.with_verdict(Verdict::Proven));
                for verdict in VERDICTS {
                    let fact = evaluate_binding(summary.with_verdict(verdict));
                    // Lean `classify_verdict_orthogonal`.
                    assert_eq!(fact.class, proven.class);
                    assert_eq!(fact.verdict, verdict);
                    let is_proven = verdict == Verdict::Proven;
                    assert_eq!(fact.proven_class(), is_proven.then_some(fact.class));
                    for class in CLASSES {
                        // Lean `fires_iff` and `fires_only_proven`.
                        assert_eq!(fact.fires_as(class), is_proven && fact.class == class);
                    }
                    checked += 1;
                }
            }
        }
    }
    assert_eq!(checked, 4 * 256 * 4 * 3);
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
    let folio = ReactivityPage::parse(CANONICAL).expect("canonical lattice folio parses");
    assert_eq!(folio.print_to_string(DumpMode::Full).as_str(), CANONICAL);
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
    let folio = ReactivityPage::of(&facts);
    let printed = folio.print_to_string(DumpMode::Full);
    assert_eq!(printed.as_str(), CANONICAL);
    assert_eq!(ReactivityPage::parse(printed.as_str()).unwrap(), folio);
}
