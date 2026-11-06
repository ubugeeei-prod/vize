#![cfg(test)]

use serde_json::{Value, json};
use vize_l0::{Allocator, Namespace, SourceRoot, Span, String};
use vize_l1::markup::document::{DocumentHtmlNode, DocumentHtmlRefusal as Refusal, NativeDocument};

const CORPUS: &str = include_str!("fixtures/document/html-foreign-content.json");

fn namespace(value: Namespace) -> &'static str {
    match value {
        Namespace::Html => "html",
        Namespace::Svg => "svg",
        Namespace::MathMl => "mathml",
    }
}

#[test]
fn foreign_packets_retain_complete_namespaces_frames_attributes_and_nodes() {
    let cases: Vec<Value> = serde_json::from_str(CORPUS).unwrap();
    assert_eq!(cases.len(), 14);
    for case in &cases {
        let label = case["name"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
        assert_eq!(owner.source(), source);
        if let Some(refusal) = case.get("refusal") {
            let expected = match refusal["kind"].as_str().unwrap() {
                "UnsupportedElement" => {
                    let token = refusal["token"].as_str().unwrap();
                    let needle = vize_l0::cstr!("<{token}");
                    let start = source.find(needle.as_str()).unwrap() + 1;
                    Refusal::UnsupportedElement(Span::new(
                        start as u32,
                        (start + token.len()) as u32,
                    ))
                }
                "NonHtmlToken" => Refusal::NonHtmlToken,
                _ => panic!("unregistered refusal"),
            };
            assert_eq!(owner.html_structure().unwrap_err(), expected, "{label}");
            continue;
        }

        let tree = owner.html_structure().expect(label);
        let elements: Vec<_> = tree.elements().collect();
        let actual: Vec<_> = elements
            .iter()
            .map(|element| {
                assert!(core::ptr::eq(element.owner(), &owner));
                let parent = element.parent().map(|parent| {
                    elements
                        .iter()
                        .position(|node| node.opening_span() == parent.opening_span())
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
                    attributes,
                    element.opening(),
                    element.closing_span().map(|span| span.slice(source)),
                    element.ignored_self_closing_slash(),
                ])
            })
            .collect();
        assert_eq!(json!(actual), case["elements"], "elements: {label}");

        let nodes: Vec<_> = elements
            .iter()
            .filter(|element| !matches!(element.name(), "html" | "head"))
            .map(|element| {
                let children: Vec<_> = element
                    .child_nodes()
                    .expect(label)
                    .map(|node| match node.expect(label) {
                        DocumentHtmlNode::Element(child) => json!([1, child.name()]),
                        DocumentHtmlNode::Text(text) => {
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
        assert_eq!(json!(nodes), case["nodes"], "nodes: {label}");
    }
}

#[test]
fn unclosed_and_cross_namespace_foreign_frames_refuse_without_recovery_claims() {
    for (body, token, unsupported) in [
        ("<svg><g></svg>", "svg", false),
        ("<math><mrow></math>", "math", false),
        (
            "<svg><foreignObject><div></foreignObject></svg>",
            "foreignObject",
            true,
        ),
    ] {
        let source = vize_l0::cstr!("<!DOCTYPE html><html><head></head><body>{body}</body></html>");
        let needle = vize_l0::cstr!("</{token}>");
        let start = source.find(needle.as_str()).unwrap() + 2;
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
        let at = Span::new(start as u32, (start + token.len()) as u32);
        let expected = if unsupported {
            Refusal::UnsupportedElement(at)
        } else {
            Refusal::ImpliedEnd(at)
        };
        assert_eq!(owner.html_structure().unwrap_err(), expected, "{body}");
    }
}
