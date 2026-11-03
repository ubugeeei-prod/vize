use super::*;
use crate::dialect::vue2::surface;
use crate::embed::syntax::parse_once;
use crate::embed::{Embed, Lang, Shape};
use alloc::vec::Vec;
use oxc_span::GetSpan;
use vize_l0::{Allocator, String};

fn source_facts(source: EmbedSource<'_>) -> String {
    vize_l0::cstr!(
        "{:p}|{:p}|{:?}|{:?}",
        source.authored_root().as_ptr(),
        source.text().as_ptr(),
        source.decode_map().map(|map| map.segments().as_ptr()),
        source
    )
}

fn syntax_facts<'o, 'a: 'o>(
    grammar: Grammar,
    profile: SourceType,
    source: EmbedSource<'a>,
    hole: Option<EmbedHole>,
    expression: Option<&'o Expression<'a>>,
    comments: impl Iterator<Item = CommentView<'o, 'a>>,
    diagnostics: impl Iterator<Item = DiagnosticView<'o, 'a>>,
) -> String {
    let comments: Vec<_> = comments
        .map(|comment| {
            (
                comment.kind(),
                comment.text().map(|text| (text.as_ptr(), text)),
                comment.decoded_span(),
                comment.authored_span(),
            )
        })
        .collect();
    let diagnostics: Vec<_> = diagnostics
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels()
                .map(|label| {
                    (
                        label.message(),
                        label.primary(),
                        label.decoded_span(),
                        label.authored_span(),
                    )
                })
                .collect();
            vize_l0::cstr!(
                "{:p}|{}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
                diagnostic.message().as_ptr(),
                diagnostic.message(),
                diagnostic.severity(),
                diagnostic.code(),
                diagnostic.help(),
                diagnostic.note(),
                diagnostic.url(),
                labels
            )
        })
        .collect();
    vize_l0::cstr!(
        "{grammar:?}|{profile:?}|{}|{hole:?}|{expression:?}|{comments:?}|{diagnostics:?}",
        source_facts(source)
    )
}

fn original_syntax(syntax: &NativeSyntax<'_>) -> String {
    let admission = syntax
        .borrow_expression()
        .map(|view| (view.parser_prefix(), vize_l0::cstr!("{:?}", view.options())));
    let facts = syntax_facts(
        syntax.grammar(),
        syntax.source_type(),
        syntax.source(),
        syntax.hole(),
        syntax.expression(),
        syntax.comments(),
        syntax.diagnostics(),
    );
    vize_l0::cstr!("{admission:?}|{facts}")
}

fn retained_syntax(syntax: &TextExpression<'_>) -> String {
    let admission = syntax.retained().and_then(|owner| {
        owner.admitted_expression().map(|view| {
            (
                owner.parser_prefix(),
                vize_l0::cstr!("{:?}", view.options()),
            )
        })
    });
    let facts = syntax_facts(
        syntax.grammar(),
        syntax.source_type(),
        syntax.source(),
        syntax.hole(),
        syntax.expression(),
        syntax.comments(),
        syntax.diagnostics(),
    );
    vize_l0::cstr!("{admission:?}|{facts}")
}

fn original_binding(binding: &TextBinding<'_>) -> String {
    let chain = binding.chain().map(|chain| {
        let filters: Vec<_> = chain
            .filters()
            .iter()
            .map(|filter| {
                let arguments: Vec<_> = filter.arguments().iter().map(original_syntax).collect();
                (filter.span(), source_facts(filter.name()), arguments)
            })
            .collect();
        (original_syntax(chain.base()), filters)
    });
    vize_l0::cstr!(
        "{:?}|{:p}|{}|{}|{:?}|{chain:?}|{}",
        binding.span(),
        binding.raw_content().as_ptr(),
        binding.raw_content(),
        source_facts(binding.source()),
        binding.boundaries(),
        binding.admitted().is_some()
    )
}

fn retained_binding(binding: &RetainedTextBinding<'_>) -> String {
    let chain = binding.chain().map(|chain| {
        let filters: Vec<_> = chain
            .filters()
            .iter()
            .map(|filter| {
                let arguments: Vec<_> = filter.arguments().iter().map(retained_syntax).collect();
                (filter.span(), source_facts(filter.name()), arguments)
            })
            .collect();
        (retained_syntax(chain.base()), filters)
    });
    vize_l0::cstr!(
        "{:?}|{:p}|{}|{}|{:?}|{chain:?}|{}",
        binding.span(),
        binding.raw_content().as_ptr(),
        binding.raw_content(),
        source_facts(binding.source()),
        binding.boundaries(),
        binding.admitted().is_some()
    )
}

#[test]
fn every_original_binding_argument_map_comment_diagnostic_and_boundary_survives() {
    let arena = Allocator::default();
    let source = "<div>literal{{ a&#43;1 | wrap(雪, &#x31;) | upper }}{{b+}}{{value | wrap(,)}}{{ }}{{/*keep*/a}}</div>";
    let component = surface::parse_component(&arena, source).unwrap();
    let before: Vec<_> = component.bindings().iter().map(original_binding).collect();
    assert_eq!(before.len(), 5);
    let pool = component.into_expression_pool();
    let after: Vec<_> = pool.bindings().iter().map(retained_binding).collect();
    assert_eq!(after, before);
    let [admitted, hole, partial, empty, comment] = pool.bindings() else {
        panic!("all five original callback positions");
    };
    assert_eq!(admitted.admitted().unwrap().filters().len(), 2);
    assert_eq!(hole.chain().unwrap().base().hole(), Some(EmbedHole::Syntax));
    assert!(hole.admitted().is_none());
    assert_eq!(partial.chain().unwrap().base().source().text(), "value");
    assert!(partial.admitted().is_none());
    assert_eq!(partial.boundaries().len(), 1);
    assert!(empty.chain().is_none() && empty.admitted().is_none());
    assert_eq!(empty.boundaries().len(), 1);
    assert!(comment.chain().is_none() && comment.admitted().is_none());
    assert_eq!(comment.boundaries().len(), 1);
}

