//! TS-16 for the disegno folio (P2-5a; expression payloads P2-5b).
//!
//! `Full` mode: `print(parse(t)) == t` byte-exact for canonical text and
//! `parse(print(v)) == v` structurally, with normalization by the first
//! print for non-canonical input. `Display` explicitly carries **no**
//! round-trip law - here it elides every span (line tails and the spans
//! inside expression payloads) and nothing else. The committed reference
//! page (`tests/fixtures/reference.folio`) covers every op kind, both
//! binding kinds, escapes, a namespace, static and dynamic names,
//! optional `ui.for` positions, and all three expression payload kinds;
//! `tests/folio_mirror.rs` builds the same tree in a live arena, and the
//! remaining opaque-reason spellings are pinned by
//! `every_opaque_reason_spelling_round_trips` below.

use vize_carton::{Span, String};
use vize_davinci::folio::{Folio, FolioMode};
use vize_disegno::expr::OpaqueReason;
use vize_disegno::folio::{
    DisegnoFolio, FolioAttribute, FolioBind, FolioBinding, FolioBranch, FolioComponent,
    FolioContract, FolioElement, FolioExpr, FolioFor, FolioForBinding, FolioIf, FolioInterpolation,
    FolioModel, FolioName, FolioOn, FolioOp, FolioSlot, FolioText, FolioVueDirective,
};
use vize_disegno::op::Namespace;

/// Canonical text of the reference tree.
const CANONICAL: &str = include_str!("fixtures/reference.folio");

/// `Display` output for the same tree: every span elided, nothing else.
const DISPLAY: &str = "\
[disegno]
ops=12

