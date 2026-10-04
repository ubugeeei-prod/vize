use alloc::vec::Vec;
use vize_l0::{Allocator, SourceRoot, String};

use super::super::DocumentHtmlStructure;
use super::super::{DocumentHtmlRefusal, NativeDocument};
use crate::markup::{QuoteType, document::DocumentLexicalRefusal};

const CORPUS: &str = include_str!("../../../../../tests/fixtures/document/html-attributes.json");

fn root(source: &str) -> SourceRoot<'_> {
    SourceRoot::new(source).expect("original small source")
}

fn envelope(body: &str) -> String {
    let mut source = String::from("<!DOCTYPE html><html><head></head><body>");
    source.push_str(body);
    source.push_str("</body></html>");
    source
}

#[test]
fn original_static_attributes_match_the_actual_chromium_census() {
    let corpus: serde_json::Value = serde_json::from_str(CORPUS).expect("shared original corpus");
    let cases = corpus.as_array().expect("whole source cases");
    assert_eq!(cases.len(), 10);
    for case in cases {
        let name = case["name"].as_str().expect("case name");
        let source = case["source"].as_str().expect("whole original HTML");
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(source));
        let tree = owner.html_structure().expect(name);
        let actual: Vec<Vec<(String, String)>> = tree
            .elements()
            .map(|element| {
                element
                    .attributes()
                    .map(|attribute| {
                        assert!(core::ptr::eq(attribute.element().owner(), &owner));
                        assert_eq!(attribute.element().opening_span(), element.opening_span());
                        (
                            attribute.name_characters().collect(),
                            attribute.value_characters().collect(),
                        )
                    })
                    .collect()
            })
            .collect();
        assert_eq!(
            serde_json::to_value(actual).expect("test-only census"),
            case["attributes"],
            "{name}"
        );
        assert_eq!(owner.source(), source);
        assert_eq!(owner.unfinished_tree_policies().len(), 4);
    }
}

#[test]
fn quote_and_empty_content_geometry_stays_in_original_bytes() {
    let source = envelope("<div A   B='' C = \"\" D=日本語 E='&amp;' F=http://x/></div>");
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(&source));
    let tree = owner.html_structure().expect("original HTML");
    let element = tree.elements().nth(3).expect("original div");
    let attributes: Vec<_> = element.attributes().collect();
    assert_eq!(attributes.len(), 6);
    let expected = [
        ("A", QuoteType::NoValue, None, ""),
        ("B", QuoteType::Single, Some(""), ""),
        ("C", QuoteType::Double, Some(""), ""),
        ("D", QuoteType::Unquoted, Some("日本語"), "日本語"),
        ("E", QuoteType::Single, Some("&amp;"), "&"),
        ("F", QuoteType::Unquoted, Some("http://x/"), "http://x/"),
    ];
    for (attribute, (name, quote, raw, value)) in attributes.iter().zip(expected) {
        assert_eq!(attribute.authored_name(), name);
        assert_eq!(attribute.name_span().slice(&source), name);
        assert_eq!(attribute.quote(), quote);
        assert_eq!(attribute.authored_value(), raw);
        assert_eq!(attribute.value_characters().collect::<String>(), value);
        if let Some(span) = attribute.value_span() {
            assert_eq!(Some(span.slice(&source)), raw);
            assert!(core::ptr::eq(
                attribute
                    .authored_value()
                    .expect("original content")
                    .as_ptr(),
                source[span.start as usize..].as_ptr()
            ));
        }
    }
}

#[test]
fn original_attribute_receipts_survive_owner_and_structure_moves() {
    let source = envelope("<div DATA-X='&NotEqualTilde;'></div>");
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(&source));
    let moved = owner;
    let tree = moved.html_structure().expect("moved genuine owner");
    let moved_tree = tree;
    let element = moved_tree.elements().nth(3).expect("original div");
    let attribute = element.attributes().next().expect("original attribute");
    assert!(core::ptr::eq(attribute.element().owner(), &moved));
    assert!(core::ptr::eq(
        attribute.element().owner().allocator,
        &allocator
    ));
    assert_eq!(attribute.authored_name(), "DATA-X");
    assert_eq!(attribute.authored_value(), Some("&NotEqualTilde;"));
    assert_eq!(attribute.value_characters().collect::<String>(), "≂\u{338}");
    assert_eq!(attribute.name_characters().collect::<String>(), "data-x");
}

#[test]
fn equal_buffers_do_not_transfer_attribute_owner_authority() {
    let source = envelope("<div x='&amp;'></div>");
    let first_arena = Allocator::default();
    let second_arena = Allocator::default();
    let first = NativeDocument::lex_in(&first_arena, root(&source));
    let second = NativeDocument::lex_in(&second_arena, root(&source));
    let first_tree = first.html_structure().expect("first genuine tree");
    let second_tree = second.html_structure().expect("second genuine tree");
    let first_element = first_tree.elements().nth(3).expect("first div");
    let second_element = second_tree.elements().nth(3).expect("second div");
    let first_attribute = first_element.attributes().next().expect("first attribute");
    let second_attribute = second_element
        .attributes()
        .next()
        .expect("second attribute");
    assert_eq!(first_attribute.name_span(), second_attribute.name_span());
    assert_eq!(first_attribute.value_span(), second_attribute.value_span());
    assert!(core::ptr::eq(first_attribute.element().owner(), &first));
    assert!(core::ptr::eq(second_attribute.element().owner(), &second));
    assert!(!core::ptr::eq(
        first_attribute.element().owner(),
        second_attribute.element().owner()
    ));
}

