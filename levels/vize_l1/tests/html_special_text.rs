#![cfg(test)]

use serde_json::{Value, json};
use vize_l0::{Allocator, SourceRoot, Span, String};
use vize_l1::markup::document::{DocumentHtmlNode, DocumentHtmlRefusal as Refusal, NativeDocument};

const CORPUS: &str = include_str!("fixtures/document/html-special-text.json");

#[test]
fn special_text_packets_match_chromium_and_the_shipped_parser() {
    let cases: Vec<Value> = serde_json::from_str(CORPUS).unwrap();
    assert_eq!(cases.len(), 8);
    for case in cases {
        let label = case["name"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
        let tree = owner.html_structure().expect(label);
        let elements: Vec<_> = tree.elements().collect();
        let actual_elements: Vec<_> = elements
            .iter()
            .map(|element| {
                assert!(core::ptr::eq(element.owner(), &owner));
                assert_eq!(element.opening_span().slice(source), element.opening());
                let parent = element.parent().map(|parent| {
                    elements
                        .iter()
                        .position(|candidate| candidate.opening_span() == parent.opening_span())
                        .unwrap()
                });
                let attributes: Vec<_> = element
                    .attributes()
                    .map(|attribute| {
                        let name: String = attribute.name_characters().collect();
                        let value: String = attribute.value_characters().collect();
                        json!([name, value])
                    })
                    .collect();
                json!([element.name(), parent, attributes])
            })
            .collect();
        assert_eq!(
            json!(actual_elements),
            case["elements"],
            "elements: {label}"
        );

        let actual_nodes: Vec<_> = elements
            .iter()
            .filter(|element| !matches!(element.name(), "html" | "head"))
            .map(|element| {
                let children: Vec<_> = element
                    .child_nodes()
                    .expect(label)
                    .map(|child| match child.expect(label) {
                        DocumentHtmlNode::Element(child) => {
                            assert_eq!(
                                child.parent().unwrap().opening_span(),
                                element.opening_span()
                            );
                            json!([1, child.name()])
                        }
                        DocumentHtmlNode::Text(text) => {
                            assert!(core::ptr::eq(text.owner(), &owner));
                            assert_eq!(text.parent().opening_span(), element.opening_span());
                            assert_eq!(text.span().slice(source), text.authored());
                            assert!(
                                text.tokens()
                                    .all(|token| { core::ptr::eq(token.owner(), &owner) })
                            );
                            let value: String = text.characters().collect();
                            json!([3, value])
                        }
                        DocumentHtmlNode::Comment(comment) => {
                            let value: String = comment.characters().collect();
                            json!([8, value])
                        }
                    })
                    .collect();
                json!([element.name(), children])
            })
            .collect();
        assert_eq!(json!(actual_nodes), case["nodes"], "nodes: {label}");
        assert!(core::ptr::eq(owner.source(), source));
    }
}

#[test]
fn unsupported_special_text_edges_keep_precise_refusals() {
    let envelope = |head: &str, body: &str| {
        vize_l0::cstr!("<!DOCTYPE html><html><head>{head}</head><body>{body}</body></html>")
    };
    for (head, body, name) in [
        ("<script/>x</script>", "", "script"),
        ("", "<script/>x</script>", "script"),
        ("", "<style/>x</style>", "style"),
        ("", "<title/>x</title>", "title"),
        ("", "<textarea/>x</textarea>", "textarea"),
    ] {
        let source = envelope(head, body);
        let start = source.find(name).unwrap() as u32;
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
        assert_eq!(
            owner.html_structure().unwrap_err(),
            Refusal::UnsupportedElement(Span::new(start, start + name.len() as u32)),
            "{source}"
        );
    }
    for body in [
        "<title>{{ value }}</title>",
        "<textarea>{{ value }}</textarea>",
    ] {
        let source = envelope("", body);
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
        assert_eq!(owner.html_structure().unwrap_err(), Refusal::VueSyntax);
    }
    for body in [
        "<script>\0x</script>",
        "<style>\0x</style>",
        "<title>\0x</title>",
        "<textarea>\0\nX</textarea>",
    ] {
        let source = envelope("", body);
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
        assert_eq!(
            owner.html_structure().unwrap_err(),
            Refusal::NonHtmlToken,
            "{body:?}"
        );
    }
}
