//! Attribute tokens retain authored spelling independently of value delimiters.

use vize_l0::dump::{Dump, Mode};
use vize_l0::{Span, String};
use vize_l2::dump::{Attribute, Element, Op, Page};
use vize_l2::op::Namespace;

fn page(name: &str, value: Option<&str>) -> Page {
    Page {
        ops: vec![Op::Element(Element {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![Attribute {
                name: String::from(name),
                value: value.map(String::from),
                span: Span::new(1, 30),
            }],
            bindings: vec![],
            children: vec![],
            span: Span::new(0, 40),
        })],
    }
}

#[test]
fn attribute_names_round_trip_without_becoming_value_syntax() {
    let names = [
        "flag",
        "data-note",
        "aria-label",
        "x:y",
        "meta=value",
        "two words",
        "quote\"name",
        "\"leading",
        "tab\tname",
        "line\nname",
        "cr\rname",
        "nonbreaking\u{a0}space",
        "back\\slash",
        "[argument]",
    ];
    for name in names {
        for value in [None, Some(""), Some("body=\"text\"\nnext\\line")] {
            let original = page(name, value);
            let printed = original.print_to_string(Mode::Full);
            let reparsed = Page::parse(printed.as_str()).expect("complete attribute token parses");
            assert_eq!(reparsed, original, "name={name:?}, value={value:?}");
            assert_eq!(reparsed.print_to_string(Mode::Full), printed);
        }
    }
}

#[test]
fn ordinary_attribute_tokens_keep_the_existing_bytes_in_both_modes() {
    let original = page("data-note", Some("body"));
    assert_eq!(
        original.print_to_string(Mode::Full).as_str(),
        "[l2-dump-v2]\nops=1\n\n[l2-dump-v2.ops]\nui.element div @0:40\n  attr data-note=\"body\" @1:30\n\n",
    );
    assert_eq!(
        original.print_to_string(Mode::Display).as_str(),
        "[l2-dump-v2]\nops=1\n\n[l2-dump-v2.ops]\nui.element div\n  attr data-note=\"body\"\n\n",
    );
}

#[test]
fn quoted_attribute_tokens_reject_incomplete_or_ambiguous_tails() {
    for token in [
        "\"\" @1:30",
        "\"unfinished @1:30",
        "\"bad\\q\" @1:30",
        "\"name\"junk @1:30",
        "\"name\"=unquoted @1:30",
        "\"name\"=\"body\" extra @1:30",
    ] {
        let source = format!(
            "[l2-dump-v2]\nops=1\n\n[l2-dump-v2.ops]\nui.element div @0:40\n  attr {token}\n\n"
        );
        assert!(Page::parse(&source).is_err(), "invalid token {token:?}");
    }
}
