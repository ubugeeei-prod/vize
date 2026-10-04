use vize_l0::{Allocator, SourceRoot};

use super::{DocumentLexicalRefusal, DocumentTokenKind as Kind, NativeDocument};
use crate::markup::entity::DecodedEntity;

fn root(source: &str) -> SourceRoot<'_> {
    SourceRoot::new(source).expect("original small document")
}

#[test]
fn multiscalar_entity_values_keep_one_original_authored_callback() {
    let source = "é<div a='&NotEqualTilde;'>&NotEqualTilde;</div>";
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(source));
    assert!(owner.normal_completion().is_ok());
    let entities = owner
        .tokens()
        .filter(|token| token.decoded_entity().is_some());
    assert_eq!(
        owner
            .tokens()
            .filter(|token| token.decoded_entity().is_some())
            .count(),
        2
    );
    for (token, kind) in entities.zip([Kind::AttributeEntity, Kind::TextEntity]) {
        assert_eq!(token.kind(), kind);
        assert_eq!(
            token.decoded_entity(),
            Some(DecodedEntity::Named("≂\u{338}"))
        );
        assert_eq!(token.source(), Some("&NotEqualTilde;"));
        assert_eq!(token.span().end - token.span().start, 15);
        assert!(core::ptr::eq(token.owner(), &owner));
        assert!(core::ptr::eq(
            token.source().expect("authored reference").as_ptr(),
            source.as_ptr().wrapping_add(token.span().start as usize)
        ));
    }
}

#[test]
fn numeric_entity_payloads_preserve_actual_unicode_corrections() {
    let source = "&#0;&#x80;&#xD800;&#1114112;&#x1F642;&#13;";
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(source));
    let expected = [
        ("&#0;", '\u{fffd}'),
        ("&#x80;", '€'),
        ("&#xD800;", '\u{fffd}'),
        ("&#1114112;", '\u{fffd}'),
        ("&#x1F642;", '🙂'),
        ("&#13;", '\r'),
    ];
    assert_eq!(owner.tokens().len(), expected.len());
    for (token, (authored, value)) in owner.tokens().zip(expected) {
        assert_eq!(token.kind(), Kind::TextEntity);
        assert_eq!(token.source(), Some(authored));
        assert_eq!(token.decoded_entity(), Some(DecodedEntity::Numeric(value)));
    }
}

#[test]
fn entity_payload_readback_never_decodes_the_following_literal_again() {
    let source = "&amp;amp;";
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(source));
    assert_eq!(owner.tokens().len(), 2);
    for (token, expected) in owner.tokens().zip([
        (Kind::TextEntity, "&amp;", Some(DecodedEntity::Named("&"))),
        (Kind::Text, "amp;", None),
    ]) {
        assert_eq!(
            (token.kind(), token.source(), token.decoded_entity()),
            (expected.0, Some(expected.1), expected.2)
        );
    }
}

#[test]
fn attribute_ambiguity_retains_the_actual_single_decode_context() {
    let source = "<div a='&copy= &copy; &notin;'>&copy= &copy;</div>";
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(source));
    assert!(owner.normal_completion().is_ok());
    let entities = owner
        .tokens()
        .filter(|token| token.decoded_entity().is_some());
    assert_eq!(
        owner
            .tokens()
            .filter(|token| token.decoded_entity().is_some())
            .count(),
        4
    );
    for (token, (kind, raw, value)) in entities.zip([
        (Kind::AttributeEntity, "&copy;", "©"),
        (Kind::AttributeEntity, "&notin;", "∉"),
        (Kind::TextEntity, "&copy", "©"),
        (Kind::TextEntity, "&copy;", "©"),
    ]) {
        assert_eq!(token.kind(), kind);
        assert_eq!(token.source(), Some(raw));
        assert_eq!(token.decoded_entity(), Some(DecodedEntity::Named(value)));
    }
}

#[test]
fn non_entity_observations_have_no_manufactured_scalar_value() {
    let source = "<!DOCTYPE html><div title='plain'>text<!--comment-->{{ count }}</div>";
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(source));
    assert!(owner.normal_completion().is_ok());
    assert!(owner.tokens().all(|token| token.decoded_entity().is_none()));
    assert_eq!(owner.source(), source);
}

#[test]
fn moving_the_original_owner_reborrows_the_same_entity_payload_and_source() {
    let source = "&ThickSpace;";
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(source));
    let moved = owner;
    for _ in 0..2 {
        let token = moved.tokens().next().expect("sole original reference");
        assert_eq!(
            token.decoded_entity(),
            Some(DecodedEntity::Named("\u{205f}\u{200a}"))
        );
        assert_eq!(token.source(), Some(source));
        assert!(core::ptr::eq(token.owner(), &moved));
        assert!(core::ptr::eq(
            token.source().expect("original reference").as_ptr(),
            source.as_ptr()
        ));
        assert_eq!(moved.tokens().len(), 1);
    }
}

#[test]
fn retained_entity_values_do_not_erase_original_recovered_syntax() {
    let source = "<div title='&amp;";
    let allocator = Allocator::default();
    let owner = NativeDocument::lex_in(&allocator, root(source));
    let token = owner
        .tokens()
        .find(|token| token.kind() == Kind::AttributeEntity)
        .expect("original complete reference");
    assert_eq!(token.source(), Some("&amp;"));
    assert_eq!(token.decoded_entity(), Some(DecodedEntity::Named("&")));
    assert_eq!(
        owner.normal_completion().unwrap_err(),
        DocumentLexicalRefusal::RecoveredSyntax
    );
    assert_eq!(owner.source(), source);
}
