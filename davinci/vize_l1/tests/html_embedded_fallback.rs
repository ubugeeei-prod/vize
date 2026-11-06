#![cfg(test)]

use serde_json::{Value, json};
use vize_l0::{Allocator, Namespace, SourceRoot, String};
use vize_l1::markup::document::{DocumentHtmlNode, DocumentHtmlRefusal as Refusal, NativeDocument};

const CORPUS: &str = include_str!("fixtures/document/html-embedded-fallback.json");

fn namespace(value: Namespace) -> &'static str {
    match value {
        Namespace::Html => "html",
        Namespace::Svg => "svg",
        Namespace::MathMl => "mathml",
    }
}

#[test]
fn embedded_fallback_packets_retain_the_complete_browser_and_legacy_tree() {
    let cases: Vec<Value> = serde_json::from_str(CORPUS).unwrap();
    assert_eq!(cases.len(), 5);
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
                json!([
                    element.name(),
                    namespace(element.namespace()),
                    parent,
                    attributes
                ])
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
                        DocumentHtmlNode::Element(child) => json!([1, child.name()]),
                        DocumentHtmlNode::Text(text) => {
                            assert!(core::ptr::eq(text.owner(), &owner));
                            let value: String = text.characters().collect();
                            json!([3, value])
                        }
                        DocumentHtmlNode::Comment(comment) => {
                            assert!(core::ptr::eq(comment.owner(), &owner));
                            let value: String = comment.characters().collect();
                            json!([8, value])
                        }
                    })
                    .collect();
                json!([element.name(), children])
            })
            .collect();
        assert_eq!(json!(actual_nodes), case["nodes"], "nodes: {label}");
    }
}

#[test]
fn iframe_raw_text_disagreements_keep_typed_fallbacks() {
    for body in [
        "<iframe>&amp;</iframe>",
        "<iframe><!--comment--></iframe>",
        "<iframe><b>markup</b></iframe>",
        "<iframe/>fallback</iframe>",
        "<noscript/><span>fallback</span></noscript>",
    ] {
        let source = vize_l0::cstr!("<!DOCTYPE html><html><head></head><body>{body}</body></html>");
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
        assert!(
            matches!(owner.html_structure(), Err(Refusal::UnsupportedElement(_))),
            "{body}"
        );
    }

    let source = "<!DOCTYPE html><html><head></head><body><iframe>\0</iframe></body></html>";
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
    assert_eq!(owner.html_structure().unwrap_err(), Refusal::NonHtmlToken);
}