#[test]
fn complete_pool_retains_converted_prefix_original_refusal_and_later_arguments() {
    let arena = Allocator::default();
    let source = EmbedSource::authored("value", Span::new(0, 5)).unwrap();
    let parse = |text: &'static str, shape| {
        parse_once(
            &arena,
            Embed {
                source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
                grammar: Grammar {
                    shape,
                    lang: Lang::Js,
                },
            },
        )
    };
    // The current sealed Vue 2 producer emits Expr only. This internal control
    // exercises the genuine returned-owner branch with a real HandlerBody parse.
    let original = parse("/*kept*/return count", Shape::HandlerBody);
    let statements = original.handler_body().unwrap().statements().as_ptr();
    let comments: Vec<_> = original
        .comments()
        .map(|comment| {
            (
                comment.text().unwrap().as_ptr(),
                comment.text().unwrap(),
                comment.decoded_span(),
                comment.authored_span(),
            )
        })
        .collect();
    let binding = TextBinding {
        span: Span::new(0, 5),
        raw_content: "value",
        source,
        boundaries: Vec::new(),
        chain: Some(FilterChain {
            base: parse("value", Shape::Expr),
            filters: Vec::from([FilterInvocation {
                span: Span::new(0, 5),
                name: source,
                arguments: Vec::from([
                    parse("first", Shape::Expr),
                    original,
                    parse("last", Shape::Expr),
                ]),
            }]),
        }),
    };
    let retained = consume(binding);
    let chain = retained.chain().unwrap();
    assert!(chain.base().expression().is_some());
    let [filter] = chain.filters() else {
        panic!("original filter position");
    };
    let [first, refused, last] = filter.arguments() else {
        panic!("all original arguments");
    };
    assert_eq!(
        (first.source().text(), last.source().text()),
        ("first", "last")
    );
    assert!(first.expression().is_some() && last.expression().is_some());
    assert!(retained.admitted().is_none() && refused.expression().is_none());
    assert!(refused.retained().is_none());
    let original = refused.original().unwrap();
    assert_eq!(original.grammar().shape, Shape::HandlerBody);
    assert_eq!(original.source().text(), "/*kept*/return count");
    assert_eq!(
        original.handler_body().unwrap().statements().as_ptr(),
        statements
    );
    let after: Vec<_> = refused
        .comments()
        .map(|comment| {
            (
                comment.text().unwrap().as_ptr(),
                comment.text().unwrap(),
                comment.decoded_span(),
                comment.authored_span(),
            )
        })
        .collect();
    assert_eq!(after, comments);
    assert_eq!(refused.diagnostics().count(), 0);
    assert_eq!(refused.source_type(), SourceType::mjs());
}

#[test]
fn original_refusal_retains_complete_malformed_program_diagnostics_and_coordinates() {
    let arena = Allocator::default();
    let syntax = parse_once(
        &arena,
        Embed {
            source: EmbedSource::authored("let =", Span::new(0, 5)).unwrap(),
            grammar: Grammar {
                shape: Shape::Program,
                lang: Lang::Js,
            },
        },
    );
    assert_eq!(syntax.hole(), Some(EmbedHole::Syntax));
    let before = original_syntax(&syntax);
    let retained = TextExpression::consume(syntax);
    assert_eq!(retained_syntax(&retained), before);
    assert!(retained.original().is_some());
    assert!(retained.retained().is_none() && retained.expression().is_none());
}

#[test]
fn moved_original_binary_payload_and_exact_spans_survive_normal_pool_drop() {
    let arena = Allocator::default();
    let source = "{{a&#43;1}}";
    let component = surface::parse_component(&arena, source).unwrap();
    let syntax = component
        .bindings()
        .first()
        .unwrap()
        .chain()
        .unwrap()
        .base();
    let Expression::BinaryExpression(binary) = syntax.expression().unwrap() else {
        panic!("original binary payload");
    };
    let payload = core::ptr::from_ref(&**binary);
    let left = match &binary.left {
        Expression::Identifier(left) => core::ptr::from_ref(&**left),
        _ => panic!("original left payload"),
    };
    let map = syntax.source().decode_map().unwrap().segments().as_ptr();
    let expected_decoded = syntax.decoded_span(binary.span);
    let expected_authored = syntax.authored_span(binary.span);
    let pool = component.into_expression_pool();
    let retained = pool.bindings().first().unwrap().chain().unwrap().base();
    let root = retained.expression().unwrap();
    assert_eq!(
        retained.source().decode_map().unwrap().segments().as_ptr(),
        map
    );
    assert_eq!(retained.decoded_span(root.span()), expected_decoded);
    assert_eq!(retained.authored_span(root.span()), expected_authored);
    drop(pool);
    let Expression::BinaryExpression(binary) = root else {
        panic!("same binary root");
    };
    assert_eq!(core::ptr::from_ref(&**binary), payload);
    let Expression::Identifier(identifier) = &binary.left else {
        panic!("same left root");
    };
    assert_eq!(core::ptr::from_ref(&**identifier), left);
    assert_eq!(identifier.name.as_str(), "a");
    assert_eq!(expected_decoded, Ok(Span::new(0, 3)));
    assert_eq!(expected_authored, Ok(Span::new(2, 9)));
}
