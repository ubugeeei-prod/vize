//! Compare the full retained product against an independently prepared ordinary parse.
use super::*;
use crate::embed::{Embed, prepare_attribute_value, syntax::parse_once};
use alloc::{format, vec::Vec};
use core::ptr;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_l0::Span;

fn compare<'a>(
    parser: &mut RetainedExpressionParser<'a>,
    source: EmbedSource<'a>,
    lang: Lang,
) -> Option<&'a Expression<'a>> {
    let normal = parse_once(
        parser.allocator,
        Embed {
            source,
            grammar: Grammar {
                lang,
                shape: Shape::Expr,
            },
        },
    )
    .into_expression()
    .unwrap();
    let actual = parser.parse(source, lang).unwrap();
    assert_eq!(actual.grammar(), normal.grammar());
    assert_eq!(actual.source_type(), normal.source_type());
    assert_eq!(actual.parser_prefix(), normal.parser_prefix());
    assert_eq!(actual.hole(), normal.hole());
    assert_eq!(actual.source().span(), source.span());
    assert!(ptr::eq(actual.source().text(), source.text()));
    assert!(ptr::eq(
        actual.source().authored_root(),
        source.authored_root()
    ));
    assert!(
        actual
            .comments()
            .map(|c| (c.kind(), c.text(), c.decoded_span(), c.authored_span()))
            .eq(normal.comments().map(|c| (
                c.kind(),
                c.text(),
                c.decoded_span(),
                c.authored_span()
            )))
    );
    assert_eq!(actual.diagnostics().count(), normal.diagnostics().count());
    for (a, b) in actual.diagnostics().zip(normal.diagnostics()) {
        assert_eq!(
            (
                a.message(),
                a.severity(),
                a.code(),
                a.help(),
                a.note(),
                a.url()
            ),
            (
                b.message(),
                b.severity(),
                b.code(),
                b.help(),
                b.note(),
                b.url()
            )
        );
        assert!(
            a.labels()
                .map(|l| (
                    l.message(),
                    l.primary(),
                    l.authored_span(),
                    l.decoded_span()
                ))
                .eq(b.labels().map(|l| (
                    l.message(),
                    l.primary(),
                    l.authored_span(),
                    l.decoded_span()
                )))
        );
    }
    match (actual.admitted_expression(), normal.admitted_expression()) {
        (None, None) => None,
        (Some(a), Some(b)) => {
            assert!(ptr::eq(a.content(), source.text()));
            assert_eq!(a.source_type(), b.source_type());
            assert_eq!(a.options(), b.options());
            assert_eq!(a.parser_content_span(), b.parser_content_span());
            assert_eq!(a.parser_container_span(), b.parser_container_span());
            assert_eq!(a.has_legacy_literals(), b.has_legacy_literals());
            let ast = a.expression();
            let original = b.expression();
            assert_eq!(format!("{ast:?}"), format!("{original:?}"));
            assert_eq!(
                actual.authored_span(ast.span()),
                normal.authored_span(original.span())
            );
            assert!(!ptr::eq(ast, original));
            Some(ast)
        }
        _ => panic!("admission mismatch"),
    }
}

#[test]
fn repeated_input_keeps_distinct_original_roots_and_authored_positions() {
    let arena = Allocator::default();
    let mut parser = RetainedExpressionParser::new(&arena);
    let source = "mark &fjlig; / fj / &fjlig; / fj";
    let mut roots = Vec::new();
    for (start, value) in [(5, "&fjlig;"), (15, "fj"), (20, "&fjlig;"), (30, "fj")] {
        let block =
            prepare_attribute_value(&arena, source, Span::new(start, start + value.len() as u32))
                .unwrap();
        assert_eq!(block.text(), "fj");
        let ast = compare(&mut parser, block, Lang::Js).unwrap();
        assert!(roots.iter().all(|prior| !ptr::eq(*prior, ast)));
        roots.push(ast);
    }
}

#[test]
fn language_comments_diagnostics_recovery_and_resource_refusals_match() {
    let arena = Allocator::default();
    let mut parser = RetainedExpressionParser::new(&arena);
    let budget = "x,".repeat(32);
    for value in [
        "name",
        "name as number",
        "/*前*/ (x) // 😀\r\n",
        "(",
        "bad +",
        "a) => (b",
        "077",
        &budget,
        "/* 💡 */ name",
    ] {
        let block = EmbedSource::authored(value, Span::new(0, value.len() as u32)).unwrap();
        for lang in [Lang::Ts, Lang::Js, Lang::Ts] {
            compare(&mut parser, block, lang);
        }
    }
}
