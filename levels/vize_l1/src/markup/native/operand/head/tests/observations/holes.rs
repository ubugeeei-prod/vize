use super::selected;
use crate::embed::{Grammar, Lang, Shape, syntax::EmbedHole};
use crate::markup::NativeConditionKind;
use alloc::vec::Vec;
use oxc_ast::ast::CommentKind;
use oxc_diagnostics::{OxcCode, Severity};
use vize_l0::{Allocator, Span};

#[test]
fn empty_and_syntax_holes_keep_complete_original_source_comments_diagnostics_and_no_admission() {
    let arena = Allocator::default();
    for (source, raw, value_span, message, decoded_label, authored_label) in [
        (
            "前<template><p v-if=''/></template>",
            "",
            Span::new(22, 22),
            "Empty parenthesized expression",
            Span::new(0, 0),
            Span::new(22, 22),
        ),
        (
            "前<template><p v-if='/*kept*/ ready +'/></template>",
            "/*kept*/ ready +",
            Span::new(22, 38),
            "Unexpected token",
            Span::new(16, 16),
            Span::new(38, 38),
        ),
    ] {
        let old_owner = selected(&arena, source);
        let new_owner = selected(&arena, source);
        let old_element = old_owner.children().next().unwrap().into_element().unwrap();
        let new_element = new_owner.children().next().unwrap().into_element().unwrap();
        let head = new_owner
            .observe_attribute_head(new_element.attributes().next().unwrap())
            .unwrap();
        assert_eq!(head.name_block().span(), Span::new(16, 20));
        assert_eq!(head.name_block().source(), "v-if");
        assert_eq!(head.condition_kind(), Some(NativeConditionKind::If));
        let old = old_owner
            .observe_attribute_expression(old_element.attributes().next().unwrap())
            .unwrap();
        let new = head.observe_expression().unwrap();
        for operand in [&old, &new] {
            let syntax = operand.syntax();
            assert_eq!(operand.kind(), NativeConditionKind::If);
            assert_eq!(operand.name_span(), Span::new(16, 20));
            assert_eq!(operand.value_span(), value_span);
            assert_eq!(operand.raw_value(), raw);
            assert_eq!(syntax.source().span(), value_span);
            assert_eq!(syntax.source().text(), raw);
            assert!(core::ptr::eq(syntax.source().text(), operand.raw_value()));
            assert!(core::ptr::eq(syntax.source().authored_root(), source));
            assert!(syntax.source().decode_map().is_none());
            assert_eq!(syntax.parser_prefix(), 2);
            assert_eq!(
                syntax.grammar(),
                Grammar {
                    shape: Shape::Expr,
                    lang: Lang::Js
                }
            );
            assert_eq!(syntax.hole(), Some(EmbedHole::Syntax));
            assert!(syntax.expression().is_none());
            assert!(syntax.admitted_expression().is_none());
            let comments: Vec<_> = syntax
                .comments()
                .map(|comment| {
                    (
                        comment.kind(),
                        comment.decoded_span().unwrap(),
                        comment.authored_span().unwrap(),
                        comment.text().unwrap(),
                    )
                })
                .collect();
            if raw.is_empty() {
                assert_eq!(comments, []);
            } else {
                assert_eq!(
                    comments,
                    [(
                        CommentKind::SingleLineBlock,
                        Span::new(0, 8),
                        Span::new(22, 30),
                        "/*kept*/"
                    )]
                );
            }
            let diagnostics: Vec<_> = syntax.diagnostics().collect();
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].message(), message);
            assert_eq!(diagnostics[0].severity(), Severity::Error);
            assert_eq!(diagnostics[0].code(), &OxcCode::default());
            assert_eq!(diagnostics[0].help(), None);
            assert_eq!(diagnostics[0].note(), None);
            assert_eq!(diagnostics[0].url(), None);
            let labels: Vec<_> = diagnostics[0]
                .labels()
                .map(|label| {
                    (
                        label.decoded_span().unwrap(),
                        label.authored_span().unwrap(),
                        label.message(),
                        label.primary(),
                    )
                })
                .collect();
            assert_eq!(labels, [(decoded_label, authored_label, None, false)]);
        }
        assert!(
            old.admitted_for(&old_owner, old_element.attributes().next().unwrap())
                .is_none()
        );
        assert!(
            new.admitted_for(&new_owner, new_element.attributes().next().unwrap())
                .is_none()
        );
    }
}

