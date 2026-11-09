//! Independent current and historical wire oracles over the same owned IR.

use vize_l0::dump::{Dump, Error, Mode};
use vize_l0::key::{ArtifactKey, KeyedArtifact};
use vize_l0::{Span, String, cstr};
use vize_l2::dump::historical::v1::Page as HistoricalPage;
use vize_l2::dump::provenance::{Page as ProvenancePage, Record};
use vize_l2::dump::{Attribute, Comment, Element, Expr, Interpolation, Op, Page, Text};
use vize_l2::expr::OpaqueReason;
use vize_l2::op::Namespace;

/// The historical v1 page header, kept as a wire identity.
const V1: &str = "disegno"; // codename: the v1 page header

/// Spell the v1 header into a fixture written with `@V1@`.
fn v1(text: &str) -> String {
    String::from(text.replace("@V1@", V1))
}

const CURRENT: &str = r#"[l2-dump-v2]
ops=6

[l2-dump-v2.ops]
ui.element div @0:120
  attr title="[@V1@]" @1:12
  ui.text "[@V1@.ops]" @12:27
  ui.interpolation js("[l2-dump-v2]" @27:42) @27:42
  ui.comment "[l2-dump-v2.ops]" @42:60
  ui.interpolation foreign(md "[@V1@]" @60:78) @60:78
  ui.interpolation opaque(multi-statement "[s2-provenance-dump]" @78:100) @78:100

"#;

const HISTORICAL: &str = r#"[@V1@]
ops=6

[@V1@.ops]
ui.element div @0:120
  attr title="[@V1@]" @1:12
  ui.text "[@V1@.ops]" @12:27
  ui.interpolation js("[l2-dump-v2]" @27:42) @27:42
  ui.comment "[l2-dump-v2.ops]" @42:60
  ui.interpolation foreign(md "[@V1@]" @60:78) @60:78
  ui.interpolation opaque(multi-statement "[s2-provenance-dump]" @78:100) @78:100

"#;

fn tree(base: u32) -> Page {
    let span = |start, end| Span::new(base + start, base + end);
    Page {
        ops: vec![Op::Element(Element {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![Attribute {
                name: String::from("title"),
                value: Some(v1("[@V1@]")),
                span: span(1, 12),
            }],
            bindings: vec![],
            children: vec![
                Op::Text(Text {
                    content: v1("[@V1@.ops]"),
                    span: span(12, 27),
                }),
                Op::Interpolation(Interpolation {
                    expression: Expr::Js {
                        source: String::from("[l2-dump-v2]"),
                        span: span(27, 42),
                    },
                    raw_html: false,
                    span: span(27, 42),
                }),
                Op::Comment(Comment {
                    content: String::from("[l2-dump-v2.ops]"),
                    span: span(42, 60),
                }),
                Op::Interpolation(Interpolation {
                    expression: Expr::Foreign {
                        dialect: String::from("md"),
                        source: v1("[@V1@]"),
                        span: span(60, 78),
                    },
                    raw_html: false,
                    span: span(60, 78),
                }),
                Op::Interpolation(Interpolation {
                    expression: Expr::Opaque {
                        reason: OpaqueReason::MultiStatement,
                        source: String::from("[s2-provenance-dump]"),
                        span: span(78, 100),
                    },
                    raw_html: false,
                    span: span(78, 100),
                }),
            ],
            span: span(0, 120),
        })],
    }
}

#[test]
fn both_explicit_codecs_preserve_header_looking_payloads_and_spans() {
    let page = tree(0);
    assert_eq!(page.print_to_string(Mode::Full), v1(CURRENT));
    assert_eq!(Page::parse(&v1(CURRENT)), Ok(page.clone()));
    let old = HistoricalPage::from_current(page.clone());
    assert_eq!(old.print_to_string(Mode::Full), v1(HISTORICAL));
    assert_eq!(HistoricalPage::parse(&v1(HISTORICAL)), Ok(old.clone()));
    assert_eq!(old.into_current(), page);
}

#[test]
fn current_and_historical_headers_never_select_each_other() {
    assert_eq!(
        Page::parse(&v1(HISTORICAL)),
        Err(Error::new(1, cstr!("first section must be [l2-dump-v2]")))
    );
    assert_eq!(
        HistoricalPage::parse(&v1(CURRENT)),
        Err(Error::new(1, cstr!("first section must be [{V1}]")))
    );
    for (input, line, message) in [
        (
            "[l2-dump-v2]\nops=0\n[@V1@.ops]\n",
            3,
            "unknown section [@V1@.ops]",
        ),
        (
            "[l2-dump-v2]\nops=0\n[l2-dump-v2]\n",
            3,
            "duplicate section [l2-dump-v2]",
        ),
        (
            "[l2-dump-v2]\nops=0\n[l2-dump-v2.ops]\n[l2-dump-v2.ops]\n",
            4,
            "duplicate section [l2-dump-v2.ops]",
        ),
        ("", 0, "missing [l2-dump-v2] header"),
    ] {
        assert_eq!(Page::parse(&v1(input)), Err(Error::new(line, v1(message))));
    }
    assert_eq!(
        HistoricalPage::parse(&v1("[@V1@]\nops=0\n[l2-dump-v2.ops]\n")),
        Err(Error::new(3, cstr!("unknown section [l2-dump-v2.ops]")))
    );
}

