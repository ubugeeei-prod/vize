use alloc::borrow::ToOwned;
use oxc_ast::ast::{Expression, Statement};
use oxc_span::{GetSpan, SourceType};
use vize_l0::{Allocator, Span};

use super::super::{Embed, EmbedHole, EmbedSource, Grammar, Lang, Shape, parse_once};
use crate::embed::prepare_attribute_value;

fn embed(text: &str, shape: Shape, lang: Lang) -> Embed<'_> {
    Embed {
        grammar: Grammar { shape, lang },
        source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
    }
}

#[test]
fn original_statement_and_directive_children_survive_consumption_and_owner_drop() {
    let arena = Allocator::default();
    let text = "'use strict'; const α = 1; return α; /*kept*/";
    let syntax = parse_once(&arena, embed(text, Shape::HandlerBody, Lang::Js));
    let original = syntax.handler_body().unwrap();
    let statements = original.statements().as_ptr();
    let directives = original.directives().as_ptr();
    let Statement::ReturnStatement(returned) = &original.statements()[1] else {
        panic!("actual return statement")
    };
    let descendant = core::ptr::from_ref(returned.argument.as_ref().unwrap());
    let comments = syntax.embedding.as_ref().unwrap().comments().as_ptr();
    let retained = syntax.into_handler_body().unwrap();
    assert!(core::mem::needs_drop::<super::RetainedHandlerBody<'_>>());
    assert_eq!(retained.hole(), None);
    let body = retained.body().unwrap();
    assert_eq!(body.statements.as_ptr(), statements);
    assert_eq!(body.directives.as_ptr(), directives);
    assert_eq!(
        retained.observation.as_ref().unwrap().comments().as_ptr(),
        comments
    );
    let Statement::ReturnStatement(returned) = &body.statements[1] else {
        panic!("original return statement")
    };
    assert_eq!(
        core::ptr::from_ref(returned.argument.as_ref().unwrap()),
        descendant
    );
    assert_eq!(
        retained.comments().next().unwrap().text().unwrap(),
        "/*kept*/"
    );
    assert!(retained.authored_span(body.span).is_err());
    assert_eq!(
        retained.authored_span(body.directives[0].span),
        Ok(Span::new(0, 13))
    );
    drop(retained);
    assert_eq!(body.statements.len(), 2);
    let Statement::ReturnStatement(returned) = &body.statements[1] else {
        panic!("arena body outlives owner")
    };
    assert!(matches!(returned.argument, Some(Expression::Identifier(_))));
}

#[test]
fn admission_keeps_the_original_body_content_profile_and_parser_window() {
    let arena = Allocator::default();
    for lang in [Lang::Js, Lang::Ts] {
        let text = "return count // tail";
        let syntax = parse_once(&arena, embed(text, Shape::HandlerBody, lang));
        let statements = syntax.handler_body().unwrap().statements().as_ptr();
        let retained = syntax.into_handler_body().unwrap();
        let admitted = retained.admitted_body().unwrap();
        assert_eq!(admitted.body().statements.as_ptr(), statements);
        assert!(core::ptr::eq(admitted.content(), text));
        assert!(core::ptr::eq(retained.source().text(), admitted.content()));
        assert_eq!(admitted.parser_content_span(), oxc_span::Span::new(6, 26));
        assert_eq!(retained.parser_prefix(), 6);
        assert_eq!(admitted.source_type(), retained.source_type());
        assert!(admitted.source_type().is_module());
        assert_eq!(admitted.source_type().is_typescript(), lang == Lang::Ts);
        assert_eq!(
            retained.grammar(),
            Grammar {
                shape: Shape::HandlerBody,
                lang
            }
        );
        assert_eq!(retained.source().span(), Span::new(0, text.len() as u32));
    }
}

#[test]
fn once_decoded_unicode_entity_maps_keep_original_body_and_comments() {
    let arena = Allocator::default();
    let file = "前/*kept*/ return α &amp;&amp; β;後";
    let source =
        prepare_attribute_value(&arena, file, Span::new(3, file.len() as u32 - 3)).unwrap();
    let map = source.decode_map().unwrap();
    let syntax = parse_once(
        &arena,
        Embed {
            grammar: Grammar {
                shape: Shape::HandlerBody,
                lang: Lang::Js,
            },
            source,
        },
    );
    let comments = syntax.comments().next().unwrap().text().unwrap().as_ptr();
    let retained = syntax.into_handler_body().unwrap();
    assert!(core::ptr::eq(retained.source().text(), source.text()));
    assert!(core::ptr::eq(
        retained.source().decode_map().unwrap().segments().as_ptr(),
        map.segments().as_ptr()
    ));
    assert_eq!(
        retained.comments().next().unwrap().text().unwrap().as_ptr(),
        comments
    );
    assert_eq!(
        retained.comments().next().unwrap().authored_span(),
        Ok(Span::new(3, 11))
    );
    let Statement::ReturnStatement(returned) = &retained.body().unwrap().statements[0] else {
        panic!("return")
    };
    let Expression::LogicalExpression(logical) = returned.argument.as_ref().unwrap() else {
        panic!("logical")
    };
    assert_eq!(
        retained.authored_span(logical.right.span()),
        Ok(Span::new(
            file.find('β').unwrap() as u32,
            file.find('β').unwrap() as u32 + 2
        ))
    );
}

#[test]
fn ts_annotations_remain_original_and_js_diagnostics_are_preserved() {
    let arena = Allocator::default();
    let text = "const count: number = 1; return count;";
    let syntax = parse_once(&arena, embed(text, Shape::HandlerBody, Lang::Ts));
    let original = syntax.handler_body().unwrap().statements().as_ptr();
    let retained = syntax.into_handler_body().unwrap();
    assert_eq!(retained.body().unwrap().statements.as_ptr(), original);
    assert_eq!(retained.source_type(), SourceType::ts().with_module(true));
    let syntax = parse_once(&arena, embed(text, Shape::HandlerBody, Lang::Js));
    let count = syntax.diagnostics().count();
    assert!(count > 0);
    let retained = syntax.into_handler_body().unwrap();
    assert_eq!(retained.hole(), Some(EmbedHole::Syntax));
    assert_eq!(retained.diagnostics().count(), count);
    assert!(retained.body().is_none() && retained.admitted_body().is_none());
}

#[test]
fn complete_original_syntax_diagnostics_and_comments_survive_local_failure() {
    let arena = Allocator::default();
    let syntax = parse_once(
        &arena,
        embed("return /x/uv; // kept", Shape::HandlerBody, Lang::Js),
    );
    let diagnostics: alloc::vec::Vec<_> = syntax
        .diagnostics()
        .map(|d| {
            (
                d.message().as_ptr(),
                d.message().to_owned(),
                alloc::format!("{:?}", d.code()),
                d.url().map(str::to_owned),
                d.severity(),
                d.help().map(str::to_owned),
                d.note().map(str::to_owned),
                d.labels()
                    .map(|l| {
                        (
                            l.message().map(str::to_owned),
                            l.primary(),
                            l.decoded_span(),
                            l.authored_span(),
                        )
                    })
                    .collect::<alloc::vec::Vec<_>>(),
            )
        })
        .collect();
    assert!(!diagnostics.is_empty());
    let comments = syntax.comments().next().unwrap().text().unwrap().as_ptr();
    let retained = syntax.into_handler_body().unwrap();
    let after: alloc::vec::Vec<_> = retained
        .diagnostics()
        .map(|d| {
            (
                d.message().as_ptr(),
                d.message().to_owned(),
                alloc::format!("{:?}", d.code()),
                d.url().map(str::to_owned),
                d.severity(),
                d.help().map(str::to_owned),
                d.note().map(str::to_owned),
                d.labels()
                    .map(|l| {
                        (
                            l.message().map(str::to_owned),
                            l.primary(),
                            l.decoded_span(),
                            l.authored_span(),
                        )
                    })
                    .collect::<alloc::vec::Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(after, diagnostics);
    assert_eq!(
        retained.comments().next().unwrap().text().unwrap().as_ptr(),
        comments
    );
    assert!(retained.body().is_none() && retained.admitted_body().is_none());
}

#[test]
fn invalid_context_wrapper_and_local_budget_holes_never_grant_admission() {
    let arena = Allocator::default();
    for text in [
        "import count from 'pkg';",
        "return await count;",
        "}; count; ()=>{",
    ] {
        let syntax = parse_once(&arena, embed(text, Shape::HandlerBody, Lang::Js));
        let expected = syntax.hole();
        assert!(expected.is_some());
        let retained = syntax.into_handler_body().unwrap();
        assert_eq!(retained.hole(), expected);
        assert!(retained.body().is_none() && retained.admitted_body().is_none());
        assert_eq!(retained.source().text(), text);
    }
    let text = ";".repeat(32);
    let before = arena.allocated_bytes();
    let retained = parse_once(&arena, embed(&text, Shape::HandlerBody, Lang::Js))
        .into_handler_body()
        .unwrap();
    assert_eq!(retained.hole(), Some(EmbedHole::TokenBudget));
    assert!(retained.body().is_none() && retained.admitted_body().is_none());
    assert_eq!(retained.diagnostics().count(), 0);
    assert_eq!(arena.allocated_bytes(), before);
}

#[test]
fn empty_and_comment_only_owners_preserve_their_actual_result() {
    let arena = Allocator::default();
    for text in ["", "/*nothing*/", "//tail"] {
        let retained = parse_once(&arena, embed(text, Shape::HandlerBody, Lang::Js))
            .into_handler_body()
            .unwrap();
        assert_eq!(retained.hole(), None);
        assert!(retained.body().unwrap().directives.is_empty());
        assert!(retained.body().unwrap().statements.is_empty());
        assert_eq!(retained.comments().count(), usize::from(!text.is_empty()));
    }
}

#[test]
fn wrong_shapes_return_the_same_original_program_expression_and_parameters() {
    let arena = Allocator::default();
    for (text, shape) in [
        ("const count = 1; /*keep*/", Shape::Program),
        ("count + 1 /*keep*/", Shape::Expr),
        ("count = 1 /*keep*/", Shape::SlotParams),
    ] {
        let syntax = parse_once(&arena, embed(text, shape, Lang::Ts));
        let program = syntax.program().map(|p| p.body.as_ptr());
        let expression = syntax.expression().map(core::ptr::from_ref);
        let parameters = syntax.slot_params().map(|p| p.parameters().as_ptr());
        let comments = syntax.comments().next().unwrap().text().unwrap().as_ptr();
        let syntax = syntax.into_handler_body().unwrap_err();
        assert_eq!(syntax.program().map(|p| p.body.as_ptr()), program);
        assert_eq!(syntax.expression().map(core::ptr::from_ref), expression);
        assert_eq!(
            syntax.slot_params().map(|p| p.parameters().as_ptr()),
            parameters
        );
        assert_eq!(
            syntax.comments().next().unwrap().text().unwrap().as_ptr(),
            comments
        );
        assert_eq!(syntax.grammar().shape, shape);
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.source().text(), text);
    }
}
