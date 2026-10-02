use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};

use super::super::{Embed, EmbedHole, EmbedSource, Grammar, Lang, Shape, parse_once};
use crate::embed::prepare_attribute_value;

fn embed(text: &str, shape: Shape) -> Embed<'_> {
    Embed {
        grammar: Grammar {
            shape,
            lang: Lang::Js,
        },
        source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
    }
}

#[test]
fn root_outlives_observation_owner_and_preserves_actual_subtree_addresses() {
    let allocator = Allocator::default();
    let retained_root = {
        let syntax = parse_once(&allocator, embed("left + right", Shape::Expr));
        let Expression::BinaryExpression(original) = syntax.expression().unwrap() else {
            panic!("expected binary expression")
        };
        let original_binary = core::ptr::from_ref(&**original);
        let Expression::Identifier(left) = &original.left else {
            panic!("expected left identifier")
        };
        let original_left = core::ptr::from_ref(&**left);
        let comments = syntax.embedding.as_ref().unwrap().comments().as_ptr();
        let retained = syntax.into_expression().unwrap();
        assert_eq!(retained.hole(), None);
        assert_eq!(
            retained.observation.as_ref().unwrap().comments().as_ptr(),
            comments
        );
        let root = retained.expression().unwrap();
        let Expression::BinaryExpression(binary) = root else {
            panic!("expected moved binary expression")
        };
        assert_eq!(core::ptr::from_ref(&**binary), original_binary);
        let Expression::Identifier(left) = &binary.left else {
            panic!("expected same left identifier")
        };
        assert_eq!(core::ptr::from_ref(&**left), original_left);
        drop(retained);
        root
    };
    let Expression::BinaryExpression(binary) = retained_root else {
        panic!("expected live retained root")
    };
    let Expression::Identifier(right) = &binary.right else {
        panic!("expected live retained right child")
    };
    assert_eq!(right.name.as_str(), "right");
    assert_eq!(retained_root.span(), oxc_span::Span::new(2, 14));
}

#[test]
fn handoff_removes_only_generated_parentheses_and_keeps_comments_and_coordinates() {
    let allocator = Allocator::default();
    let syntax = parse_once(&allocator, embed("/*keep*/ (count) //tail", Shape::Expr));
    let original_comments = syntax.embedding.as_ref().unwrap().comments().as_ptr();
    let Expression::ParenthesizedExpression(original) = syntax.expression().unwrap() else {
        panic!("expected authored parentheses")
    };
    let original_parentheses = core::ptr::from_ref(&**original);
    let retained = syntax.into_expression().unwrap();
    let Expression::ParenthesizedExpression(parentheses) = retained.expression().unwrap() else {
        panic!("authored parentheses must survive")
    };
    assert_eq!(core::ptr::from_ref(&**parentheses), original_parentheses);
    assert_eq!(
        retained.observation.as_ref().unwrap().comments().as_ptr(),
        original_comments
    );
    assert_eq!(retained.parser_prefix(), 2);
    assert_eq!(
        retained.decoded_span(parentheses.span),
        Ok(Span::new(9, 16))
    );
    assert_eq!(
        retained.authored_span(parentheses.span),
        Ok(Span::new(9, 16))
    );
    let mut comments = retained.comments();
    assert_eq!(comments.next().unwrap().text().unwrap(), "/*keep*/");
    assert_eq!(comments.next().unwrap().text().unwrap(), "//tail");
    assert!(comments.next().is_none());
    assert_eq!(retained.diagnostics().count(), 0);
}

#[test]
fn moved_expression_retains_actual_once_decoded_source_and_authored_projection() {
    let allocator = Allocator::default();
    let source = prepare_attribute_value(&allocator, "xx&fjlig;yy", Span::new(2, 9)).unwrap();
    let map = source.decode_map().unwrap().segments().as_ptr();
    let syntax = parse_once(
        &allocator,
        Embed {
            grammar: Grammar {
                shape: Shape::Expr,
                lang: Lang::Js,
            },
            source,
        },
    );
    let retained = syntax.into_expression().unwrap();
    let Expression::Identifier(identifier) = retained.expression().unwrap() else {
        panic!("expected actual decoded identifier")
    };
    assert_eq!(identifier.name.as_str(), "fj");
    assert_eq!(
        retained.source().decode_map().unwrap().segments().as_ptr(),
        map
    );
    assert_eq!(retained.decoded_span(identifier.span), Ok(Span::new(0, 2)));
    assert_eq!(retained.authored_span(identifier.span), Ok(Span::new(2, 9)));
}

#[test]
fn syntax_and_admission_holes_keep_owned_observations_without_recovery_ast() {
    let allocator = Allocator::default();
    let syntax = parse_once(&allocator, embed("count + /*keep*/", Shape::Expr));
    let message = syntax.diagnostics().next().unwrap().message().as_ptr();
    let retained = syntax.into_expression().unwrap();
    assert_eq!(retained.hole(), Some(EmbedHole::Syntax));
    assert!(retained.expression().is_none());
    assert_eq!(
        retained.diagnostics().next().unwrap().message().as_ptr(),
        message
    );
    assert_eq!(
        retained.comments().next().unwrap().text().unwrap(),
        "/*keep*/"
    );
    for diagnostic in retained.diagnostics() {
        for label in diagnostic.labels() {
            let span = label.decoded_span().unwrap();
            assert!(
                retained
                    .source()
                    .text()
                    .get(span.start as usize..span.end as usize)
                    .is_some()
            );
        }
    }
    let too_large = ";".repeat(32);
    let retained = parse_once(&allocator, embed(&too_large, Shape::Expr))
        .into_expression()
        .unwrap();
    assert_eq!(retained.hole(), Some(EmbedHole::TokenBudget));
    assert_eq!(retained.source().text(), too_large);
    assert!(retained.expression().is_none());
    assert_eq!(retained.comments().count(), 0);
    assert_eq!(retained.diagnostics().count(), 0);
}

#[test]
fn non_expression_artifacts_are_returned_intact() {
    let allocator = Allocator::default();
    let syntax = parse_once(
        &allocator,
        embed("/*keep*/return count", Shape::HandlerBody),
    );
    let statements = syntax.handler_body().unwrap().statements().as_ptr();
    let comments = syntax.comments().next().unwrap().text().unwrap().as_ptr();
    let original = syntax.into_expression().unwrap_err();
    assert_eq!(original.grammar().shape, Shape::HandlerBody);
    assert_eq!(original.hole(), None);
    assert_eq!(original.handler_body().unwrap().statements().len(), 1);
    assert_eq!(
        original.handler_body().unwrap().statements().as_ptr(),
        statements
    );
    assert_eq!(
        original.comments().next().unwrap().text().unwrap().as_ptr(),
        comments
    );
    assert_eq!(original.source().text(), "/*keep*/return count");

    let syntax = parse_once(&allocator, embed("let =", Shape::Program));
    let message = syntax.diagnostics().next().unwrap().message().as_ptr();
    let original = syntax.into_expression().unwrap_err();
    assert_eq!(original.hole(), Some(EmbedHole::Syntax));
    assert_eq!(
        original.diagnostics().next().unwrap().message().as_ptr(),
        message
    );
    assert_eq!(original.source().text(), "let =");
}
