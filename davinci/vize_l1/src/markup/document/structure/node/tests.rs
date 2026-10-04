use alloc::vec::Vec;
use vize_l0::{Allocator, SourceRoot, String};

use super::{DocumentHtmlElement, DocumentHtmlNode, NativeDocument, Refusal};
use crate::markup::document::DocumentTreePolicy;

mod custody;

const CORPUS: &str = include_str!("../../../../../tests/fixtures/document/html-body-nodes.json");

fn envelope(body: &str) -> String {
    let mut source = String::from("<!DOCTYPE html><html><head></head><body>");
    source.push_str(body);
    source.push_str("</body></html>");
    source
}

fn children(element: &DocumentHtmlElement<'_, '_, '_>) -> Vec<(u8, String)> {
    element
        .child_nodes()
        .expect("genuine body child admission")
        .map(|node| match node.expect("complete original child cursor") {
            DocumentHtmlNode::Element(child) => (1, String::from(child.name())),
            DocumentHtmlNode::Text(text) => (3, text.characters().collect()),
            DocumentHtmlNode::Comment(comment) => (8, comment.characters().collect()),
        })
        .collect()
}

#[test]
fn every_original_body_node_matches_the_whole_actual_chromium_census() {
    let corpus: serde_json::Value = serde_json::from_str(CORPUS).expect("whole original sources");
    let cases = corpus.as_array().expect("original census");
    assert_eq!(cases.len(), 15);
    assert_eq!(
        cases
            .iter()
            .filter(|case| case["bodyNodeRefusal"] == true)
            .count(),
        3
    );
    for case in cases {
        let name = case["name"].as_str().expect("original case name");
        let source = case["source"].as_str().expect("whole original HTML");
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
        let tree = owner.html_structure().expect(name);
        let refusal = case["bodyNodeRefusal"] == true;
        let actual: Vec<_> = tree
            .elements()
            .filter(|element| {
                matches!(
                    element.name(),
                    "body" | "div" | "span" | "br" | "hr" | "img" | "input"
                )
            })
            .filter_map(|element| {
                if refusal && element.name() == "body" {
                    assert!(
                        matches!(element.child_nodes(), Err(Refusal::OutsideBodyText(_))),
                        "{name}"
                    );
                    None
                } else {
                    Some((String::from(element.name()), children(&element)))
                }
            })
            .collect();
        let expected = if refusal {
            serde_json::Value::Array(
                case["nodes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .skip(1)
                    .cloned()
                    .collect(),
            )
        } else {
            case["nodes"].clone()
        };
        assert_eq!(serde_json::to_value(actual).unwrap(), expected, "{name}");
        assert!(core::ptr::eq(owner.source(), source));
    }
}

#[test]
fn unsupported_comment_frames_vue_and_general_tree_policies_keep_typed_refusals() {
    for body in ["<!-->", "<!--->", "<!--a<!--b-->", "<!--a--!>"] {
        let source = envelope(body);
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
        assert!(owner.tokens().count() > 0);
        assert!(
            matches!(
                owner.html_structure(),
                Err(Refusal::Lexical(_)) | Err(Refusal::NonHtmlToken)
            ),
            "{body}"
        );
    }
    for body in ["{{ x }}", "<div v-scope=state></div>"] {
        let source = envelope(body);
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
        assert_eq!(owner.html_structure().unwrap_err(), Refusal::VueSyntax);
    }
    for (body, policy) in [
        ("<p>x</p>", DocumentTreePolicy::ImpliedEndTags),
        ("<table></table>", DocumentTreePolicy::TableContentModel),
    ] {
        let source = envelope(body);
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
        assert_eq!(
            owner.html_structure().unwrap_err(),
            Refusal::UnsupportedPolicy(policy)
        );
    }
}

#[test]
fn outer_insertion_modes_refuse_nodes_without_rejecting_existing_ancestry() {
    let source = "<!DOCTYPE html><html><head></head><body><div>x</div></body>&#13;</html>\n";
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
    let tree = owner
        .html_structure()
        .expect("existing ancestry still admitted");
    for element in tree
        .elements()
        .filter(|element| matches!(element.name(), "html" | "head"))
    {
        assert_eq!(
            element.child_nodes().unwrap_err(),
            Refusal::UnsupportedNodeParent(element.opening_span())
        );
    }
    let body = tree
        .elements()
        .find(|element| element.name() == "body")
        .unwrap();
    let Err(Refusal::OutsideBodyText(span)) = body.child_nodes() else {
        panic!("original tail refusal")
    };
    assert_eq!(span.slice(source), "&#13;");
    let div = tree
        .elements()
        .find(|element| element.name() == "div")
        .unwrap();
    assert_eq!(children(&div), [(3, String::from("x"))]);
    assert_eq!(
        body.children().next().unwrap().opening_span(),
        div.opening_span()
    );
}

#[test]
fn real_nested_callback_workload_reads_only_direct_nodes_and_original_scalar_payloads() {
    let mut body = String::from("left<div>start");
    for _ in 0..32 {
        body.push_str("<span>");
    }
    for _ in 0..512 {
        body.push_str("&NotEqualTilde;&#13;\r\n&amp;amp;");
    }
    for _ in 0..32 {
        body.push_str("</span>");
    }
    body.push_str("end</div><br>right");
    let source = envelope(&body);
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
    let tree = owner.html_structure().unwrap();
    let body = tree
        .elements()
        .find(|element| element.name() == "body")
        .unwrap();
    let div = tree
        .elements()
        .find(|element| element.name() == "div")
        .unwrap();
    let deepest = tree.elements().last().unwrap();
    // The final void follows the nested subtree; select its deepest actual span.
    assert_eq!(deepest.name(), "br");
    let inner = tree
        .elements()
        .filter(|element| element.name() == "span")
        .last()
        .unwrap();
    let before = arena.allocated_bytes();
    for _ in 0..3 {
        assert_eq!(
            children(&body),
            [
                (3, String::from("left")),
                (1, String::from("div")),
                (1, String::from("br")),
                (3, String::from("right"))
            ]
        );
        assert_eq!(
            children(&div),
            [
                (3, String::from("start")),
                (1, String::from("span")),
                (3, String::from("end"))
            ]
        );
        let DocumentHtmlNode::Text(text) = inner.child_nodes().unwrap().next().unwrap().unwrap()
        else {
            panic!("original inner text")
        };
        let expected = "≂\u{338}\r\n&amp;";
        assert_eq!(text.characters().count(), expected.chars().count() * 512);
        assert!(
            text.characters().eq(expected
                .chars()
                .cycle()
                .take(expected.chars().count() * 512))
        );
        assert!(
            text.tokens()
                .all(|token| core::ptr::eq(token.owner(), &owner))
        );
    }
    assert_eq!(arena.allocated_bytes(), before);
}
