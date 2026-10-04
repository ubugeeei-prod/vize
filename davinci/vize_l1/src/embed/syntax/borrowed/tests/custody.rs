use alloc::{boxed::Box, vec::Vec};
use oxc_ast::ast::Expression;
use oxc_parser::ParseOptions;
use oxc_span::{GetSpan, SourceType};
use vize_l0::{Allocator, Span};

use super::super::super::{Embed, EmbedHole, Grammar, Lang, Shape, parse_once};
use super::syntax;
use crate::embed::{DecodeSegmentKind, SourceError, source::prepare_text_value};

#[test]
fn facade_keeps_complete_nonzero_unicode_entity_maps_and_original_comment_addresses() {
    let arena = Allocator::default();
    let root = "前🙂<template>{{μ /*keep*/ + &#x32;}}</template>後";
    let start = root.find('μ').unwrap() as u32;
    let end = root.find("}}").unwrap() as u32;
    let source = prepare_text_value(&arena, root, Span::new(start, end)).unwrap();
    assert_eq!(source.text(), "μ /*keep*/ + 2");
    let owner = parse_once(
        &arena,
        Embed {
            source,
            grammar: Grammar {
                shape: Shape::Expr,
                lang: Lang::Js,
            },
        },
    );
    let original = owner.embedding.as_ref().unwrap();
    let root_pointer = core::ptr::from_ref(owner.expression().unwrap());
    let comments_pointer = original.comments().as_ptr();
    let segments_pointer = source.decode_map().unwrap().segments().as_ptr();
    let before = arena.allocated_bytes();
    let view = owner.borrow_expression().unwrap();
    assert!(core::ptr::eq(view.original(), &owner));
    assert!(core::ptr::eq(
        view.admitted_expression().original(),
        original
    ));
    assert_eq!(core::ptr::from_ref(view.expression()), root_pointer);
    assert_eq!(
        view.grammar(),
        Grammar {
            shape: Shape::Expr,
            lang: Lang::Js
        }
    );
    assert_eq!(view.source_type(), SourceType::mjs());
    assert_eq!(view.options(), ParseOptions::default());
    assert_eq!(view.parser_prefix(), 2);
    assert!(!view.has_legacy_literals());
    assert!(core::ptr::eq(view.source().authored_root(), root));
    assert!(core::ptr::eq(view.source().text(), source.text()));
    assert_eq!(view.source().span(), Span::new(start, end));
    assert_eq!(
        view.source().decode_map().unwrap().segments().as_ptr(),
        segments_pointer
    );
    let segments: Vec<_> = view
        .source()
        .decode_map()
        .unwrap()
        .segments()
        .iter()
        .map(|segment| (segment.decoded(), segment.authored(), segment.kind()))
        .collect();
    assert_eq!(
        segments,
        [
            (
                Span::new(0, 14),
                Span::new(start, start + 14),
                DecodeSegmentKind::Identity
            ),
            (
                Span::new(14, 15),
                Span::new(start + 14, end),
                DecodeSegmentKind::Entity
            )
        ]
    );
    assert_eq!(
        view.decoded_span(view.expression().span()),
        Ok(Span::new(0, 15))
    );
    assert_eq!(
        view.authored_span(view.expression().span()),
        Ok(Span::new(start, end))
    );
    let comments: Vec<_> = view.comments().collect();
    assert_eq!(comments.len(), 1);
    assert_eq!(core::ptr::from_ref(comments[0].comment), comments_pointer);
    assert_eq!(comments[0].text(), Ok("/*keep*/"));
    assert_eq!(comments[0].decoded_span(), Ok(Span::new(3, 11)));
    assert_eq!(
        comments[0].authored_span(),
        Ok(Span::new(start + 3, start + 11))
    );
    assert_eq!(view.diagnostics().count(), 0);
    assert_eq!(
        view.decoded_span(oxc_span::Span::new(0, 2)),
        Err(SourceError::InvalidDecodedSpan)
    );
    assert_eq!(arena.allocated_bytes(), before);
    let expanded = prepare_text_value(&arena, "前&fjlig;後", Span::new(3, 10)).unwrap();
    let expanded_owner = parse_once(
        &arena,
        Embed {
            source: expanded,
            grammar: Grammar {
                shape: Shape::Expr,
                lang: Lang::Js,
            },
        },
    );
    let expanded_view = expanded_owner.borrow_expression().unwrap();
    assert_eq!(
        expanded_view.authored_span(oxc_span::Span::new(2, 4)),
        Ok(Span::new(3, 10))
    );
    assert_eq!(
        expanded_view.authored_span(oxc_span::Span::new(2, 3)),
        Err(SourceError::PartialEntityBoundary)
    );
}

