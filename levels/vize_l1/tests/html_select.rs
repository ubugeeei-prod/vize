#![cfg(test)]

use serde_json::{Value, json};
use vize_l0::{Allocator, SourceRoot, Span, String};
use vize_l1::markup::document::{
    DocumentHtmlNode, DocumentHtmlRefusal as Refusal, DocumentTokenKind, NativeDocument,
};

const CORPUS: &str = include_str!("fixtures/document/html-select.json");

fn span(value: Span) -> [u32; 2] {
    [value.start, value.end]
}

#[test]
fn select_packets_retain_complete_original_tags_frames_attributes_and_child_nodes() {
    let cases: Vec<Value> = serde_json::from_str(CORPUS).unwrap();
    assert_eq!(cases.len(), 13);
    for case in &cases {
        let label = case["name"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
        assert_eq!(owner.source(), source);
        if let Some(refusal) = case.get("refusal") {
            let coordinates = refusal["span"].as_array().unwrap();
            let at = Span::new(
                coordinates[0].as_u64().unwrap() as u32,
                coordinates[1].as_u64().unwrap() as u32,
            );
            let expected = match refusal["kind"].as_str().unwrap() {
                "UnsupportedSelectContent" => Refusal::UnsupportedSelectContent(at),
                "IgnoredTag" => Refusal::IgnoredTag(at),
                "ImpliedEnd" => Refusal::ImpliedEnd(at),
                _ => panic!("unregistered refusal"),
            };
            assert_eq!(owner.html_structure().unwrap_err(), expected, "{label}");
            continue;
        }
        let tags: Vec<_> = owner
            .tokens()
            .filter_map(|token| {
                assert!(core::ptr::eq(token.owner(), &owner));
                let kind = match token.kind() {
                    DocumentTokenKind::OpenTagName => "open",
                    DocumentTokenKind::CloseTagName => "close",
                    _ => return None,
                };
                Some(json!([kind, span(token.span()), token.source().unwrap()]))
            })
            .collect();
        assert_eq!(json!(tags), case["tags"], "{label}");
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
                json!([element.name(), parent, attributes])
            })
            .collect();
        assert_eq!(json!(actual), case["elements"], "{label}");
        let frames: Vec<_> = elements
            .iter()
            .map(|element| {
                json!([
                    element.name(),
                    span(element.opening_span()),
                    element.closing_span().map(span),
                    element.ignored_self_closing_slash(),
                ])
            })
            .collect();
        assert_eq!(json!(frames), case["frames"], "{label}");
        let implied: Vec<_> = elements
            .iter()
            .filter(|element| element.closing_span().is_none() && element.name() != "hr")
            .map(|element| element.name())
            .collect();
        assert_eq!(json!(implied), case["implied"], "{label}");
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
                                element.opening_span(),
                            );
                            json!([1, child.name()])
                        }
                        DocumentHtmlNode::Text(text) => {
                            assert_eq!(text.parent().opening_span(), element.opening_span());
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
        assert_eq!(json!(nodes), case["nodes"], "{label}");
    }
}

#[test]
fn unfinished_select_frames_and_cross_table_transitions_keep_precise_refusals() {
    for (body, kind, token) in [
        ("</select>", "end", "select"),
        ("<select><option>a</body>", "end", "body"),
        ("<select><option>a</optgroup>", "end", "optgroup"),
        ("<table><select></select></table>", "foster", "select"),
        (
            "<table><tbody><tr><td><select><option>a</td></tr></tbody></table>",
            "end",
            "td",
        ),
    ] {
        let source = vize_l0::cstr!("<!DOCTYPE html><html><head></head><body>{body}</body></html>");
        let prefix = if kind == "foster" { "<" } else { "</" };
        let needle = vize_l0::cstr!("{prefix}{token}>");
        let start = source.find(needle.as_str()).unwrap() + prefix.len();
        let at = Span::new(start as u32, (start + token.len()) as u32);
        let expected = if kind == "foster" {
            Refusal::FosterParenting(at)
        } else {
            Refusal::ImpliedEnd(at)
        };
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
        assert_eq!(owner.html_structure().unwrap_err(), expected, "{body}");
    }
}
