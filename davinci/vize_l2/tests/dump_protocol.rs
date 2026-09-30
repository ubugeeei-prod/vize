//! Independent current and historical wire oracles over the same owned IR.

use vize_davinci::dump::{Dump, Error, Mode};
use vize_davinci::key::{ArtifactKey, KeyedArtifact};
use vize_l0::{Span, String, cstr};
use vize_l2::dump::historical::v1::Page as HistoricalPage;
use vize_l2::dump::provenance::{Page as ProvenancePage, Record};
use vize_l2::dump::{Attribute, Comment, Element, Expr, Interpolation, Op, Page, Text};
use vize_l2::expr::OpaqueReason;
use vize_l2::op::Namespace;

const CURRENT: &str = r#"[l2-dump-v2]
ops=6

[l2-dump-v2.ops]
ui.element div @0:120
  attr title="[disegno]" @1:12
  ui.text "[disegno.ops]" @12:27
  ui.interpolation js("[l2-dump-v2]" @27:42) @27:42
  ui.comment "[l2-dump-v2.ops]" @42:60
  ui.interpolation foreign(md "[disegno]" @60:78) @60:78
  ui.interpolation opaque(multi-statement "[s2-provenance-folio]" @78:100) @78:100

"#;

const HISTORICAL: &str = r#"[disegno]
ops=6

[disegno.ops]
ui.element div @0:120
  attr title="[disegno]" @1:12
  ui.text "[disegno.ops]" @12:27
  ui.interpolation js("[l2-dump-v2]" @27:42) @27:42
  ui.comment "[l2-dump-v2.ops]" @42:60
  ui.interpolation foreign(md "[disegno]" @60:78) @60:78
  ui.interpolation opaque(multi-statement "[s2-provenance-folio]" @78:100) @78:100

"#;

fn tree(base: u32) -> Page {
    let span = |start, end| Span::new(base + start, base + end);
    Page {
        ops: vec![Op::Element(Element {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![Attribute {
                name: String::from("title"),
                value: Some(String::from("[disegno]")),
                span: span(1, 12),
            }],
            bindings: vec![],
            children: vec![
                Op::Text(Text {
                    content: String::from("[disegno.ops]"),
                    span: span(12, 27),
                }),
                Op::Interpolation(Interpolation {
                    expression: Expr::Js {
                        source: String::from("[l2-dump-v2]"),
                        span: span(27, 42),
                    },
                    span: span(27, 42),
                }),
                Op::Comment(Comment {
                    content: String::from("[l2-dump-v2.ops]"),
                    span: span(42, 60),
                }),
                Op::Interpolation(Interpolation {
                    expression: Expr::Foreign {
                        dialect: String::from("md"),
                        source: String::from("[disegno]"),
                        span: span(60, 78),
                    },
                    span: span(60, 78),
                }),
                Op::Interpolation(Interpolation {
                    expression: Expr::Opaque {
                        reason: OpaqueReason::MultiStatement,
                        source: String::from("[s2-provenance-folio]"),
                        span: span(78, 100),
                    },
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
    assert_eq!(page.print_to_string(Mode::Full).as_str(), CURRENT);
    assert_eq!(Page::parse(CURRENT), Ok(page.clone()));
    let old = HistoricalPage::from_current(page.clone());
    assert_eq!(old.print_to_string(Mode::Full).as_str(), HISTORICAL);
    assert_eq!(HistoricalPage::parse(HISTORICAL), Ok(old.clone()));
    assert_eq!(old.into_current(), page);
}

#[test]
fn current_and_historical_headers_never_select_each_other() {
    assert_eq!(
        Page::parse(HISTORICAL),
        Err(Error::new(1, cstr!("first section must be [l2-dump-v2]")))
    );
    assert_eq!(
        HistoricalPage::parse(CURRENT),
        Err(Error::new(1, cstr!("first section must be [disegno]")))
    );
    for (input, line, message) in [
        (
            "[l2-dump-v2]\nops=0\n[disegno.ops]\n",
            3,
            "unknown section [disegno.ops]",
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
        assert_eq!(
            Page::parse(input),
            Err(Error::new(line, String::from(message)))
        );
    }
    assert_eq!(
        HistoricalPage::parse("[disegno]\nops=0\n[l2-dump-v2.ops]\n"),
        Err(Error::new(3, cstr!("unknown section [l2-dump-v2.ops]")))
    );
}

#[test]
fn provenance_v2_preserves_quoted_headers_and_record_spans() {
    let page = ProvenancePage {
        records: vec![Record {
            rule: String::from("lower.text"),
            node: Some(7),
            before: String::from("[s2-provenance-folio]\n[disegno]"),
            after: String::from("[l2-provenance-dump-v2]"),
            span: Span::new(3, 42),
        }],
    };
    let expected = r#"[l2-provenance-dump-v2]

[l2-provenance-dump-v2.records]
rule=lower.text node=7 before="[s2-provenance-folio]\n[disegno]" after="[l2-provenance-dump-v2]" @3:42

"#;
    assert_eq!(page.print_to_string(Mode::Full).as_str(), expected);
    assert_eq!(page.print_to_string(Mode::Display).as_str(), expected);
    assert_eq!(ProvenancePage::parse(expected), Ok(page));
    assert!(ProvenancePage::parse("[s2-provenance-folio]\n").is_err());
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
fn recipe3_preserves_relative_offsets_and_before_base_injectivity() {
    assert_eq!(<Page as KeyedArtifact>::SCHEMA_VERSION, 3);
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
