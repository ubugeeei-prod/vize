use alloc::vec::Vec;
use vize_l0::{Allocator, SourceRoot, Span, String};

use super::super::{Refusal, construct_as};
use crate::markup::document::{DocumentTokenKind, NativeDocument};
use crate::markup::{Component, Document};

const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/document/head-text.html"
));

fn envelope(head: &str, body: &str) -> String {
    let mut source = String::from("<!DOCTYPE html><html><head>");
    source.push_str(head);
    source.push_str("</head><body>");
    source.push_str(body);
    source.push_str("</body></html>");
    source
}

#[test]
fn head_text_ancestry_borrows_the_original_single_lexer_owner() {
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(ORIGINAL).expect("small source"));
    let before: Vec<_> = owner
        .tokens()
        .map(|token| (token.kind(), token.span()))
        .collect();
    let tree = owner.html_structure().expect("explicit head text");
    let elements: Vec<_> = tree.elements().collect();
    assert_eq!(
        elements.iter().map(|node| node.name()).collect::<Vec<_>>(),
        ["html", "head", "meta", "title", "style", "body", "div"]
    );
    let head = elements.get(1).expect("original head");
    assert_eq!(
        head.children().map(|node| node.name()).collect::<Vec<_>>(),
        ["meta", "title", "style"]
    );
    for (index, opening, closing) in [
        (3, "<TITLE data-label=\"&amp;\">", "</TiTlE>"),
        (4, "<STYLE>", "</sTyLe>"),
    ] {
        let node = elements.get(index).expect("original head child");
        assert!(core::ptr::eq(node.owner(), &owner));
        assert_eq!(node.opening(), opening);
        assert_eq!(
            node.closing_span().expect("authored end").slice(ORIGINAL),
            closing
        );
        assert_eq!(
            node.parent().expect("actual head").opening_span(),
            head.opening_span()
        );
        assert_eq!(node.children().count(), 0);
        let values: Vec<String> = node
            .child_nodes()
            .unwrap()
            .map(|child| match child.unwrap() {
                crate::markup::document::DocumentHtmlNode::Text(text) => {
                    text.characters().collect()
                }
                unexpected => panic!("unexpected head text child: {unexpected:?}"),
            })
            .collect();
        assert_eq!(values.len(), 1);
        let offset = ORIGINAL.find(opening).expect("authored opening");
        assert!(core::ptr::eq(
            node.opening().as_ptr(),
            ORIGINAL[offset..].as_ptr()
        ));
    }
    assert_eq!(
        owner
            .tokens()
            .map(|token| (token.kind(), token.span()))
            .collect::<Vec<_>>(),
        before
    );
    // RCDATA decodes the title reference once. RAWTEXT style keeps the same
    // bytes, including Vue-looking braces and apparent nested start tags.
    let entities: Vec<_> = owner
        .tokens()
        .filter(|token| token.kind() == DocumentTokenKind::TextEntity)
        .collect();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].source(), Some("&amp;"));
    let mut decoded = String::default();
    entities[0]
        .decoded_entity()
        .expect("title entity")
        .for_each(|ch| decoded.push(ch));
    assert_eq!(decoded, "&");
    assert!(
        owner
            .tokens()
            .any(|token| token.kind() == DocumentTokenKind::Text
                && token.source() == Some("p::before { content: \"&amp; {{ literal }} <div>\"; }"))
    );
}

#[test]
fn explicit_unslashed_head_text_preserves_both_profiles_name_rules() {
    let source = envelope(
        "<title>a &amp; <b></title><style>&amp; <b></style>",
        "<span></span>",
    );
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).expect("small source"));
    for tree in [
        construct_as::<Document>(&owner),
        construct_as::<Component>(&owner),
    ] {
        assert_eq!(
            tree.expect("explicit nesting")
                .elements()
                .map(|node| node.name())
                .collect::<Vec<_>>(),
            ["html", "head", "title", "style", "body", "span"]
        );
    }
    let source = envelope("<TITLE></TITLE>", "");
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).expect("small source"));
    assert!(construct_as::<Document>(&owner).is_ok());
    assert_eq!(
        construct_as::<Component>(&owner).unwrap_err(),
        Refusal::UnsupportedElement(Span::new(28, 33))
    );
}

#[test]
fn slashed_special_text_starts_refuse_instead_of_borrowing_html_ownership() {
    for (head, body) in [
        ("<title/>x</title>", ""),
        ("<style/>x</style>", ""),
        ("<script/>x</script>", ""),
        ("", "<title/>x</title>"),
        ("", "<style/>x</style>"),
        ("", "<script/>x</script>"),
        ("", "<textarea/>x</textarea>"),
    ] {
        let source = envelope(head, body);
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).expect("small source"));
        let raw = ["title", "style", "script", "textarea"]
            .into_iter()
            .find(|name| source.contains(name))
            .unwrap();
        let start = source.find(raw).expect("authored element") as u32;
        assert_eq!(
            owner.html_structure().unwrap_err(),
            Refusal::UnsupportedElement(Span::new(start, start + raw.len() as u32)),
            "{source}"
        );
    }
}

#[test]
fn missing_head_text_ends_and_vue_interpolation_cannot_mint_completion() {
    for head in [
        "<title>unfinished",
        "<style>unfinished",
        "<script>unfinished",
    ] {
        let source = envelope(head, "");
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).expect("small source"));
        assert!(owner.normal_completion().is_ok());
        assert_eq!(
            owner.html_structure().unwrap_err(),
            Refusal::PendingStructure
        );
    }
    let source = envelope("<title>{{ value }}</title>", "");
    let arena = Allocator::default();
    let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).expect("small source"));
    assert_eq!(owner.html_structure().unwrap_err(), Refusal::VueSyntax);
}

#[test]
fn other_head_and_foreign_features_keep_their_existing_refusal() {
    for head in ["<svg></svg>", "<b></b>", "<template></template>"] {
        let source = envelope(head, "");
        let arena = Allocator::default();
        let owner = NativeDocument::lex_in(&arena, SourceRoot::new(&source).expect("small source"));
        assert!(
            matches!(owner.html_structure(), Err(Refusal::UnsupportedElement(_))),
            "{source}"
        );
    }
}
