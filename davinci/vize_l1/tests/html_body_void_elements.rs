#![cfg(test)]

use serde_json::{Value, json};
use vize_l0::{Allocator, SourceRoot, String, cstr};
use vize_l1::markup::document::{DocumentHtmlNode, DocumentHtmlRefusal, NativeDocument};

const CORPUS: &str = include_str!("fixtures/document/html-body-void-elements.json");
const TAGS: [&str; 5] = ["area", "embed", "source", "track", "wbr"];

#[test]
fn original_body_void_elements_match_complete_browser_packets() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let cases = corpus.as_array().unwrap();
    assert_eq!(cases.len(), 10);
    for case in cases {
        let label = case["name"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
        let tree = owner.html_structure().expect(label);
        let elements: Vec<_> = tree.elements().collect();
        let actual: Vec<_> = elements
            .iter()
            .map(|element| {
                assert!(core::ptr::eq(element.owner(), &owner));
                assert_eq!(element.opening_span().slice(source), element.opening());
                let parent = element.parent().map(|parent| {
                    elements
                        .iter()
                        .position(|element| element.opening_span() == parent.opening_span())
                        .unwrap()
                });
                let attributes: Vec<_> = element
                    .attributes()
                    .map(|attribute| {
                        assert_eq!(attribute.element().opening_span(), element.opening_span());
                        let name: String = attribute.name_characters().collect();
                        let value: String = attribute.value_characters().collect();
                        json!([name, value])
                    })
                    .collect();
                json!([element.name(), parent, attributes])
            })
            .collect();
        assert_eq!(json!(actual), case["elements"], "{label}");
        let nodes: Vec<_> = elements
            .iter()
            .filter(|element| !matches!(element.name(), "html" | "head"))
            .map(|element| {
                let children: Vec<_> = element
                    .child_nodes()
                    .expect(label)
                    .map(|node| match node.expect(label) {
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
                            let value: String = text.characters().collect();
                            json!([3, value])
                        }
                        DocumentHtmlNode::Comment(comment) => {
                            assert!(core::ptr::eq(comment.owner(), &owner));
                            assert_eq!(comment.parent().opening_span(), element.opening_span());
                            let value: String = comment.characters().collect();
                            json!([8, value])
                        }
                    })
                    .collect();
                if TAGS.contains(&element.name()) {
                    assert!(element.closing_span().is_none());
                    assert!(!element.ignored_self_closing_slash());
                    assert!(children.is_empty(), "{label}");
                    assert_eq!(element.children().count(), 0);
                }
                json!([element.name(), children])
            })
            .collect();
        assert_eq!(json!(nodes), case["nodes"], "{label}");
        assert!(core::ptr::eq(owner.source(), source));
    }
}

#[test]
fn new_void_names_do_not_admit_wrong_modes_prefixes_or_explicit_closes() {
    for tag in TAGS {
        for (source, expected) in [
            (
                cstr!("<!DOCTYPE html><html><head><{tag}></head><body></body></html>"),
                "envelope",
            ),
            (
                cstr!("<!DOCTYPE html><html><head></head><body><{tag}></{tag}></body></html>"),
                "end",
            ),
            (
                cstr!("<!DOCTYPE html><html><head></head><body><{tag}-x></{tag}-x></body></html>"),
                "element",
            ),
            (
                cstr!(
                    "<!DOCTYPE html><html><head></head><body><{tag} v-scope=state></body></html>"
                ),
                "vue",
            ),
        ] {
            let arena = Allocator::default();
            let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
            let refusal = owner.html_structure().unwrap_err();
            assert!(
                matches!(
                    (expected, refusal),
                    ("envelope", DocumentHtmlRefusal::ExplicitEnvelope)
                        | ("end", DocumentHtmlRefusal::ImpliedEnd(_))
                        | ("element", DocumentHtmlRefusal::UnsupportedElement(_))
                        | ("vue", DocumentHtmlRefusal::VueSyntax)
                ),
                "{source}: {refusal:?}"
            );
        }
    }
    let source = "<!DOCTYPE html><html><head></head><body><col></body></html>";
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
    assert!(matches!(
        owner.html_structure(),
        Err(DocumentHtmlRefusal::UnsupportedPolicy(_))
    ));
}
