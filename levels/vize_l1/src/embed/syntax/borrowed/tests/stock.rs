use oxc_ast::ast::Expression;
use oxc_parser::{EmbeddingGoal, EmbeddingHole, ParseOptions};
use oxc_span::{GetSpan, SourceType};
use vize_l0::Allocator;

use super::observe;

#[test]
fn stock_witness_keeps_exact_original_root_observation_and_zero_arena_allocation() {
    let arena = Allocator::default();
    let content = "/*keep*/ left + right";
    let owner = observe(&arena, content, EmbeddingGoal::Expr);
    let root = owner.expression().unwrap();
    let Expression::BinaryExpression(binary) = root else {
        panic!("original binary")
    };
    let left = core::ptr::from_ref(&binary.left);
    let right = core::ptr::from_ref(&binary.right);
    let comments = owner.comments().as_ptr();
    let diagnostics = owner.diagnostics().as_ptr();
    let before = arena.allocated_bytes();
    for _ in 0..4 {
        let admitted = owner.admitted_expression().unwrap();
        assert!(core::ptr::eq(admitted.original(), &owner));
        assert!(core::ptr::eq(admitted.expression(), root));
        assert!(core::ptr::eq(admitted.content(), content));
        assert_eq!(admitted.source_type(), SourceType::mjs());
        assert_eq!(admitted.options(), ParseOptions::default());
        assert_eq!(admitted.parser_content_span(), oxc_span::Span::new(2, 23));
        assert_eq!(
            admitted.parser_container_span(),
            oxc_span::Span::new(11, 23)
        );
        assert_eq!(admitted.comments().len(), 1);
        assert_eq!(admitted.comments().as_ptr(), comments);
        assert_eq!(admitted.diagnostics().as_ptr(), diagnostics);
        assert!(admitted.diagnostics().is_empty());
        assert!(!admitted.has_legacy_literals());
        let Expression::BinaryExpression(same) = admitted.expression() else {
            panic!("unchanged binary")
        };
        assert_eq!(core::ptr::from_ref(&same.left), left);
        assert_eq!(core::ptr::from_ref(&same.right), right);
        assert_eq!(admitted.expression().span(), root.span());
    }
    assert_eq!(arena.allocated_bytes(), before);
    assert!(core::ptr::eq(owner.expression().unwrap(), root));
    assert_eq!(owner.comments().as_ptr(), comments);
}

#[test]
fn stock_wrong_goals_and_original_recovery_never_supply_a_borrowed_root() {
    let arena = Allocator::default();
    for (content, goal) in [
        ("return value; //keep", EmbeddingGoal::HandlerBody),
        ("item,index=0", EmbeddingGoal::Parameters),
    ] {
        let owner = observe(&arena, content, goal);
        assert_eq!(owner.hole(), None);
        let before = arena.allocated_bytes();
        assert!(owner.admitted_expression().is_none());
        assert!(core::ptr::eq(owner.content(), content));
        assert_eq!(arena.allocated_bytes(), before);
    }
    for (content, expected) in [
        ("value + /*keep*/", EmbeddingHole::Syntax),
        (
            "value); other; (value",
            EmbeddingHole::InvalidExpressionShape,
        ),
    ] {
        let owner = observe(&arena, content, EmbeddingGoal::Expr);
        assert_eq!(owner.hole(), Some(expected));
        let diagnostics = owner.diagnostics().as_ptr();
        let comments = owner.comments().as_ptr();
        let count = owner.diagnostics().len();
        let before = arena.allocated_bytes();
        assert!(owner.admitted_expression().is_none());
        assert_eq!(owner.hole(), Some(expected));
        assert_eq!(owner.diagnostics().as_ptr(), diagnostics);
        assert_eq!(owner.diagnostics().len(), count);
        assert_eq!(owner.comments().as_ptr(), comments);
        assert!(core::ptr::eq(owner.content(), content));
        assert_eq!(arena.allocated_bytes(), before);
        if expected == EmbeddingHole::Syntax {
            assert!(count > 0);
            assert_eq!(owner.comments().len(), 1);
        }
    }
}

#[test]
fn original_lexer_fact_survives_borrow_then_unchanged_consuming_handoff() {
    let arena = Allocator::default();
    for (content, legacy) in [
        ("010", true),
        ("'\\1'", true),
        ("0o10", false),
        ("'\\0'", false),
    ] {
        let owner = observe(&arena, content, EmbeddingGoal::Expr);
        let admitted = owner.admitted_expression().unwrap();
        assert_eq!(admitted.has_legacy_literals(), legacy, "{content}");
        assert!(core::ptr::eq(admitted.original(), &owner));
        let before = arena.allocated_bytes();
        assert_eq!(
            owner.admitted_expression().unwrap().has_legacy_literals(),
            legacy
        );
        assert_eq!(arena.allocated_bytes(), before);
        let retained = owner.into_expression().unwrap();
        assert_eq!(retained.admitted().unwrap().has_legacy_literals(), legacy);
        assert!(core::ptr::eq(retained.content(), content));
    }
}
