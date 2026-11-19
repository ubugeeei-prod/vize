use oxc_ast::ast::{Expression, Statement};
use oxc_parser::{EmbeddingGoal, EmbeddingHole, EmbeddingInput, EmbeddingInputError, ParseOptions};
use oxc_span::{GetSpan, SourceType};
use vize_l0::{Allocator, Span};

use super::{Embed, EmbedHole, EmbedSource, Grammar, Lang, Shape, parse_once};

fn observe<'a>(
    arena: &'a Allocator,
    source: &'a str,
    goal: EmbeddingGoal,
) -> oxc_parser::EmbeddingObservation<'a> {
    EmbeddingInput::prepare_in(arena.as_oxc(), source, SourceType::mjs(), goal)
        .unwrap()
        .observe()
}

#[test]
fn preparation_keeps_exact_wrappers_and_has_no_arena_promotion() {
    let arena = Allocator::default();
    for (goal, expected, prefix) in [
        (EmbeddingGoal::Expr, "(\nα\n)", 2),
        (EmbeddingGoal::HandlerBody, "()=>{\nα\n}", 6),
        (EmbeddingGoal::Parameters, "(\nα\n)=>{}", 2),
    ] {
        let before = arena.allocated_bytes();
        let input = EmbeddingInput::prepare_in(
            arena.as_oxc(),
            "α",
            SourceType::ts().with_module(true),
            goal,
        )
        .unwrap();
        assert_eq!(input.parser_source(), expected);
        assert_eq!(
            input.parser_content_span(),
            oxc_span::Span::new(prefix, prefix + 2)
        );
        drop(input);
        assert_eq!(arena.allocated_bytes(), before);
    }
    for profile in [
        SourceType::cjs(),
        SourceType::unambiguous(),
        SourceType::d_ts(),
    ] {
        assert!(matches!(
            EmbeddingInput::prepare_in(arena.as_oxc(), "item", profile, EmbeddingGoal::Parameters),
            Err(EmbeddingInputError::UnsupportedProfile)
        ));
    }
}

#[test]
fn expression_projection_preserves_content_root_children_and_comments_after_owner_drop() {
    let arena = Allocator::default();
    let content = "/*keep*/ left + right";
    let observation = observe(&arena, content, EmbeddingGoal::Expr);
    let Expression::BinaryExpression(binary) = observation.expression().unwrap() else {
        panic!("binary")
    };
    let pointer = core::ptr::from_ref(&**binary);
    let comments = observation.comments().as_ptr();
    let owner = observation.into_expression().unwrap();
    assert_eq!(owner.comments().as_ptr(), comments);
    let token = owner.admitted().unwrap();
    assert!(core::ptr::eq(token.content(), content));
    assert_eq!(token.options(), ParseOptions::default());
    assert_eq!(token.source_type(), SourceType::mjs());
    let root = token.expression();
    drop(owner);
    let Expression::BinaryExpression(binary) = root else {
        panic!("same binary")
    };
    assert_eq!(core::ptr::from_ref(&**binary), pointer);
    assert_eq!(root.span(), oxc_span::Span::new(11, 23));
}

#[test]
fn body_projection_retains_original_statement_children_and_generated_container_range() {
    let arena = Allocator::default();
    let content = "'use strict';return value;";
    let observation = observe(&arena, content, EmbeddingGoal::HandlerBody);
    let original = observation.handler_body().unwrap();
    let statements = original.statements.as_ptr();
    let directives = original.directives.as_ptr();
    let owner = observation.into_handler_body().unwrap();
    let token = owner.admitted().unwrap();
    assert_eq!(token.parser_content_span(), oxc_span::Span::new(6, 32));
    assert_eq!(token.parser_container_span(), oxc_span::Span::new(4, 34));
    let root = token.body();
    drop(owner);
    assert_eq!(root.statements.as_ptr(), statements);
    assert_eq!(root.directives.as_ptr(), directives);
    assert!(matches!(root.statements[0], Statement::ReturnStatement(_)));
}

#[test]
fn parameter_projection_keeps_formals_kind_order_rest_and_original_children() {
    let arena = Allocator::default();
    let observation = observe(&arena, "{item},index=0,...rest", EmbeddingGoal::Parameters);
    let original = observation.parameters().unwrap();
    let items = original.items.as_ptr();
    let rest = core::ptr::from_ref(original.rest.as_deref().unwrap());
    let kind = original.kind;
    let owner = observation.into_parameters().unwrap();
    let root = owner.admitted().unwrap().parameters();
    drop(owner);
    assert_eq!(root.kind, kind);
    assert_eq!(root.items.len(), 2);
    assert_eq!(root.items.as_ptr(), items);
    assert_eq!(core::ptr::from_ref(root.rest.as_deref().unwrap()), rest);
}

