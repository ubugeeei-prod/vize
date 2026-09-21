//! P4-6a's witness law, pinned by construction: **an error diagnostic always
//! carries a witness — a non-empty fact chain or a declared exemption — and
//! nothing else can build one.**
//!
//! The negative half (an unwitnessed error does not compile; a heuristic
//! error rule does not compile) is the `compile_fail` doctests on
//! `vize_davinci::diagnostic` and `diagnostic::tier`. This suite pins the
//! positive half as exact values: what each constructor stores, what the
//! accessors read back, and the witness types' own laws.

use std::panic::catch_unwind;

use vize_davinci::diagnostic::{
    Advisory, Diagnostic, DiagnosticPart, Exemption, PartKind, Severity, Stage, Witness,
    WitnessChain, WitnessKey, WitnessKeyed, WitnessLink,
};
use vize_davinci::id::NodeId;
use vize_davinci::pass::AnalysisId;
use vize_s0::{Span, String};

static FIXTURE_EXEMPTION: Exemption = Exemption::new("witness_law_fixture", "fixture/kind");

const BINDINGS: AnalysisId = AnalysisId::new(3);
const ESCAPES: AnalysisId = AnalysisId::new(7);

fn node(index: u32) -> NodeId {
    NodeId::from_index(index).expect("a non-zero index is a node id")
}

fn binding_link() -> WitnessLink {
    WitnessLink::new(
        BINDINGS,
        Span::new(4, 9),
        WitnessKey::Name(String::from("count")),
    )
}

fn escape_link() -> WitnessLink {
    WitnessLink::new(ESCAPES, Span::new(20, 31), WitnessKey::Node(node(2)))
}

#[test]
fn an_advisory_diagnostic_carries_its_severity_and_no_witness() {
    for advisory in Advisory::ALL {
        let diagnostic = Diagnostic::new(advisory, Stage::Semantic, Span::new(4, 9), "no key");
        assert_eq!(diagnostic.severity(), advisory.severity());
        assert_eq!(diagnostic.stage, Stage::Semantic);
        assert_eq!(diagnostic.span, Span::new(4, 9));
        assert_eq!(diagnostic.message.as_str(), "no key");
        assert_eq!(diagnostic.parts, []);
        assert_eq!(diagnostic.witness(), None);
        assert_eq!(diagnostic.witness_chain(), None);
        assert_eq!(diagnostic.exemption(), None);
    }
}

#[test]
fn a_proven_error_stores_its_chain_and_no_exemption() {
    let chain = WitnessChain::new(binding_link()).then(escape_link());
    let error = Diagnostic::proven(Stage::Lowered, Span::new(4, 9), "escapes", chain.clone());
    assert_eq!(error.severity(), Severity::Error);
    assert_eq!(error.stage, Stage::Lowered);
    assert_eq!(error.span, Span::new(4, 9));
    assert_eq!(error.message.as_str(), "escapes");
    assert_eq!(error.witness(), Some(&Witness::Proven(chain.clone())));
    assert_eq!(error.witness_chain(), Some(&chain));
    assert_eq!(error.exemption(), None);
}

#[test]
fn a_legacy_error_reports_under_its_declared_exemption() {
    let error = Diagnostic::legacy_error(
        &FIXTURE_EXEMPTION,
        Stage::Surface,
        Span::new(0, 1),
        "unterminated",
    );
    assert_eq!(error.severity(), Severity::Error);
    assert_eq!(
        error.witness(),
        Some(&Witness::LegacyExempt(&FIXTURE_EXEMPTION))
    );
    assert_eq!(error.witness_chain(), None);
    assert_eq!(error.exemption(), Some(&FIXTURE_EXEMPTION));
    let exemption = error.exemption().expect("exempt");
    assert_eq!(
        (exemption.producer(), exemption.code()),
        ("witness_law_fixture", "fixture/kind")
    );
}