#[test]
fn duplicate_boolean_values_and_interleaved_entities_keep_first_original_frame() {
    let source = envelope(
        "<div X x='second' Y='&NotEqualTilde;&amp;z' y='fourth'></div><span X='sibling'></span>",
    );
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(&source));
    let tree = owner
        .html_structure()
        .expect("original duplicate attributes");
    let div = tree.elements().nth(3).expect("original div");
    let attributes: Vec<_> = div.attributes().collect();
    assert_eq!(attributes.len(), 2);
    assert_eq!(attributes[0].authored_name(), "X");
    assert_eq!(attributes[0].quote(), QuoteType::NoValue);
    assert_eq!(attributes[0].authored_value(), None);
    assert_eq!(
        attributes[1].authored_value(),
        Some("&NotEqualTilde;&amp;z")
    );
    assert_eq!(
        attributes[1].value_characters().collect::<String>(),
        "≂\u{338}&z"
    );
    let span = tree.elements().nth(4).expect("original sibling");
    assert_eq!(
        span.attributes()
            .next()
            .expect("own attribute")
            .authored_value(),
        Some("sibling")
    );
}

#[test]
fn recovered_pending_vue_and_unsupported_original_frames_never_mint_attributes() {
    for body in [
        "<div a='",
        "<div a=></div>",
        "<div a='x'b='y'></div>",
        "<div a=\"x\"",
        "<div a=x`></div>",
    ] {
        let source = envelope(body);
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(&source));
        assert!(
            matches!(owner.html_structure(), Err(DocumentHtmlRefusal::Lexical(_))),
            "{body}"
        );
    }
    for body in ["<div v-scope=state></div>", "<div :x='y'></div>"] {
        let source = envelope(body);
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(&source));
        assert_eq!(
            owner.html_structure().unwrap_err(),
            DocumentHtmlRefusal::VueSyntax
        );
    }
    let source = envelope("<div a=\"x\"b=y></div>");
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(&source));
    assert_eq!(
        owner.html_structure().unwrap_err(),
        DocumentHtmlRefusal::Lexical(DocumentLexicalRefusal::RecoveredSyntax)
    );
}

#[test]
fn actual_common_prefix_attribute_iteration_stays_inside_count_and_byte_admission() {
    let prefix = "DATA-".to_owned() + &"x".repeat(180);
    for count in [
        DocumentHtmlStructure::MAX_ATTRIBUTES,
        DocumentHtmlStructure::MAX_ATTRIBUTES + 1,
    ] {
        let mut body = String::from("<div");
        for ordinal in 0..count {
            body.push(' ');
            body.push_str(&prefix);
            body.push_str(&ordinal.to_string());
            body.push_str("='&NotEqualTilde;&#13;\r\n&amp;amp;'");
        }
        body.push_str("></div>");
        let source = envelope(&body);
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(&source));
        assert!(owner.normal_completion().is_ok());
        if count > DocumentHtmlStructure::MAX_ATTRIBUTES {
            assert!(matches!(
                owner.html_structure(),
                Err(DocumentHtmlRefusal::AttributeCountLimit(_))
            ));
            continue;
        }
        let tree = owner
            .html_structure()
            .expect("actual maximum admitted name workload");
        let div = tree.elements().nth(3).expect("original div");
        assert!(div.opening_span().len() <= DocumentHtmlStructure::MAX_OPENING_BYTES);
        let actual: Vec<_> = div.attributes().collect();
        assert_eq!(actual.len(), count);
        for (ordinal, attribute) in actual.iter().enumerate() {
            let expected = prefix.to_ascii_lowercase() + &ordinal.to_string();
            assert_eq!(attribute.name_characters().collect::<String>(), expected);
            assert_eq!(
                attribute.value_characters().collect::<String>(),
                "≂\u{338}\r\n&amp;"
            );
        }
    }
    let mut body = String::from("<div");
    for _ in 0..DocumentHtmlStructure::MAX_ATTRIBUTES + 1 {
        body.push_str(" X='original'");
    }
    body.push_str("></div>");
    let source = envelope(&body);
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(&source));
    assert!(matches!(
        owner.html_structure(),
        Err(DocumentHtmlRefusal::AttributeCountLimit(_))
    ));
}

#[test]
fn exact_opening_byte_limit_iterates_values_and_over_limit_retains_original_refusal() {
    let prefix = "<div data-x='";
    let suffix = "'>";
    for extra in [0, 1] {
        let size = DocumentHtmlStructure::MAX_OPENING_BYTES as usize + extra;
        let value = "x".repeat(size - prefix.len() - suffix.len());
        let mut body = String::from(prefix);
        body.push_str(&value);
        body.push_str(suffix);
        body.push_str("</div>");
        let source = envelope(&body);
        let allocator = Allocator::default();
        let owner = NativeDocument::lex_in(&allocator, root(&source));
        assert!(owner.normal_completion().is_ok());
        if extra > 0 {
            assert!(matches!(
                owner.html_structure(),
                Err(DocumentHtmlRefusal::OpeningByteLimit(_))
            ));
            continue;
        }
        let tree = owner.html_structure().expect("exact original byte limit");
        let div = tree.elements().nth(3).expect("original div");
        assert_eq!(
            div.opening_span().len(),
            DocumentHtmlStructure::MAX_OPENING_BYTES
        );
        let attribute = div.attributes().next().expect("original static value");
        assert_eq!(attribute.authored_value(), Some(value.as_str()));
        assert_eq!(attribute.value_characters().count(), value.len());
        assert!(attribute.value_characters().all(|ch| ch == 'x'));
    }
}
