use alloc::vec::Vec;
use vize_l0::{Allocator, SourceRoot, String};

use super::{DocumentHtmlElement, DocumentHtmlNode, NativeDocument, Refusal};
use crate::markup::document::{DocumentTokenKind, DocumentTreePolicy};

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
fn text_and_comment_geometry_keeps_actual_original_callback_addresses() {
    let source =
        envelope("前🙂<div data-x='&amp;'>雪&amp;&#x1f642;<!--&amp;-->&NotEqualTilde;</div>終");
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
    let tree = owner.html_structure().unwrap();
    let div = tree
        .elements()
        .find(|element| element.name() == "div")
        .unwrap();
    let before = arena.allocated_bytes();
    let mut nodes = div.child_nodes().unwrap();
    let DocumentHtmlNode::Text(text) = nodes.next().unwrap().unwrap() else {
        panic!("original text")
    };
    assert!(core::ptr::eq(text.owner(), &owner));
    assert_eq!(text.parent().opening_span(), div.opening_span());
    assert_eq!(text.authored(), "雪&amp;&#x1f642;");
    assert_eq!(text.span().slice(&source), text.authored());
    assert_eq!(text.span().start as usize, source.find('雪').unwrap());
    assert!(core::ptr::eq(
        text.authored().as_ptr(),
        source[text.span().start as usize..].as_ptr()
    ));
    assert_eq!(text.characters().collect::<String>(), "雪&🙂");
    let tokens: Vec<_> = text.tokens().collect();
    assert_eq!(tokens.len(), 3);
    for (offset, token) in tokens.iter().enumerate() {
        let original = owner.events.get(text.parts.start + offset).unwrap();
        assert!(core::ptr::eq(token.owner(), &owner));
        assert!(core::ptr::eq(token.event, original));
        assert_eq!(token.span(), original.span);
        assert_eq!(token.decoded_entity(), original.decoded_entity());
    }
    assert_eq!(tokens[0].kind(), DocumentTokenKind::Text);
    assert_eq!(tokens[1].kind(), DocumentTokenKind::TextEntity);
    let DocumentHtmlNode::Comment(comment) = nodes.next().unwrap().unwrap() else {
        panic!("original comment")
    };
    assert!(core::ptr::eq(comment.owner(), &owner));
    assert_eq!(comment.parent().opening_span(), div.opening_span());
    assert_eq!(comment.span().slice(&source), "<!--&amp;-->");
    assert_eq!(comment.content_span().slice(&source), "&amp;");
    assert_eq!(comment.authored_content(), "&amp;");
    assert_eq!(comment.characters().collect::<String>(), "&amp;");
    assert!(core::ptr::eq(comment.token().event, comment.event));
    assert_eq!(comment.token().kind(), DocumentTokenKind::Comment);
    assert_eq!(comment.token().decoded_entity(), None);
    let DocumentHtmlNode::Text(last) = nodes.next().unwrap().unwrap() else {
        panic!("second original text")
    };
    assert_eq!(last.characters().collect::<String>(), "≂\u{338}");
    assert!(nodes.next().is_none());
    assert!(nodes.next().is_none());
    assert_eq!(arena.allocated_bytes(), before);
}

#[test]
fn moved_owners_and_structures_keep_original_parent_and_normal_drop_custody() {
    let source = envelope("<div>雪&amp;<!--keep--><span>x</span>終</div>");
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
    let moved = owner;
    let tree = moved.html_structure().unwrap();
    let moved_tree = tree;
    let div = moved_tree
        .elements()
        .find(|element| element.name() == "div")
        .unwrap();
    for node in div.child_nodes().unwrap() {
        let parent = match node.unwrap() {
            DocumentHtmlNode::Element(child) => child.parent().unwrap(),
            DocumentHtmlNode::Text(text) => {
                assert!(core::ptr::eq(text.owner(), &moved));
                text.parent()
            }
            DocumentHtmlNode::Comment(comment) => {
                assert!(core::ptr::eq(comment.owner(), &moved));
                comment.parent()
            }
        };
        assert_eq!(parent.opening_span(), div.opening_span());
        assert!(core::ptr::eq(parent.owner().allocator, &arena));
    }
    drop(moved_tree);
    assert!(moved.normal_completion().is_ok());
    assert_eq!(moved.source(), source.as_str());
}

#[test]
fn same_buffer_and_equal_foreign_buffers_keep_distinct_node_authorities() {
    let mut source = String::with_capacity(256);
    source.push_str("<!DOCTYPE html><html><head></head><body>&amp;<!--same--></body></html>");
    let mut other_source = String::with_capacity(256);
    other_source.push_str(&source);
    assert_eq!(source, other_source);
    assert!(!core::ptr::eq(source.as_ptr(), other_source.as_ptr()));
    let arena = Allocator::default();
    let first = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
    let duplicate = NativeDocument::lex_in(&arena, SourceRoot::new(&source).unwrap());
    let foreign = NativeDocument::lex_in(&arena, SourceRoot::new(&other_source).unwrap());
    let first_tree = first.html_structure().unwrap();
    let duplicate_tree = duplicate.html_structure().unwrap();
    let foreign_tree = foreign.html_structure().unwrap();
    let first_body = first_tree
        .elements()
        .find(|element| element.name() == "body")
        .unwrap();
    let duplicate_body = duplicate_tree
        .elements()
        .find(|element| element.name() == "body")
        .unwrap();
    let foreign_body = foreign_tree
        .elements()
        .find(|element| element.name() == "body")
        .unwrap();
    let DocumentHtmlNode::Text(first_text) =
        first_body.child_nodes().unwrap().next().unwrap().unwrap()
    else {
        panic!()
    };
    let DocumentHtmlNode::Text(duplicate_text) = duplicate_body
        .child_nodes()
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
    else {
        panic!()
    };
    let DocumentHtmlNode::Text(foreign_text) =
        foreign_body.child_nodes().unwrap().next().unwrap().unwrap()
    else {
        panic!()
    };
    assert_eq!(first_text.span(), duplicate_text.span());
    assert_eq!(first_text.span(), foreign_text.span());
    assert!(core::ptr::eq(first_text.owner(), &first));
    assert!(core::ptr::eq(duplicate_text.owner(), &duplicate));
    assert!(core::ptr::eq(foreign_text.owner(), &foreign));
    assert!(!core::ptr::eq(first_text.owner(), duplicate_text.owner()));
    assert!(!core::ptr::eq(first_text.owner(), foreign_text.owner()));
    assert!(core::ptr::eq(
        first_text.authored(),
        duplicate_text.authored()
    ));
    assert!(!core::ptr::eq(
        first_text.authored(),
        foreign_text.authored()
    ));
    assert!(!core::ptr::eq(
        first_text.tokens().next().unwrap().event,
        duplicate_text.tokens().next().unwrap().event
    ));
    assert!(!core::ptr::eq(
        first_text.tokens().next().unwrap().event,
        foreign_text.tokens().next().unwrap().event
    ));
    assert!(first_text.characters().eq(duplicate_text.characters()));
    assert!(first_text.characters().eq(foreign_text.characters()));
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