#[test]
fn wrong_goal_preserves_full_original_owner_and_local_holes_keep_diagnostics() {
    let arena = Allocator::default();
    let observation = observe(&arena, "return value;//keep", EmbeddingGoal::HandlerBody);
    let body = observation.handler_body().unwrap().statements.as_ptr();
    let comments = observation.comments().as_ptr();
    let original = observation.into_parameters().unwrap_err();
    assert_eq!(original.handler_body().unwrap().statements.as_ptr(), body);
    assert_eq!(original.comments().as_ptr(), comments);
    let observation = observe(&arena, "return /x/uv; //keep", EmbeddingGoal::HandlerBody);
    assert_eq!(observation.hole(), Some(EmbeddingHole::Syntax));
    assert!(!observation.panicked());
    let diagnostics = observation.diagnostics().as_ptr();
    let count = observation.diagnostics().len();
    let owner = observation.into_handler_body().unwrap();
    assert!(owner.admitted().is_none());
    assert_eq!(owner.diagnostics().as_ptr(), diagnostics);
    assert_eq!(owner.diagnostics().len(), count);
    assert!(core::mem::needs_drop::<
        oxc_parser::HandlerBodyObservation<'_>,
    >());
    drop(owner);
}

#[test]
fn wrapper_escapes_and_module_context_holes_never_mint_selected_roots() {
    let arena = Allocator::default();
    for (source, goal, expected) in [
        (
            "value); other; (value",
            EmbeddingGoal::Expr,
            EmbeddingHole::InvalidExpressionShape,
        ),
        (
            "}; other; ()=>{",
            EmbeddingGoal::HandlerBody,
            EmbeddingHole::InvalidWrappedShape,
        ),
        (
            "import value from 'pkg';",
            EmbeddingGoal::HandlerBody,
            EmbeddingHole::InvalidModuleContext,
        ),
        (
            "item=await value",
            EmbeddingGoal::Parameters,
            EmbeddingHole::InvalidParameterContext,
        ),
    ] {
        let owner = observe(&arena, source, goal);
        assert_eq!(owner.hole(), Some(expected), "{source}");
        assert!(
            owner.expression().is_none()
                && owner.handler_body().is_none()
                && owner.parameters().is_none()
        );
    }
}

#[test]
fn same_context_walk_distinguishes_runtime_bindings_references_keys_and_erased_signatures() {
    let arena = Allocator::default();
    for profile in [SourceType::mjs(), SourceType::ts().with_module(true)] {
        for source in [
            "eval",
            "arguments",
            "yield",
            "\\u0061rguments",
            "item=(()=>{return 0})",
        ] {
            let owner = EmbeddingInput::prepare_in(
                arena.as_oxc(),
                source,
                profile,
                EmbeddingGoal::Parameters,
            )
            .unwrap()
            .observe();
            let expected = (source != "item=(()=>{return 0})")
                .then_some(EmbeddingHole::InvalidParameterContext);
            assert_eq!(owner.hole(), expected, "{source} {profile:?}");
        }
        for source in [
            "{arguments:item}",
            "item=arguments",
            "item=eval",
            "item=(async()=>{await value})",
        ] {
            let owner = EmbeddingInput::prepare_in(
                arena.as_oxc(),
                source,
                profile,
                EmbeddingGoal::Parameters,
            )
            .unwrap()
            .observe();
            assert_eq!(owner.hole(), None, "{source} {profile:?}");
        }
    }
    let owner = EmbeddingInput::prepare_in(
        arena.as_oxc(),
        "item: (arguments:string)=>void",
        SourceType::ts().with_module(true),
        EmbeddingGoal::Parameters,
    )
    .unwrap()
    .observe();
    assert_eq!(owner.hole(), None);
}

#[test]
fn l1_refusal_does_not_promote_wrapper_and_entities_keep_original_admitted_identity() {
    let arena = Allocator::default();
    let large = ";".repeat(32);
    let source = EmbedSource::authored(&large, Span::new(0, large.len() as u32)).unwrap();
    let before = arena.allocated_bytes();
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
    assert_eq!(owner.hole(), Some(EmbedHole::TokenBudget));
    assert_eq!(arena.allocated_bytes(), before);
    let source =
        crate::embed::prepare_attribute_value(&arena, "xx&fjlig;yy", Span::new(2, 9)).unwrap();
    let owner = parse_once(
        &arena,
        Embed {
            source,
            grammar: Grammar {
                shape: Shape::Expr,
                lang: Lang::Js,
            },
        },
    )
    .into_expression()
    .unwrap();
    let admitted = owner.admitted_expression().unwrap();
    assert!(core::ptr::eq(admitted.content(), source.text()));
    assert_eq!(
        owner.authored_span(admitted.expression().span()),
        Ok(Span::new(2, 9))
    );
}