#[test]
fn provenance_v2_preserves_quoted_headers_and_record_spans() {
    let page = ProvenancePage {
        records: vec![Record {
            rule: String::from("lower.text"),
            node: Some(7),
            before: v1("[s2-provenance-dump]\n[@V1@]"),
            after: String::from("[l2-provenance-dump-v2]"),
            span: Span::new(3, 42),
        }],
    };
    let expected = v1(r#"[l2-provenance-dump-v2]

[l2-provenance-dump-v2.records]
rule=lower.text node=7 before="[s2-provenance-dump]\n[@V1@]" after="[l2-provenance-dump-v2]" @3:42

"#);
    assert_eq!(page.print_to_string(Mode::Full), expected);
    assert_eq!(page.print_to_string(Mode::Display), expected);
    assert_eq!(ProvenancePage::parse(&expected), Ok(page));
    assert!(ProvenancePage::parse("[s2-provenance-dump]\n").is_err());
}

fn text_page(value: &str, span: Span) -> Page {
    Page {
        ops: vec![Op::Text(Text {
            content: String::from(value),
            span,
        })],
    }
}

#[test]
fn recipe4_preserves_relative_offsets_and_before_base_injectivity() {
    assert_eq!(<Page as KeyedArtifact>::SCHEMA_VERSION, 4);
    assert_eq!(
        ArtifactKey::of(&tree(0), 0),
        ArtifactKey::of(&tree(128), 128)
    );
    assert_ne!(
        ArtifactKey::of(&text_page("same", Span::new(3, 5)), 10),
        ArtifactKey::of(&text_page("same", Span::new(0, 0)), 10)
    );
    assert_ne!(
        ArtifactKey::of(&text_page("same", Span::new(3, 5)), 0),
        ArtifactKey::of(&text_page("changed", Span::new(3, 5)), 0)
    );
    assert_ne!(
        ArtifactKey::of(&text_page("same", Span::new(3, 5)), 0),
        ArtifactKey::of(&text_page("same", Span::new(3, 6)), 0)
    );
}

#[test]
fn typed_slots_project_only_to_the_identical_frozen_legacy_wire_record() {
    use vize_l2::dump::{Binding, Component, SlotsObject, VueDirective};
    let value = Expr::Js {
        source: String::from(r#"pick("猫&")"#),
        span: Span::new(9, 21),
    };
    let span = Span::new(0, 31);
    let page = |binding| Page {
        ops: vec![Op::Component(Component {
            name: String::from("B"),
            attributes: vec![],
            bindings: vec![binding],
            children: vec![],
            span: Span::new(0, 32),
        })],
    };
    let typed = page(Binding::SlotsObject(SlotsObject {
        value: value.clone(),
        span,
    }));
    let legacy = page(Binding::VueDirective(VueDirective {
        name: String::from("slots"),
        argument: None,
        modifiers: vec![],
        value: Some(value),
        span,
    }));
    let old = v1(r#"[@V1@]
ops=2

[@V1@.ops]
ui.component B @0:32
  vue.directive "slots" value=js("pick(\"猫&\")" @9:21) @0:31

"#);
    let current = r#"[l2-dump-v2]
ops=2

[l2-dump-v2.ops]
ui.component B @0:32
  ui.slots-object value=js("pick(\"猫&\")" @9:21) @0:31

"#;
    assert_eq!(typed.print_to_string(Mode::Full).as_str(), current);
    assert_eq!(Page::parse(current), Ok(typed.clone()));
    assert_eq!(
        HistoricalPage::from_current(typed)
            .print_to_string(Mode::Full)
            .as_str(),
        old.as_str()
    );
    assert_eq!(
        HistoricalPage::from_current(legacy.clone())
            .print_to_string(Mode::Full)
            .as_str(),
        old.as_str()
    );
    assert_eq!(
        HistoricalPage::parse(&old)
            .expect("old grammar")
            .into_current(),
        legacy
    );
    let forged = old.replace("vue.directive \"slots\"", "ui.slots-object");
    assert!(
        HistoricalPage::parse(&forged).is_err(),
        "old header does not grant the typed opcode"
    );
}
