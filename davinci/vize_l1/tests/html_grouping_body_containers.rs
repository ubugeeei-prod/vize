#![cfg(test)]

use serde_json::{Value, json};
use vize_l0::{Allocator, SourceRoot, String, cstr};
use vize_l1::markup::document::{
    DocumentHtmlNode, DocumentHtmlRefusal, DocumentTreePolicy, NativeDocument,
};

const CORPUS: &str = include_str!("fixtures/document/html-grouping-body-containers.json");
const TAGS: [&str; 6] = [
    "address",
    "blockquote",
    "figure",
    "figcaption",
    "hgroup",
    "search",
];

#[test]
fn original_grouping_containers_match_complete_browser_packets() {
    let cases: Vec<Value> = serde_json::from_str(CORPUS).unwrap();
    assert_eq!(cases.len(), 8);
    for case in &cases {
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
        let frames: Vec<_> = elements
            .iter()
            .filter(|element| TAGS.contains(&element.name()))
            .map(|element| {
                json!([
                    element.name(),
                    element.opening(),
                    element.closing_span().expect(label).slice(source),
                    element.ignored_self_closing_slash()
                ])
            })
            .collect();
        assert_eq!(json!(frames), case["frames"], "{label}");
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
                json!([element.name(), children])
            })
            .collect();
        assert_eq!(json!(nodes), case["nodes"], "{label}");
        assert!(core::ptr::eq(owner.source(), source));
    }
}

#[test]
fn grouping_names_keep_wrong_mode_implied_end_and_vue_refusals() {
    for tag in TAGS {
        for (source, expected) in [
            (
                cstr!("<!DOCTYPE html><html><head><{tag}></head><body></body></html>"),
                "envelope",
            ),
            (
                cstr!("<!DOCTYPE html><html><head></head><body><{tag}/></body></html>"),
                "end",
            ),
            (
                cstr!(
                    "<!DOCTYPE html><html><head></head><body><{tag}><span></{tag}></span></body></html>"
                ),
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
    for (body, expected) in [
        ("<p><figure></figure></p>", "implied"),
        ("<table><figure></figure></table>", "table"),
        ("<svg><figure></figure></svg>", "element"),
        ("<script><figure></figure></script>", "element"),
        ("<b><figure></figure></b>", "element"),
    ] {
        let source = cstr!("<!DOCTYPE html><html><head></head><body>{body}</body></html>");
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
        assert!(matches!(
            (expected, owner.html_structure()),
            (
                "implied",
                Err(DocumentHtmlRefusal::UnsupportedPolicy(
                    DocumentTreePolicy::ImpliedEndTags
                ))
            ) | (
                "table",
                Err(DocumentHtmlRefusal::UnsupportedPolicy(
                    DocumentTreePolicy::TableContentModel
                ))
            ) | ("element", Err(DocumentHtmlRefusal::UnsupportedElement(_)))
        ));
    }
}
