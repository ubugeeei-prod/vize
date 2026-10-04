use super::{observe, selected};
use crate::embed::{Grammar, Lang, Shape, syntax::EmbedHole};
use alloc::vec::Vec;
use oxc_ast::ast::CommentKind;
use oxc_diagnostics::{OxcCode, Severity};
use vize_l0::{Allocator, Span};

#[test]
fn empty_and_syntax_holes_keep_complete_value_comments_diagnostics_and_no_binding_admission() {
    for (source, raw, value, message, decoded_label, authored_label) in [
        (
            "<template><p :id=''/></template>",
            "",
            Span::new(18, 18),
            "Empty parenthesized expression",
            Span::new(0, 0),
            Span::new(18, 18),
        ),
        (
            "<template><p :id='/*kept*/ ready +'/></template>",
            "/*kept*/ ready +",
            Span::new(18, 34),
            "Unexpected token",
            Span::new(16, 16),
            Span::new(34, 34),
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let operand = observe(&owner, 0);
        assert_eq!(operand.name_span(), Span::new(13, 16));
        assert_eq!(operand.argument_span(), Span::new(14, 16));
        assert_eq!(operand.value_span(), value);
        assert_eq!(operand.raw_value(), raw);
        let syntax = operand.syntax();
        assert_eq!(syntax.source().span(), value);
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
                    Span::new(18, 26),
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
        let element = owner.children().next().unwrap().into_element().unwrap();
        assert!(
            operand
                .admitted_for(&owner, element.attributes().next().unwrap())
                .is_none()
        );
    }
}

#[test]
fn real_over31_wrapped_units_and_unbalanced_safety_preserve_the_original_hole_and_value() {
    let deep_source = concat!(
        "<template><p :id='",
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
    // The existing wrapped 31-unit admission runs before the nesting guard.
    // The small unbalanced control reaches SafetyAdmission instead.
    for (source, raw, value, hole) in [
        (
            deep_source,
            deep_raw,
            Span::new(18, 83),
            EmbedHole::TokenBudget,
        ),
        (
            "<template><p :id='(ready'/></template>",
            "(ready",
            Span::new(18, 24),
            EmbedHole::SafetyAdmission,
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let operand = observe(&owner, 0);
        assert_eq!(operand.name_span(), Span::new(13, 16));
        assert_eq!(operand.argument_span(), Span::new(14, 16));
        assert_eq!(operand.value_span(), value);
        assert_eq!(operand.raw_value(), raw);
        assert_eq!(value.slice(source), raw);
        let syntax = operand.syntax();
        assert_eq!(syntax.source().span(), value);
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
        let element = owner.children().next().unwrap().into_element().unwrap();
        assert!(
            operand
                .admitted_for(&owner, element.attributes().next().unwrap())
                .is_none()
        );
    }
}
