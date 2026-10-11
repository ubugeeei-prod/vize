#![cfg(test)]

use serde_json::{Value, json};
use vize_l0::{Allocator, SourceRoot, String, cstr};
use vize_l1::markup::document::{
    DocumentHtmlNode, DocumentHtmlRefusal, DocumentTreePolicy, NativeDocument,
};

const CORPORA: [&str; 2] = [
    include_str!("fixtures/document/html-semantic-body-containers.json"),
    include_str!("fixtures/document/html-semantic-body-nesting.json"),
];
const FRAMES: &str = include_str!("fixtures/document/html-semantic-body-frames.json");
const TAGS: [&str; 7] = [
    "article", "aside", "footer", "header", "main", "nav", "section",
];

#[test]
fn original_semantic_containers_match_complete_browser_packets() {
    let cases: Vec<Value> = CORPORA
        .into_iter()
        .flat_map(|corpus| serde_json::from_str::<Vec<Value>>(corpus).unwrap())
        .collect();
    let frames: Vec<Value> = serde_json::from_str(FRAMES).unwrap();
    assert_eq!(cases.len(), 10);
    assert_eq!(frames.len(), 10);
    for (case, expected_frames) in cases.iter().zip(&frames) {
        assert_eq!(case["name"], expected_frames["name"]);
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
        assert_eq!(json!(frames), expected_frames["frames"], "{label}");
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
fn semantic_names_preserve_wrong_mode_implied_end_and_vue_refusals() {
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
        ("<p><article></article></p>", "implied"),
        ("<table><article></article></table>", "table"),
        ("<svg><article></article></svg>", "element"),
        ("<script><article></article></script>", "element"),
        ("<b><article></article></b>", "element"),
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
