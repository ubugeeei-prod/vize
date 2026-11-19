use oxc_ast::ast::{BindingPattern, Expression, Statement};
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};

use super::{Embed, EmbedHole, EmbedSource, Grammar, Lang, Shape, SourceError, parse_once};
use crate::embed::prepare_attribute_value;

fn raw_embed(text: &str, shape: Shape, lang: Lang) -> Embed<'_> {
    Embed {
        grammar: Grammar { shape, lang },
        source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
    }
}

#[test]
fn handlers_retain_authored_directives_return_and_comments_without_wrapper_nodes() {
    let allocator = Allocator::default();
    let text = "'use strict'; return α; // tail";
    for lang in [Lang::Js, Lang::Ts] {
        let tree = parse_once(&allocator, raw_embed(text, Shape::HandlerBody, lang));
        assert_eq!(tree.hole(), None);
        assert!(tree.program().is_none() && tree.expression().is_none());
        assert!(tree.slot_params().is_none());
        let body = tree.handler_body().unwrap();
        assert_eq!(body.directives().len(), 1);
        assert_eq!(
            tree.decoded_span(body.directives()[0].span),
            Ok(Span::new(0, 13))
        );
        let [Statement::ReturnStatement(statement)] = body.statements() else {
            panic!("expected authored return")
        };
        assert_eq!(tree.decoded_span(statement.span), Ok(Span::new(14, 24)));
        let Expression::Identifier(identifier) = statement.argument.as_ref().unwrap() else {
            panic!("expected authored identifier")
        };
        assert_eq!(identifier.name.as_str(), "α");
        assert_eq!(tree.decoded_span(identifier.span), Ok(Span::new(21, 23)));
        let comment = tree.comments().next().unwrap();
        assert_eq!(comment.text().unwrap(), "// tail");
        assert_eq!(comment.decoded_span(), Ok(Span::new(25, 32)));
        assert_eq!(tree.diagnostics().count(), 0);
        assert_eq!(
            tree.decoded_span(oxc_span::Span::new(4, 6)),
            Err(SourceError::InvalidDecodedSpan)
        );
    }
}

#[test]
fn slot_parameters_keep_real_patterns_defaults_and_rest_without_list_wrapper() {
    let allocator = Allocator::default();
    let text = "{item}, index = 0, ...rest";
    for lang in [Lang::Js, Lang::Ts] {
        let tree = parse_once(&allocator, raw_embed(text, Shape::SlotParams, lang));
        assert_eq!(tree.hole(), None);
        assert!(tree.program().is_none() && tree.expression().is_none());
        assert!(tree.handler_body().is_none());
        let params = tree.slot_params().unwrap();
        assert_eq!(params.parameters().len(), 2);
        assert!(matches!(
            params.parameters()[0].pattern,
            BindingPattern::ObjectPattern(_)
        ));
        assert_eq!(
            tree.decoded_span(params.parameters()[0].span),
            Ok(Span::new(0, 6))
        );
        let second = &params.parameters()[1];
        assert_eq!(tree.decoded_span(second.span), Ok(Span::new(8, 17)));
        assert!(matches!(
            second.initializer.as_deref(),
            Some(Expression::NumericLiteral(_))
        ));
        let rest = params.rest().unwrap();
        assert_eq!(tree.decoded_span(rest.span), Ok(Span::new(19, 26)));
        assert_eq!(
            tree.decoded_span(rest.rest.argument.span()),
            Ok(Span::new(22, 26))
        );
        assert_eq!(tree.source().text(), text);
    }
}

#[test]
fn ts_handler_and_slot_annotations_require_ts_language_and_preserve_spans() {
    let allocator = Allocator::default();
    for (shape, text) in [
        (Shape::HandlerBody, "const value: number = 1;"),
        (Shape::SlotParams, "item: number"),
    ] {
        let js = parse_once(&allocator, raw_embed(text, shape, Lang::Js));
        assert_eq!(js.hole(), Some(EmbedHole::Syntax));
        let ts = parse_once(&allocator, raw_embed(text, shape, Lang::Ts));
        assert_eq!(ts.hole(), None);
        if shape == Shape::SlotParams {
            let parameter = &ts.slot_params().unwrap().parameters()[0];
            assert_eq!(ts.decoded_span(parameter.span), Ok(Span::new(0, 12)));
            assert_eq!(
                ts.decoded_span(parameter.type_annotation.as_ref().unwrap().span),
                Ok(Span::new(4, 12))
            );
        }
    }
}

