use alloc::vec::Vec;
use vize_l0::{Allocator, SourceRoot, String};

use super::{DocumentHtmlNode, NativeDocument, envelope};
use crate::markup::document::DocumentTokenKind;

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
