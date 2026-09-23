//! P3-15 bridge from the Rust naive lattice evaluator into the Lean reference.
//!
//! The committed fact page is exactly what `evaluate` prints for an enumerated
//! input matrix. `lake exe impetoRef --check-lattice-fixtures` re-enumerates the
//! same matrix, parses this page without Rust code and recomputes every class
//! with the independent Lean classifier whose laws are proved in
//! `tests/formal/impeto/Impeto/LatticeLaws.lean`. Regenerate the page with
//! `VIZE_UPDATE_LATTICE_REFERENCE_FIXTURE=1` only when the rule spec changes.

use std::path::Path;

use vize_davinci::folio::{Folio, FolioMode};
use vize_impeto::lattice::{
    BindingId, BindingInput, BindingOrigin, EffectKind, EffectSet, EscapeKind, ReactivityClass,
    ReactivityFolio, Verdict, evaluate,
};
use vize_s0::{Allocator, Span};

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
const ORIGINS: [BindingOrigin; 4] = [
    BindingOrigin::Local,
    BindingOrigin::Prop,
    BindingOrigin::ProvideInject,
    BindingOrigin::TemplateRef,
];
const ESCAPES: [EscapeKind; 4] = [
    EscapeKind::None,
    EscapeKind::Returned,
    EscapeKind::Stored,
    EscapeKind::Global,
];
const VERDICTS: [Verdict; 3] = [Verdict::Proven, Verdict::Refuted, Verdict::Unknown];

fn subset(mask: usize) -> EffectSet {
    EFFECTS
        .iter()
        .enumerate()
        .filter(|(bit, _)| mask & (1 << bit) != 0)
        .fold(EffectSet::empty(), |set, (_, kind)| set.with(*kind))
}

/// All 256 effect sets for an inert local binding, then every origin/escape
/// pair with no effect and with each single effect.
fn matrix() -> impl Iterator<Item = (BindingOrigin, EffectSet, EscapeKind)> {
    let powerset = (0..256).map(|mask| (BindingOrigin::Local, subset(mask), EscapeKind::None));
    let singles = core::iter::once(EffectSet::empty()).chain(EFFECTS.map(EffectSet::one));
    let crossed = ORIGINS.into_iter().flat_map(move |origin| {
        let singles = singles.clone();
        ESCAPES.into_iter().flat_map(move |escape| {
            singles
                .clone()
                .map(move |effects| (origin, effects, escape))
        })
    });
    powerset.chain(crossed)
}

fn inputs() -> impl Iterator<Item = BindingInput> {
    matrix()
        .enumerate()
        .map(|(index, (origin, effects, escape))| {
            BindingInput::local(
                BindingId::new(u32::try_from(index).unwrap()),
                Span::new(0, 0),
            )
            .with_origin(origin)
            .with_effects(effects)
            .with_escape(escape)
            .with_verdict(VERDICTS[index % VERDICTS.len()])
        })
}

#[test]
fn rust_lattice_fact_page_matches_lean_reference_fixture() {
    let allocator = Allocator::default();
    let facts = evaluate(&allocator, inputs());
    assert_eq!(facts.bindings.len(), 256 + 4 * 4 * 9);
    let printed = ReactivityFolio::of(&facts).print_to_string(FolioMode::Full);
    let mut expected = printed.trim_end().as_bytes().to_vec();
    expected.push(b'\n');

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/formal/impeto/fixtures/reactivity-lattice.folio");
    if std::env::var("VIZE_UPDATE_LATTICE_REFERENCE_FIXTURE").as_deref() == Ok("1") {
        std::fs::write(&path, &expected).unwrap();
    }
    let committed = std::fs::read(&path).unwrap();
    assert!(
        committed == expected,
        "{} drifted from the Rust lattice evaluator",
        path.display()
    );
}

#[test]
fn fixture_matrix_publishes_every_class_and_verdict() {
    let allocator = Allocator::default();
    let facts = evaluate(&allocator, inputs());
    for class in [
        ReactivityClass::Static,
        ReactivityClass::PropsStable,
        ReactivityClass::Reactive,
        ReactivityClass::Unstable,
    ] {
        assert!(facts.bindings.iter().any(|fact| fact.class == class));
    }
    for verdict in VERDICTS {
        assert!(facts.bindings.iter().any(|fact| fact.verdict == verdict));
    }
}
