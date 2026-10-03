//! Original BigInt descendants retain whole refusals and existing resource gates.

use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_glyph::native_doc::{ExpressionRefusal, LineEnding, expression_document};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::{
    Lang,
    syntax::{EmbedHole, RetainedExpression},
};

use super::{assert_preserved, format, options, retained};
use crate::refusals::assert_unsupported;

fn assert_refusal_preserves<'a>(
    original: &RetainedExpression<'a>,
    source: &'a str,
    arena: &'a Allocator,
    expected: ExpressionRefusal,
) {
    let ast = original.expression().map(core::ptr::from_ref);
    let hole = original.hole();
    let view = original.source();
    let text = view.text().to_owned();
    let text_storage = view.text().as_ptr();
    assert_eq!(view.authored_root(), source);
    assert_eq!(view.authored_root().as_ptr(), source.as_ptr());
    let map = view
        .decode_map()
        .map(|map| (map.segments().as_ptr(), map.segments().to_vec()));
    let comments = || {
        original
            .comments()
            .map(|comment| {
                (
                    comment.kind(),
                    comment.decoded_span().unwrap(),
                    comment.authored_span().unwrap(),
                    comment.text().unwrap().as_ptr(),
                )
            })
            .collect::<std::vec::Vec<_>>()
    };
    let diagnostics = || {
        original
            .diagnostics()
            .map(|diagnostic| (diagnostic.message().as_ptr(), diagnostic.message().len()))
            .collect::<std::vec::Vec<_>>()
    };
    let before_comments = comments();
    let before_diagnostics = diagnostics();
    assert_eq!(
        expression_document(
            original,
            SourceRoot::new(source).unwrap().whole_block(),
            arena,
        )
        .unwrap_err(),
        expected
    );
    assert_eq!(original.expression().map(core::ptr::from_ref), ast);
    assert_eq!(original.hole(), hole);
    assert_eq!(original.source().text(), text);
    assert_eq!(original.source().text().as_ptr(), text_storage);
    assert_eq!(original.source().authored_root(), source);
    assert_eq!(original.source().authored_root().as_ptr(), source.as_ptr());
    assert_eq!(original.source().span(), view.span());
    assert_eq!(
        original
            .source()
            .decode_map()
            .map(|map| (map.segments().as_ptr(), map.segments().to_vec(),)),
        map
    );
    assert_eq!(comments(), before_comments);
    assert_eq!(diagnostics(), before_diagnostics);
}

#[test]
fn unsupported_children_and_bigint_property_keys_still_refuse_complete_documents() {
    for content in [
        "1n + f(...a)",
        "f(1n,...a)",
        "[1n,...a]",
        "{a:1n,b:()=>c}",
        "a?1n:b?.c",
        "1n,await a",
        "!(1n,++a)",
        "{1n:a}",
        "1n + /x/",
        "new A(1n)",
        "1n + this",
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let arena = Allocator::default();
            let source = std::format!("/*kept*/ {content}");
            let original = retained(
                &arena,
                &source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            assert_unsupported(&original, &source, &arena);
        }
    }
    for source in ["1n as T", "(1n as T)", "1n!", "f<T>(1n)"] {
        let arena = Allocator::default();
        let original = retained(
            &arena,
            source,
            Span::new(0, source.len() as u32),
            Lang::Ts,
            false,
        );
        assert_unsupported(&original, source, &arena);
    }
    let source = "1&#110;+f&#40;...a&#41;";
    for lang in [Lang::Js, Lang::Ts] {
        let arena = Allocator::default();
        let original = retained(
            &arena,
            source,
            Span::new(0, source.len() as u32),
            lang,
            true,
        );
        assert_eq!(original.hole(), None);
        assert_eq!(original.source().text(), "1n+f(...a)");
        assert!(original.source().decode_map().is_some());
        let Expression::BinaryExpression(binary) = original.expression().unwrap() else {
            panic!("original mapped Binary")
        };
        assert!(matches!(&binary.left, Expression::BigIntLiteral(_)));
        let Expression::CallExpression(call) = &binary.right else {
            panic!("original mapped Call")
        };
        let span = original.decoded_span(call.span).unwrap();
        assert_refusal_preserves(
            &original,
            source,
            &arena,
            ExpressionRefusal::UnsupportedNode { span },
        );
    }
}