[disegno.ops]
ui.element form
  attr method=\"post\"
  ui.model read=js(\"draft.note\") write=js(\"draft.note\")
    attr element-kind=\"textarea\"
  vue.directive \"pin\" arg=\"top\" mods=\"lazy,trim\" value=opaque(multi-statement \"top++; sync()\")
  ui.if
    branch js(\"open\")
      ui.text \"a\\\"b\\\\c\"
    branch
      ui.interpolation opaque(nesting-refused \"((((x))))\")
ui.for source=opaque(for-value \"a in b in c\") value=js(\"a\") key=js(\"i\")
  ui.slot name=js(\"kind\")
    attr tone=\"brisk\"
    ui.bind name=\"chip\" mods=\"camel\" value=js(\"chipVal\")
    ui.on name=\"press\" mods=\"stop\"
    ui.interpolation foreign(moonbit \"count + 1\")
ui.component Chrome

";

fn js(source: &str, start: u32, end: u32) -> FolioExpr {
    FolioExpr::Js {
        source: String::from(source),
        span: Span::new(start, end),
    }
}

fn opaque(reason: OpaqueReason, source: &str, start: u32, end: u32) -> FolioExpr {
    FolioExpr::Opaque {
        reason,
        source: String::from(source),
        span: Span::new(start, end),
    }
}

/// The reference tree, hand-built in the owned model.
fn hand_built() -> DisegnoFolio {
    DisegnoFolio {
        ops: vec![
            FolioOp::Element(FolioElement {
                tag: String::from("form"),
                namespace: Namespace::Html,
                attributes: vec![FolioAttribute {
                    name: String::from("method"),
                    value: Some(String::from("post")),
                    span: Span::new(5, 20),
                }],
                bindings: vec![
                    FolioBinding::Model(FolioModel {
                        contract: FolioContract {
                            read: js("draft.note", 29, 39),
                            write: js("draft.note", 29, 39),
                        },
                        attributes: vec![FolioAttribute {
                            name: String::from("element-kind"),
                            value: Some(String::from("textarea")),
                            span: Span::new(21, 40),
                        }],
                        span: Span::new(21, 40),
                    }),
                    FolioBinding::VueDirective(FolioVueDirective {
                        name: String::from("pin"),
                        argument: Some(FolioName::Static(String::from("top"))),
                        modifiers: vec![String::from("lazy"), String::from("trim")],
                        value: Some(opaque(
                            OpaqueReason::MultiStatement,
                            "top++; sync()",
                            46,
                            59,
                        )),
                        span: Span::new(41, 60),
                    }),
                ],
                children: vec![FolioOp::If(FolioIf {
                    branches: vec![
                        FolioBranch {
                            condition: Some(js("open", 65, 69)),
                            ops: vec![FolioOp::Text(FolioText {
                                content: String::from("a\"b\\c"),
                                span: Span::new(66, 70),
                            })],
                            span: Span::new(61, 75),
                        },
                        FolioBranch {
                            condition: None,
                            ops: vec![FolioOp::Interpolation(FolioInterpolation {
                                expression: opaque(
                                    OpaqueReason::NestingRefused,
                                    "((((x))))",
                                    82,
                                    88,
                                ),
                                span: Span::new(80, 88),
                            })],
                            span: Span::new(75, 90),
                        },
                    ],
                    span: Span::new(61, 90),
                })],
                span: Span::new(0, 99),
            }),
            FolioOp::For(FolioFor {
                binding: FolioForBinding {
                    source: opaque(OpaqueReason::ForValue, "a in b in c", 110, 121),
                    value: js("a", 105, 106),
                    key: Some(js("i", 108, 109)),
                    index: None,
                },
                ops: vec![FolioOp::Slot(FolioSlot {
                    name: FolioName::Dynamic(js("kind", 136, 140)),
                    attributes: vec![FolioAttribute {
                        name: String::from("tone"),
                        value: Some(String::from("brisk")),
                        span: Span::new(141, 147),
                    }],
                    bindings: vec![
                        FolioBinding::Bind(FolioBind {
                            name: Some(FolioName::Static(String::from("chip"))),
                            modifiers: vec![String::from("camel")],
                            value: Some(js("chipVal", 149, 156)),
                            span: Span::new(148, 157),
                        }),
                        FolioBinding::On(FolioOn {
                            name: Some(FolioName::Static(String::from("press"))),
                            modifiers: vec![String::from("stop")],
                            handler: None,
                            span: Span::new(158, 160),
                        }),
                    ],
                    fallback: vec![FolioOp::Interpolation(FolioInterpolation {
                        expression: FolioExpr::Foreign {
                            dialect: String::from("moonbit"),
                            source: String::from("count + 1"),
                            span: Span::new(150, 159),
                        },
                        span: Span::new(148, 161),
                    })],
                    span: Span::new(131, 161),
                })],
                span: Span::new(100, 161),
            }),
            FolioOp::Component(FolioComponent {
                name: String::from("Chrome"),
                attributes: vec![],
                bindings: vec![],
                children: vec![],
                span: Span::new(162, 171),
            }),
        ],
    }
}

#[test]
fn full_print_is_identity_on_canonical_text() {
    let value = DisegnoFolio::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(value.print_to_string(FolioMode::Full).as_str(), CANONICAL);
}

#[test]
fn parse_print_is_structural_identity() {
    let value = hand_built();
    let printed = value.print_to_string(FolioMode::Full);
    let reparsed = DisegnoFolio::parse(printed.as_str()).expect("printed text parses");
    assert_eq!(reparsed, value);
}

#[test]
fn unsafe_modifier_names_round_trip_through_canonical_brackets() {
    let value = DisegnoFolio {
        ops: vec![FolioOp::Element(FolioElement {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![],
            bindings: vec![FolioBinding::VueDirective(FolioVueDirective {
                name: String::from("if*\"props"),
                argument: None,
                modifiers: vec![
                    String::from("showLineNumbers\""),
                    String::from("with,comma"),
                ],
                value: None,
                span: Span::new(97, 125),
            })],
            children: vec![],
            span: Span::new(92, 290),
        })],
    };
    let canonical = "\
[disegno]
ops=2

[disegno.ops]
ui.element div @92:290
  vue.directive \"if*\\\"props\" mods=[\"showLineNumbers\\\"\",\"with,comma\"] @97:125

";
    let printed = value.print_to_string(FolioMode::Full);
    assert_eq!(printed.as_str(), canonical);
    assert_eq!(
        DisegnoFolio::parse(printed.as_str()).expect("printed text parses"),
        value
    );
}

#[test]
fn a_hand_built_value_prints_the_canonical_text() {
    assert_eq!(
        hand_built().print_to_string(FolioMode::Full).as_str(),
        CANONICAL
    );
    assert_eq!(
        DisegnoFolio::parse(CANONICAL).expect("canonical text parses"),
        hand_built()
    );
}

#[test]
fn display_elides_spans_and_carries_no_round_trip_law() {
    assert_eq!(
        hand_built().print_to_string(FolioMode::Display).as_str(),
        DISPLAY
    );
}

#[test]
fn an_empty_tree_prints_the_header_only() {
    let empty = DisegnoFolio::default();
    assert_eq!(
        empty.print_to_string(FolioMode::Full).as_str(),
        "[disegno]\nops=0\n\n"
    );
    assert_eq!(
        DisegnoFolio::parse("[disegno]\nops=0\n\n").expect("empty page parses"),
        empty
    );
}

#[test]
fn non_canonical_input_is_normalized_by_the_first_print() {
    // A stale count, blank-line separators, and a zero-padded span offset:
    // parse accepts all of it and the first print is canonical.
    let scrambled = "\
[disegno]
ops=999

[disegno.ops]

ui.element form @0:007


  attr method=\"post\" @5:20
ui.component Chrome @121:130
";
    let value = DisegnoFolio::parse(scrambled).expect("scrambled text parses");
    assert_eq!(
        value.print_to_string(FolioMode::Full).as_str(),
        "\
[disegno]
ops=2

[disegno.ops]
ui.element form @0:7
  attr method=\"post\" @5:20
ui.component Chrome @121:130

"
    );
}

/// Every [`OpaqueReason`] spelling is pinned, both directions, on one
/// exact page - so a renamed or added reason breaks a committed oracle,
/// not just the enum.
#[test]
fn every_opaque_reason_spelling_round_trips() {
    let reasons = [
        (OpaqueReason::ForValue, "for-value"),
        (OpaqueReason::MultiStatement, "multi-statement"),
        (OpaqueReason::NestingRefused, "nesting-refused"),
        (OpaqueReason::ParseRejected, "parse-rejected"),
        (OpaqueReason::Compound, "compound"),
    ];
    let value = DisegnoFolio {
        ops: reasons
            .iter()
            .map(|(reason, _)| {
                FolioOp::Interpolation(FolioInterpolation {
                    expression: opaque(*reason, "x", 1, 2),
                    span: Span::new(0, 3),
                })
            })
            .collect(),
    };
    let canonical = "\
[disegno]
ops=5

[disegno.ops]
ui.interpolation opaque(for-value \"x\" @1:2) @0:3
ui.interpolation opaque(multi-statement \"x\" @1:2) @0:3
ui.interpolation opaque(nesting-refused \"x\" @1:2) @0:3
ui.interpolation opaque(parse-rejected \"x\" @1:2) @0:3
ui.interpolation opaque(compound \"x\" @1:2) @0:3

";
    assert_eq!(value.print_to_string(FolioMode::Full).as_str(), canonical);
    assert_eq!(
        DisegnoFolio::parse(canonical).expect("canonical text parses"),
        value
    );
    for (reason, mnemonic) in reasons {
        assert_eq!(reason.mnemonic(), mnemonic);
        assert_eq!(OpaqueReason::from_mnemonic(mnemonic), Some(reason));
    }
    assert_eq!(OpaqueReason::from_mnemonic("js"), None);
}
