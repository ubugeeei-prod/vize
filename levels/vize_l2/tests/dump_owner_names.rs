//! Current owner tokens preserve arbitrary names without changing historical pages.

use vize_l0::dump::{Dump, Mode};
use vize_l0::key::ArtifactKey;
use vize_l0::{Span, String};
use vize_l2::dump::{Attribute, Component, Element, Op, Page};
use vize_l2::op::Namespace;

fn pages(name: &str) -> Vec<Page> {
    let mut result = vec![Page {
        ops: vec![Op::Component(Component {
            name: String::from(name),
            attributes: vec![],
            bindings: vec![],
            children: vec![],
            span: Span::new(0, 99),
        })],
    }];
    for namespace in [Namespace::Html, Namespace::Svg, Namespace::MathMl] {
        result.push(Page {
            ops: vec![Op::Element(Element {
                tag: String::from(name),
                namespace,
                attributes: vec![],
                bindings: vec![],
                children: vec![],
                span: Span::new(0, 99),
            })],
        });
    }
    result
}

#[test]
fn owner_names_round_trip_without_injecting_lines_or_metadata() {
    for name in [
        "",
        "div",
        "MyComponent",
        "Foo Bar",
        "Foo\nBar",
        "Foo\r\nBar",
        "Foo\tBar",
        "\"Quoted\"",
        "quote\"inside",
        "back\\slash",
        "日本🧭",
        "ns=svg",
        "@11562:11601n",
        "\u{85}",
        "\u{a0}",
        "\u{2028}",
        "\0",
        "Foo\u{1d}Bar",
    ] {
        for original in pages(name) {
            let printed = original.print_to_string(Mode::Full);
            let parsed = Page::parse(&printed).unwrap();
            assert_eq!(parsed, original, "{name:?}");
            assert_eq!(parsed.print_to_string(Mode::Full), printed);
        }
    }
}

#[test]
fn ordinary_names_keep_the_existing_current_and_historical_bytes() {
    let original = pages("MyComponent").remove(0);
    assert_eq!(
        original.print_to_string(Mode::Full).as_str(),
        "[l2-dump-v2]\nops=1\n\n[l2-dump-v2.ops]\nui.component MyComponent @0:99\n\n"
    );
    assert_eq!(
        original.print_to_string(Mode::Display).as_str(),
        "[l2-dump-v2]\nops=1\n\n[l2-dump-v2.ops]\nui.component MyComponent\n\n"
    );
    let historical =
        vize_l2::dump::historical::v1::Page::from_current(pages("\"Quoted\"").remove(0));
    let printed = historical.print_to_string(Mode::Full);
    assert_eq!(
        printed.as_str(),
        "[disegno]\nops=1\n\n[disegno.ops]\nui.component \"Quoted\" @0:99\n\n"
    );
    assert_eq!(
        vize_l2::dump::historical::v1::Page::parse(&printed).unwrap(),
        historical
    );
}

#[test]
fn malformed_quoted_names_keep_exact_error_locations() {
    for op in ["ui.element", "ui.component"] {
        for (body, message) in [
            ("\"unterminated @0:99", "unterminated quoted string"),
            ("\"bad\\q\" @0:99", "invalid escape `\\q`"),
            ("\"ok\"junk @0:99", "trailing content `junk @0:99`"),
            ("\"ok\"", "missing span"),
        ] {
            let source = format!("[l2-dump-v2]\nops=1\n\n[l2-dump-v2.ops]\n{op} {body}\n\n");
            let error = Page::parse(&source).unwrap_err();
            assert_eq!((error.line, error.message.as_str()), (5, message));
        }
    }
}

#[test]
fn an_owner_name_cannot_collide_with_a_real_attached_attribute() {
    let named = pages("Foo @0:99\n  attr injected").remove(0);
    let attached = Page {
        ops: vec![Op::Component(Component {
            name: String::from("Foo"),
            attributes: vec![Attribute {
                name: String::from("injected"),
                value: None,
                span: Span::new(0, 99),
            }],
            bindings: vec![],
            children: vec![],
            span: Span::new(0, 99),
        })],
    };
    assert_ne!(named, attached);
    // Retain the original wire collision as evidence; historical pages have
    // no artifact-key producer. Current bytes and keys must distinguish it.
    let historical =
        |page| vize_l2::dump::historical::v1::Page::from_current(page).print_to_string(Mode::Full);
    assert_eq!(historical(named.clone()), historical(attached.clone()));
    assert_ne!(
        named.print_to_string(Mode::Full),
        attached.print_to_string(Mode::Full)
    );
    assert_ne!(ArtifactKey::of(&named, 0), ArtifactKey::of(&attached, 0));
    assert_eq!(Page::parse(&named.print_to_string(Mode::Full)), Ok(named));
}