#[test]
fn facade_refuses_local_holes_and_other_shapes_without_moving_full_diagnostics() {
    let arena = Allocator::default();
    let owner = syntax(&arena, "value + /*keep*/", Shape::Expr);
    assert_eq!(owner.hole(), Some(EmbedHole::Syntax));
    let original = owner.embedding.as_ref().unwrap();
    let diagnostics = original.diagnostics().as_ptr();
    let comments = original.comments().as_ptr();
    let before = arena.allocated_bytes();
    assert!(owner.borrow_expression().is_none());
    assert_eq!(original.diagnostics().as_ptr(), diagnostics);
    assert_eq!(original.comments().as_ptr(), comments);
    let observed: Vec<_> = owner.diagnostics().collect();
    assert_eq!(observed.len(), original.diagnostics().len());
    assert!(!observed.is_empty());
    for (view, actual) in observed.iter().zip(original.diagnostics().iter()) {
        assert!(core::ptr::eq(view.diagnostic, actual));
        let actual_message: &str = &actual.message;
        assert_eq!(view.message(), actual_message);
        for label in view.labels() {
            let decoded = label.decoded_span().unwrap();
            assert!(
                owner
                    .source()
                    .text()
                    .get(decoded.start as usize..decoded.end as usize)
                    .is_some()
            );
        }
    }
    assert_eq!(owner.comments().next().unwrap().text(), Ok("/*keep*/"));
    assert_eq!(arena.allocated_bytes(), before);
    let large = ";".repeat(32);
    let rejected = syntax(&arena, &large, Shape::Expr);
    assert_eq!(rejected.hole(), Some(EmbedHole::TokenBudget));
    assert!(rejected.borrow_expression().is_none());
    assert_eq!(rejected.source().text(), large);
    for (text, shape, hole) in [
        ("return item;", Shape::HandlerBody, None),
        ("item,index", Shape::SlotParams, None),
        ("const item=1", Shape::Program, None),
        (
            "item",
            Shape::FilterChain,
            Some(EmbedHole::UnsupportedShape),
        ),
    ] {
        let original = syntax(&arena, text, shape);
        assert_eq!(original.hole(), hole);
        assert!(original.borrow_expression().is_none());
        assert_eq!(original.source().text(), text);
        assert_eq!(original.grammar().shape, shape);
    }
}

#[test]
fn original_owner_moves_before_borrow_and_short_reborrows_preserve_root_and_drop_custody() {
    let arena = Allocator::default();
    let owner = syntax(&arena, "/*keep*/ left + right", Shape::Expr);
    let root = core::ptr::from_ref(owner.expression().unwrap());
    let comments = owner.embedding.as_ref().unwrap().comments().as_ptr();
    let mut owners = Vec::new();
    owners.push(owner);
    let owner = Box::new(owners.pop().unwrap());
    let before = arena.allocated_bytes();
    for _ in 0..4 {
        let view = owner.borrow_expression().unwrap();
        assert!(core::ptr::eq(view.original(), &*owner));
        assert_eq!(core::ptr::from_ref(view.expression()), root);
        assert_eq!(view.admitted_expression().comments().as_ptr(), comments);
        assert_eq!(view.comments().next().unwrap().text(), Ok("/*keep*/"));
    }
    assert_eq!(arena.allocated_bytes(), before);
    assert!(core::mem::needs_drop::<super::super::super::NativeSyntax<'_>>());
    let Expression::BinaryExpression(binary) = owner.expression().unwrap() else {
        panic!("original binary")
    };
    let child = core::ptr::from_ref(&**binary);
    let retained = (*owner).into_expression().unwrap();
    let Expression::BinaryExpression(same) = retained.expression().unwrap() else {
        panic!("same original child after existing consume")
    };
    assert_eq!(core::ptr::from_ref(&**same), child);
    assert_eq!(retained.comments().next().unwrap().text(), Ok("/*keep*/"));
    assert_eq!(retained.parser_prefix(), 2);
    drop(retained);
}

#[test]
fn same_buffer_duplicate_and_equal_foreign_buffers_keep_distinct_original_authority() {
    let arena = Allocator::default();
    let text = alloc::string::String::from("left + right");
    let other_text = text.clone();
    let first = syntax(&arena, &text, Shape::Expr);
    let duplicate = syntax(&arena, &text, Shape::Expr);
    let other = syntax(&arena, &other_text, Shape::Expr);
    let first_view = first.borrow_expression().unwrap();
    let duplicate_view = duplicate.borrow_expression().unwrap();
    let other_view = other.borrow_expression().unwrap();
    assert!(core::ptr::eq(
        first_view.source().text(),
        duplicate_view.source().text()
    ));
    assert!(!core::ptr::eq(
        first_view.original(),
        duplicate_view.original()
    ));
    assert!(!core::ptr::eq(
        first_view.expression(),
        duplicate_view.expression()
    ));
    assert!(!core::ptr::eq(
        first_view.admitted_expression().original(),
        duplicate_view.admitted_expression().original()
    ));
    assert_eq!(first_view.source().text(), other_view.source().text());
    assert!(!core::ptr::eq(
        first_view.source().authored_root(),
        other_view.source().authored_root()
    ));
    assert!(!core::ptr::eq(
        first_view.expression(),
        other_view.expression()
    ));
    assert!(core::ptr::eq(first_view.original(), &first));
    assert!(core::ptr::eq(duplicate_view.original(), &duplicate));
    assert!(core::ptr::eq(other_view.original(), &other));
}
