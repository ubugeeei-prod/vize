use alloc::borrow::ToOwned;
use oxc_ast::ast::{Expression, Statement};
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};
use vize_l1::embed::{
    Embed, EmbedSource, Grammar, Lang, Shape, SourceError, prepare_attribute_value,
    syntax::parse_once,
};

use super::{HandlerInput, HandlerInputErrorKind};

fn embed(text: &str, lang: Lang) -> Embed<'_> {
    Embed {
        grammar: Grammar {
            shape: Shape::HandlerBody,
            lang,
        },
        source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
    }
}

#[test]
fn the_whole_original_body_and_profile_survive_a_movable_normal_owner() {
    let allocator = Allocator::default();
    let text = "'use strict'; let α: number=1; return α; /*kept*/";
    let syntax = parse_once(&allocator, embed(text, Lang::Ts));
    assert!(syntax.hole().is_none(), "{:?}", syntax.hole());
    let stock = syntax.handler_body().unwrap();
    let statements = stock.statements().as_ptr();
    let directives = stock.directives().as_ptr();
    let comments = syntax.comments().next().unwrap().text().unwrap().as_ptr();
    let input = HandlerInput::new(syntax.into_handler_body().unwrap()).unwrap();
    assert!(core::mem::needs_drop::<HandlerInput<'_>>());
    let mut parked = alloc::vec::Vec::new();
    parked.push(input);
    parked.reserve(32);
    let input = parked.pop().unwrap();
    assert_eq!(input.syntax().grammar(), embed(text, Lang::Ts).grammar);
    assert!(input.syntax().source_type().is_typescript());
    assert_eq!(input.syntax().parser_prefix(), 6);
    assert!(core::ptr::eq(input.source().text(), text));
    assert_eq!(input.body().statements.as_ptr(), statements);
    assert_eq!(input.body().directives.as_ptr(), directives);
    assert_eq!(
        input
            .syntax()
            .comments()
            .next()
            .unwrap()
            .text()
            .unwrap()
            .as_ptr(),
        comments
    );
    let body = input.body();
    drop(input);
    assert_eq!(body.statements.len(), 2);
    assert!(matches!(body.statements[1], Statement::ReturnStatement(_)));
}

#[test]
fn exact_original_wrapper_and_decoded_entity_coordinates_have_independent_goldens() {
    let allocator = Allocator::default();
    let file = "前return α &amp;&amp; β;後";
    let source = prepare_attribute_value(&allocator, file, Span::new(3, 27)).unwrap();
    let map = source.decode_map().unwrap();
    assert_eq!(source.text(), "return α && β;");
    let syntax = parse_once(
        &allocator,
        Embed {
            grammar: embed("", Lang::Js).grammar,
            source,
        },
    );
    let input = HandlerInput::new(syntax.into_handler_body().unwrap()).unwrap();
    assert!(core::ptr::eq(input.source().authored_root(), file));
    assert!(core::ptr::eq(input.source().text(), source.text()));
    assert_eq!(
        input.source().decode_map().unwrap().segments().as_ptr(),
        map.segments().as_ptr()
    );
    let Statement::ReturnStatement(returned) = &input.body().statements[0] else {
        panic!("original return statement")
    };
    let Expression::LogicalExpression(logical) = returned.argument.as_ref().unwrap() else {
        panic!("original logical expression")
    };
    assert_eq!(logical.left.span(), oxc_span::Span::new(13, 15));
    assert_eq!(logical.right.span(), oxc_span::Span::new(19, 21));
    assert_eq!(input.decoded_span(logical.left.span()), Ok(Span::new(7, 9)));
    assert_eq!(
        input.authored_span(logical.left.span()),
        Ok(Span::new(10, 12))
    );
    assert_eq!(
        input.authored_span(logical.right.span()),
        Ok(Span::new(24, 26))
    );
    assert_eq!(input.decoded_span(returned.span), Ok(Span::new(0, 16)));
    assert_eq!(input.authored_span(returned.span), Ok(Span::new(3, 27)));
}

#[test]
fn generated_container_invalid_utf8_and_partial_entity_ranges_cannot_be_authored() {
    let allocator = Allocator::default();
    let file = "前return '&#x1f338;';後";
    let source = prepare_attribute_value(&allocator, file, Span::new(3, 22)).unwrap();
    let syntax = parse_once(
        &allocator,
        Embed {
            grammar: embed("", Lang::Js).grammar,
            source,
        },
    );
    let input = HandlerInput::new(syntax.into_handler_body().unwrap()).unwrap();
    assert_eq!(
        input.authored_span(input.body().span),
        Err(SourceError::InvalidDecodedSpan)
    );
    assert_eq!(
        input.decoded_span(oxc_span::Span::new(0, 6)),
        Err(SourceError::InvalidDecodedSpan)
    );
    assert_eq!(
        input.decoded_span(oxc_span::Span::new(15, 16)),
        Err(SourceError::InvalidDecodedSpan)
    );
    assert_eq!(
        input.decoded_span(oxc_span::Span::new(21, 22)),
        Err(SourceError::InvalidDecodedSpan)
    );
    // Complete flower expansion maps to the full authored entity.
    assert_eq!(
        input.authored_span(oxc_span::Span::new(14, 18)),
        Ok(Span::new(11, 20))
    );
    assert_eq!(
        input.authored_span(oxc_span::Span::new(18, 17)),
        Err(SourceError::InvalidDecodedSpan)
    );
    let file = "前return '&fjlig;';後";
    let source = prepare_attribute_value(&allocator, file, Span::new(3, 20)).unwrap();
    let syntax = parse_once(
        &allocator,
        Embed {
            grammar: embed("", Lang::Js).grammar,
            source,
        },
    );
    let input = HandlerInput::new(syntax.into_handler_body().unwrap()).unwrap();
    assert_eq!(input.source().text(), "return 'fj';");
    assert_eq!(
        input.decoded_span(oxc_span::Span::new(14, 15)),
        Ok(Span::new(8, 9))
    );
    assert_eq!(
        input.authored_span(oxc_span::Span::new(14, 15)),
        Err(SourceError::PartialEntityBoundary)
    );
    assert_eq!(
        input.authored_span(oxc_span::Span::new(14, 16)),
        Ok(Span::new(11, 18))
    );
}

#[test]
fn incomplete_input_returns_the_full_original_diagnostic_and_comment_owner() {
    let allocator = Allocator::default();
    let syntax = parse_once(&allocator, embed("return /x/uv; //kept", Lang::Js));
    let before: alloc::vec::Vec<_> = syntax
        .diagnostics()
        .map(|diagnostic| {
            (
                diagnostic.message().as_ptr(),
                diagnostic.message().to_owned(),
                alloc::format!("{:?}", diagnostic.code()),
                diagnostic.url().map(str::to_owned),
                diagnostic.severity(),
                diagnostic.help().map(str::to_owned),
                diagnostic.note().map(str::to_owned),
                diagnostic
                    .labels()
                    .map(|label| {
                        (
                            label.message().map(str::to_owned),
                            label.primary(),
                            label.decoded_span(),
                            label.authored_span(),
                        )
                    })
                    .collect::<alloc::vec::Vec<_>>(),
            )
        })
        .collect();
    assert!(!before.is_empty());
    let comment = syntax.comments().next().unwrap().text().unwrap().as_ptr();
    let rejected = HandlerInput::new(syntax.into_handler_body().unwrap()).unwrap_err();
    assert_eq!(rejected.kind, HandlerInputErrorKind::IncompleteSyntax);
    assert_eq!(rejected.span, Span::new(0, 20));
    let after: alloc::vec::Vec<_> = rejected
        .syntax()
        .diagnostics()
        .map(|diagnostic| {
            (
                diagnostic.message().as_ptr(),
                diagnostic.message().to_owned(),
                alloc::format!("{:?}", diagnostic.code()),
                diagnostic.url().map(str::to_owned),
                diagnostic.severity(),
                diagnostic.help().map(str::to_owned),
                diagnostic.note().map(str::to_owned),
                diagnostic
                    .labels()
                    .map(|label| {
                        (
                            label.message().map(str::to_owned),
                            label.primary(),
                            label.decoded_span(),
                            label.authored_span(),
                        )
                    })
                    .collect::<alloc::vec::Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(after, before);
    let syntax = rejected.into_syntax();
    assert!(syntax.admitted_body().is_none());
    assert_eq!(
        syntax.comments().next().unwrap().text().unwrap().as_ptr(),
        comment
    );
}

#[test]
fn empty_and_comment_only_handlers_keep_real_zero_length_content() {
    let allocator = Allocator::default();
    for text in ["", "/*kept*/", "//kept"] {
        let syntax = parse_once(&allocator, embed(text, Lang::Js));
        let input = HandlerInput::new(syntax.into_handler_body().unwrap()).unwrap();
        assert!(input.body().statements.is_empty());
        assert!(core::ptr::eq(input.source().text(), text));
        assert_eq!(
            input.decoded_span(oxc_span::Span::new(6, 6)),
            Ok(Span::new(0, 0))
        );
        assert_eq!(
            input.authored_span(oxc_span::Span::new(6, 6)),
            Ok(Span::new(0, 0))
        );
        assert!(input.syntax().diagnostics().next().is_none());
        assert_eq!(
            input.syntax().comments().count(),
            usize::from(!text.is_empty())
        );
        let retained = input.into_syntax();
        assert!(retained.admitted_body().is_some());
    }
}

#[test]
fn the_original_resource_refusal_cannot_mint_l2_handler_admission() {
    let allocator = Allocator::default();
    let text = "'use strict'; const 作者: number = 1; return 作者; /*kept*/";
    let syntax = parse_once(&allocator, embed(text, Lang::Ts));
    assert_eq!(
        syntax.hole(),
        Some(vize_l1::embed::syntax::EmbedHole::TokenBudget)
    );
    let rejected = HandlerInput::new(syntax.into_handler_body().unwrap()).unwrap_err();
    assert_eq!(rejected.kind, HandlerInputErrorKind::IncompleteSyntax);
    assert!(core::ptr::eq(rejected.syntax().source().text(), text));
    assert_eq!(rejected.syntax().parser_prefix(), 6);
    assert!(rejected.syntax().source_type().is_typescript());
    assert!(rejected.syntax().body().is_none());
    assert!(rejected.syntax().diagnostics().next().is_none());
}