#[test]
fn empty_bodies_and_lists_have_no_generated_authored_nodes() {
    let allocator = Allocator::default();
    let handler = parse_once(&allocator, raw_embed("", Shape::HandlerBody, Lang::Js));
    assert_eq!(handler.hole(), None);
    assert!(handler.handler_body().unwrap().directives().is_empty());
    assert!(handler.handler_body().unwrap().statements().is_empty());
    let slot = parse_once(&allocator, raw_embed("", Shape::SlotParams, Lang::Js));
    assert_eq!(slot.hole(), None);
    assert!(slot.slot_params().unwrap().parameters().is_empty());
    assert!(slot.slot_params().unwrap().rest().is_none());
}

#[test]
fn line_comments_remain_authored_without_consuming_generated_tail() {
    let allocator = Allocator::default();
    for (shape, text, comment) in [
        (Shape::HandlerBody, "return count // tail", "// tail"),
        (Shape::SlotParams, "item // tail", "// tail"),
    ] {
        let tree = parse_once(&allocator, raw_embed(text, shape, Lang::Js));
        assert_eq!(tree.hole(), None);
        let retained = tree.comments().next().unwrap();
        assert_eq!(retained.text().unwrap(), comment);
        let span = retained.decoded_span().unwrap();
        assert_eq!(span.end as usize, text.len());
    }
}

#[test]
fn once_decoded_entities_are_actual_handler_and_parameter_syntax() {
    let allocator = Allocator::default();
    for (shape, authored, range, identifier_span, authored_span) in [
        (
            Shape::HandlerBody,
            "xxreturn &fjlig;;yy",
            Span::new(2, 17),
            Span::new(7, 9),
            Span::new(9, 16),
        ),
        (
            Shape::SlotParams,
            "xx&fjlig;yy",
            Span::new(2, 9),
            Span::new(0, 2),
            Span::new(2, 9),
        ),
    ] {
        let source = prepare_attribute_value(&allocator, authored, range).unwrap();
        let tree = parse_once(
            &allocator,
            Embed {
                grammar: Grammar {
                    shape,
                    lang: Lang::Js,
                },
                source,
            },
        );
        assert_eq!(tree.hole(), None);
        let span = if shape == Shape::HandlerBody {
            let [Statement::ReturnStatement(statement)] = tree.handler_body().unwrap().statements()
            else {
                panic!("expected return")
            };
            statement.argument.as_ref().unwrap().span()
        } else {
            tree.slot_params().unwrap().parameters()[0].pattern.span()
        };
        assert_eq!(tree.decoded_span(span), Ok(identifier_span));
        assert_eq!(tree.authored_span(span), Ok(authored_span));
        assert_eq!(
            tree.authored_span(oxc_span::Span::new(span.start, span.start + 1)),
            Err(SourceError::PartialEntityBoundary)
        );
    }
}

#[test]
fn syntax_holes_keep_real_comments_and_diagnostic_metadata_with_source_labels() {
    let allocator = Allocator::default();
    for (shape, text) in [
        (Shape::HandlerBody, "return + /* keep */"),
        (Shape::SlotParams, "item = /* keep */"),
    ] {
        let tree = parse_once(&allocator, raw_embed(text, shape, Lang::Js));
        assert_eq!(tree.hole(), Some(EmbedHole::Syntax));
        assert!(tree.handler_body().is_none() && tree.slot_params().is_none());
        assert_eq!(tree.source().text(), text);
        assert_eq!(
            tree.comments().next().unwrap().text().unwrap(),
            "/* keep */"
        );
        assert!(tree.diagnostics().next().is_some());
        for diagnostic in tree.diagnostics() {
            assert!(!diagnostic.message().is_empty());
            for label in diagnostic.labels() {
                let span = label.decoded_span().unwrap();
                assert!(text.get(span.start as usize..span.end as usize).is_some());
                assert_eq!(label.authored_span(), Ok(span));
            }
        }
    }
}

