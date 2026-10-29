#![cfg(test)]

use serde_json::{Value, json};
use vize_l0::{Allocator, SourceRoot, Span, String};
use vize_l1::markup::document::{DocumentHtmlNode, DocumentHtmlRefusal, NativeDocument};

const CORPUS: &str = include_str!("fixtures/document/html-implied-ends-and-tables.json");
const VOID: [&str; 3] = ["hr", "col", "br"];

#[test]
fn implied_ends_and_table_modes_match_complete_browser_packets() {
    let cases: Vec<Value> = serde_json::from_str(CORPUS).unwrap();
    assert_eq!(cases.len(), 12);
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
                let parent = element.parent().map(|parent| {
                    elements
                        .iter()
                        .position(|element| element.opening_span() == parent.opening_span())
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
        let implied: Vec<_> = elements
            .iter()
            .filter(|element| element.closing_span().is_none() && !VOID.contains(&element.name()))
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
                                element.opening_span()
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

/// Each refusal names the token where the browser's tree would stop
/// following authored tags.
#[test]
fn transitions_without_authored_tags_refuse_with_typed_reasons() {
    let at = |start: u32, end: u32| Span::new(40 + start, 40 + end);
    for (body, refusal) in [
        (
            "<table><tr><td>x</table>",
            DocumentHtmlRefusal::ImpliedStart(at(8, 10)),
        ),
        (
            "<table><col></table>",
            DocumentHtmlRefusal::ImpliedStart(at(8, 11)),
        ),
        (
            "<table><tbody><td>x</table>",
            DocumentHtmlRefusal::ImpliedStart(at(15, 17)),
        ),
        ("<table>x</table>", DocumentHtmlRefusal::TableText(at(7, 8))),
        (
            "<table><tbody>&amp;</table>",
            DocumentHtmlRefusal::TableText(at(14, 19)),
        ),
        (
            "<table><div></div></table>",
            DocumentHtmlRefusal::FosterParenting(at(8, 11)),
        ),
        ("<td>x</td>", DocumentHtmlRefusal::IgnoredTag(at(1, 3))),
        (
            "<table><caption></table><tbody>",
            DocumentHtmlRefusal::IgnoredTag(at(25, 30)),
        ),
        (
            "<p><span><div></div></span>",
            DocumentHtmlRefusal::ImpliedEnd(at(10, 13)),
        ),
        (
            "<li><span><li>",
            DocumentHtmlRefusal::ImpliedEnd(at(11, 13)),
        ),
        ("<div></p></div>", DocumentHtmlRefusal::ImpliedEnd(at(7, 8))),
        (
            "<ul><li></ul></li>",
            DocumentHtmlRefusal::ImpliedEnd(at(15, 17)),
        ),
        (
            "<table><tbody><tr><td></tr></td>",
            DocumentHtmlRefusal::ImpliedEnd(at(29, 31)),
        ),
        (
            "<table></td></table>",
            DocumentHtmlRefusal::ImpliedEnd(at(9, 11)),
        ),
        (
            "<div><table></div>",
            DocumentHtmlRefusal::ImpliedEnd(at(14, 17)),
        ),
        (
            "<span><p>x</span>",
            DocumentHtmlRefusal::ImpliedEnd(at(12, 16)),
        ),
        (
            "<div><span>x</body>",
            DocumentHtmlRefusal::ImpliedEnd(at(14, 18)),
        ),
        (
            "<table><table></table>",
            DocumentHtmlRefusal::ImpliedEnd(at(8, 13)),
        ),
    ] {
        let source = vize_l0::cstr!("<!DOCTYPE html><html><head></head><body>{body}</body></html>");
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
        assert_eq!(owner.html_structure().unwrap_err(), refusal, "{body}");
    }
}