#[test]
fn real_over31_parentheses_and_unbalanced_safety_hole_preserve_the_earliest_admission() {
    let arena = Allocator::default();
    let deep_source = concat!(
        "前<template><p v-if='",
        "((((((((",
        "((((((((",
        "((((((((",
        "((((((((",
        "x",
        "))))))))",
        "))))))))",
        "))))))))",
        "))))))))",
        "'/></template>",
    );
    let deep_raw = concat!(
        "((((((((", "((((((((", "((((((((", "((((((((", "x", "))))))))", "))))))))", "))))))))",
        "))))))))",
    );
    assert_eq!(deep_raw.len(), 65);
    // The unchanged 31-unit admission includes the generated wrapper and runs
    // before the nesting guard. Real depth 32 is therefore TokenBudget; the
    // small unbalanced control reaches SafetyAdmission without a stock parse.
    for (source, raw, value_span, hole) in [
        (
            deep_source,
            deep_raw,
            Span::new(22, 87),
            EmbedHole::TokenBudget,
        ),
        (
            "前<template><p v-if='(ready'/></template>",
            "(ready",
            Span::new(22, 28),
            EmbedHole::SafetyAdmission,
        ),
    ] {
        let old_owner = selected(&arena, source);
        let new_owner = selected(&arena, source);
        let old_element = old_owner.children().next().unwrap().into_element().unwrap();
        let new_element = new_owner.children().next().unwrap().into_element().unwrap();
        let head = new_owner
            .observe_attribute_head(new_element.attributes().next().unwrap())
            .unwrap();
        assert_eq!(head.name_block().span(), Span::new(16, 20));
        assert_eq!(head.name_block().source(), "v-if");
        assert_eq!(head.condition_kind(), Some(NativeConditionKind::If));
        let old = old_owner
            .observe_attribute_expression(old_element.attributes().next().unwrap())
            .unwrap();
        let new = head.observe_expression().unwrap();
        for operand in [&old, &new] {
            let syntax = operand.syntax();
            assert_eq!(operand.kind(), NativeConditionKind::If);
            assert_eq!(operand.name_span(), Span::new(16, 20));
            assert_eq!(operand.value_span(), value_span);
            assert_eq!(operand.raw_value(), raw);
            assert_eq!(value_span.slice(source), raw);
            assert_eq!(syntax.source().span(), value_span);
            assert_eq!(syntax.source().text(), raw);
            assert!(core::ptr::eq(syntax.source().text(), operand.raw_value()));
            assert!(core::ptr::eq(syntax.source().authored_root(), source));
            assert!(syntax.source().decode_map().is_none());
            assert_eq!(syntax.parser_prefix(), 2);
            assert_eq!(
                syntax.grammar(),
                Grammar {
                    shape: Shape::Expr,
                    lang: Lang::Js
                }
            );
            assert_eq!(syntax.hole(), Some(hole));
            assert!(syntax.expression().is_none());
            assert!(syntax.admitted_expression().is_none());
            assert_eq!(syntax.comments().count(), 0);
            assert_eq!(syntax.diagnostics().count(), 0);
        }
        assert!(
            old.admitted_for(&old_owner, old_element.attributes().next().unwrap())
                .is_none()
        );
        assert!(
            new.admitted_for(&new_owner, new_element.attributes().next().unwrap())
                .is_none()
        );
    }
}