#[test]
fn handler_lexical_context_rejects_top_level_await_new_target_and_module_syntax() {
    let allocator = Allocator::default();
    for lang in [Lang::Js, Lang::Ts] {
        for text in ["await count", "new.target"] {
            let tree = parse_once(&allocator, raw_embed(text, Shape::HandlerBody, lang));
            assert_eq!(tree.hole(), Some(EmbedHole::Syntax), "{text}");
            assert!(tree.diagnostics().next().is_some());
            assert!(tree.handler_body().is_none());
            assert!(
                tree.diagnostics()
                    .any(|diagnostic| diagnostic.help().is_some())
            );
        }
        for text in [
            "import x from 'm'",
            "{ export default 1; }",
            "function f(){import 'm'}",
            "/*keep*/ import 'm'",
        ] {
            let tree = parse_once(&allocator, raw_embed(text, Shape::HandlerBody, lang));
            assert_eq!(tree.hole(), Some(EmbedHole::InvalidModuleContext), "{text}");
            assert!(tree.handler_body().is_none());
            if text.starts_with("/*keep*/") {
                assert_eq!(tree.comments().next().unwrap().text().unwrap(), "/*keep*/");
                assert_eq!(tree.diagnostics().count(), 0);
            }
        }
    }
    for text in [
        "return import('m')",
        "return async()=>await count",
        "function f(){return new.target}",
    ] {
        let tree = parse_once(&allocator, raw_embed(text, Shape::HandlerBody, Lang::Js));
        assert_eq!(tree.hole(), None, "{text}");
    }
    for text in ["import x = require('m')", "export = 1"] {
        let tree = parse_once(&allocator, raw_embed(text, Shape::HandlerBody, Lang::Ts));
        assert_eq!(tree.hole(), Some(EmbedHole::InvalidModuleContext), "{text}");
    }
    let slot = parse_once(
        &allocator,
        raw_embed("x = (()=>{ import 'm' })", Shape::SlotParams, Lang::Js),
    );
    assert_eq!(slot.hole(), Some(EmbedHole::InvalidModuleContext));
    assert!(slot.slot_params().is_none());
}

#[test]
fn generated_wrapper_escape_never_yields_authored_body_or_parameters() {
    let allocator = Allocator::default();
    for (shape, text) in [
        (Shape::HandlerBody, "} ; x; ()=>{"),
        (Shape::SlotParams, "x)=>{}; (y"),
        (Shape::SlotParams, "x)=>{ return y; }; (z"),
    ] {
        let tree = parse_once(&allocator, raw_embed(text, shape, Lang::Js));
        assert_eq!(tree.hole(), Some(EmbedHole::InvalidWrappedShape));
        assert!(tree.handler_body().is_none() && tree.slot_params().is_none());
        assert_eq!(tree.source().text(), text);
    }
}

#[test]
fn admission_counts_actual_wrapper_and_keeps_composite_shapes_unsupported() {
    let allocator = Allocator::default();
    let text = ";".repeat(26);
    let program = parse_once(&allocator, raw_embed(&text, Shape::Program, Lang::Js));
    assert_eq!(program.hole(), None);
    let handler = parse_once(&allocator, raw_embed(&text, Shape::HandlerBody, Lang::Js));
    assert_eq!(handler.hole(), Some(EmbedHole::TokenBudget));
    assert_eq!(handler.diagnostics().count(), 0);
    let parameters = vize_l0::cstr!("{}x", "x,".repeat(15));
    let slot = parse_once(
        &allocator,
        raw_embed(&parameters, Shape::SlotParams, Lang::Js),
    );
    assert_eq!(slot.hole(), Some(EmbedHole::TokenBudget));
    assert_eq!(slot.diagnostics().count(), 0);
    for shape in [Shape::ForHead, Shape::FilterChain] {
        let tree = parse_once(&allocator, raw_embed("count", shape, Lang::Js));
        assert_eq!(tree.hole(), Some(EmbedHole::UnsupportedShape));
    }
}