#[test]
fn admitted_bigint_leaf_keeps_document_depth_sixteen_and_refuses_seventeen() {
    let supported = std::format!("{}1n", "!".repeat(16));
    for lang in [Lang::Js, Lang::Ts] {
        assert_eq!(
            format(&supported, lang, false, options(200, LineEnding::Lf)),
            std::format!("{}1n", "! ".repeat(16))
        );
        let source = std::format!("{}1n", "!".repeat(17));
        let arena = Allocator::default();
        let original = retained(
            &arena,
            &source,
            Span::new(0, source.len() as u32),
            lang,
            false,
        );
        assert_eq!(original.hole(), None);
        assert!(original.admitted_expression().is_some());
        let mut child = original.expression().unwrap();
        for _ in 0..17 {
            let Expression::UnaryExpression(unary) = child else {
                panic!("original unary prefix")
            };
            child = &unary.argument;
        }
        assert!(
            matches!(child, Expression::BigIntLiteral(literal) if literal.value.as_str() == "1")
        );
        let span = original.decoded_span(child.span()).unwrap();
        assert_eq!(span, Span::new(17, 19));
        assert_refusal_preserves(
            &original,
            &source,
            &arena,
            ExpressionRefusal::DepthLimit { span },
        );
    }
    assert_preserved(&supported, false);
}

#[test]
fn numeric_run_cap_counts_original_separator_and_suffix_bytes_with_unchanged_source_unit_admission()
{
    for (source, decode) in [
        (std::format!("{}n", "1".repeat(4095)), false),
        (std::format!("{}1n", "1_".repeat(2047)), false),
        (std::format!("&#49;{}n", "1".repeat(4094)), true),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let arena = Allocator::default();
            let original = retained(
                &arena,
                &source,
                Span::new(0, source.len() as u32),
                lang,
                decode,
            );
            assert_eq!(original.hole(), None);
            let Expression::BigIntLiteral(literal) = original.expression().unwrap() else {
                panic!("original bounded BigInt")
            };
            assert_eq!(literal.raw.as_ref().unwrap().as_str().len(), 4096);
            for width in [0, 1, 7, 80, 200] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(
                        format(&source, lang, decode, options(width, ending)),
                        source
                    );
                }
            }
        }
        assert_preserved(&source, decode);
    }
    for (source, hole) in [
        (
            std::format!("{}n", "1".repeat(4096)),
            EmbedHole::SafetyAdmission,
        ),
        (
            std::format!("{}11n", "1_".repeat(2047)),
            EmbedHole::SafetyAdmission,
        ),
        (
            std::format!("1n{}", " + 1n".repeat(32)),
            EmbedHole::TokenBudget,
        ),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let arena = Allocator::default();
            let original = retained(
                &arena,
                &source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            assert_eq!(original.hole(), Some(hole));
            assert!(original.admitted_expression().is_none());
            assert_refusal_preserves(
                &original,
                &source,
                &arena,
                ExpressionRefusal::Unadmitted { hole: Some(hole) },
            );
        }
    }
}

#[test]
fn malformed_bigint_syntax_and_non_ascii_gaps_preserve_original_observations() {
    for source in ["01n", "1.0n", "1e2n", "0x_n", "1_nn", "1n2"] {
        for lang in [Lang::Js, Lang::Ts] {
            let arena = Allocator::default();
            let original = retained(
                &arena,
                source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            let hole = original.hole();
            assert!(hole.is_some(), "{source}");
            assert_refusal_preserves(
                &original,
                source,
                &arena,
                ExpressionRefusal::Unadmitted { hole },
            );
        }
    }
    for (source, decode) in [
        ("\u{a0}1n", false),
        ("1n\u{a0}+2n", false),
        ("1n+\u{a0}2n", false),
        ("1n&#160;+2n", true),
        ("1n+&#160;2n", true),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let arena = Allocator::default();
            let original = retained(
                &arena,
                source,
                Span::new(0, source.len() as u32),
                lang,
                decode,
            );
            assert_eq!(original.hole(), None);
            let refusal = expression_document(
                &original,
                SourceRoot::new(source).unwrap().whole_block(),
                &arena,
            )
            .unwrap_err();
            assert!(matches!(refusal, ExpressionRefusal::InvalidGap { .. }));
            assert_refusal_preserves(&original, source, &arena, refusal);
        }
    }
}