#[test]
fn a_proof_supersedes_an_exemption_and_never_changes_the_severity() {
    let chain = WitnessChain::new(binding_link());
    let proven = Diagnostic::legacy_error(&FIXTURE_EXEMPTION, Stage::Surface, Span::new(0, 1), "x")
        .with_witness(chain.clone());
    assert_eq!(proven.severity(), Severity::Error);
    assert_eq!(proven.witness(), Some(&Witness::Proven(chain.clone())));
    assert_eq!(proven.exemption(), None);
    assert_eq!(
        proven,
        Diagnostic::proven(Stage::Surface, Span::new(0, 1), "x", chain.clone())
    );

    let why = Diagnostic::new(Advisory::Hint, Stage::Semantic, Span::new(0, 1), "why")
        .with_witness(chain.clone());
    assert_eq!(why.severity(), Severity::Hint);
    assert_eq!(why.witness_chain(), Some(&chain));
}

#[test]
fn parts_accumulate_in_the_order_they_are_attached() {
    let diagnostic = Diagnostic::legacy_error(
        &FIXTURE_EXEMPTION,
        Stage::Surface,
        Span::new(0, 1),
        "unterminated element",
    )
    .with_part(DiagnosticPart::new(
        PartKind::Primary,
        Span::new(0, 1),
        "opened here",
    ))
    .with_part(DiagnosticPart::new(
        PartKind::Help,
        Span::new(9, 9),
        "add a closing tag",
    ));
    assert_eq!(
        diagnostic.parts,
        [
            DiagnosticPart::new(PartKind::Primary, Span::new(0, 1), "opened here"),
            DiagnosticPart::new(PartKind::Help, Span::new(9, 9), "add a closing tag"),
        ]
    );
}

#[test]
fn a_chain_is_never_empty_and_keeps_proof_order() {
    assert_eq!(WitnessChain::from_links(Vec::new()), None);
    let built = WitnessChain::new(binding_link()).then(escape_link());
    assert_eq!(built.links(), [binding_link(), escape_link()]);
    assert_eq!(built.first(), &binding_link());
    assert_eq!(
        WitnessChain::from_links(vec![binding_link(), escape_link()]),
        Some(built)
    );
}

fn assert_round_trip<K: WitnessKeyed + PartialEq + std::fmt::Debug>(key: K, erased: WitnessKey) {
    assert_eq!(key.to_witness_key(), erased);
    assert_eq!(K::from_witness_key(&erased), Some(key));
}

#[test]
fn every_key_shape_round_trips_and_refuses_the_other_shapes() {
    let shapes = [
        WitnessKey::Artifact,
        WitnessKey::Node(node(5)),
        WitnessKey::Index(11),
        WitnessKey::Name(String::from("count")),
    ];
    assert_round_trip((), shapes[0].clone());
    assert_round_trip(node(5), shapes[1].clone());
    assert_round_trip(11u32, shapes[2].clone());
    assert_round_trip(String::from("count"), shapes[3].clone());
    let accepted = |shape: &WitnessKey| {
        [
            <()>::from_witness_key(shape).is_some(),
            NodeId::from_witness_key(shape).is_some(),
            u32::from_witness_key(shape).is_some(),
            String::from_witness_key(shape).is_some(),
        ]
    };
    let table: Vec<[bool; 4]> = shapes.iter().map(accepted).collect();
    assert_eq!(
        table,
        [
            [true, false, false, false],
            [false, true, false, false],
            [false, false, true, false],
            [false, false, false, true],
        ]
    );
}

/// The const assertions panic with literal messages, whose payload is a
/// `&'static str`.
fn panic_message(run: impl FnOnce() + std::panic::UnwindSafe) -> &'static str {
    let payload = catch_unwind(run).expect_err("must panic");
    *payload
        .downcast::<&'static str>()
        .expect("a literal panic message")
}

#[test]
fn an_exemption_name_outside_the_inventory_alphabet_is_refused() {
    let producer = "an exemption's producer must be a non-empty [a-z0-9_./-] name";
    let code = "an exemption's code must be a non-empty [a-z0-9_./-] name";
    let refused = [
        ("", "kind", producer),
        ("Vize", "kind", producer),
        ("vize\ts1", "kind", producer),
        ("vize_s1_to_s2", "", code),
        ("vize_s1_to_s2", "v-if \"key\"", code),
    ];
    for (name, kind, expected) in refused {
        let message = panic_message(move || {
            let _ = Exemption::new(name, kind);
        });
        assert_eq!(message, expected, "{name:?} / {kind:?}");
    }
    let accepted = Exemption::new("vize_patina", "a11y/alt-text.v2");
    assert_eq!(
        (accepted.producer(), accepted.code()),
        ("vize_patina", "a11y/alt-text.v2")
    );
}
