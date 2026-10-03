//! Whole ternary refusals retain syntax holes and genuinely admitted limits.

use vize_glyph::native_doc::{ExpressionRefusal, LineEnding, expression_document};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

use super::{assert_preserved, format, options, retained};
use crate::refusals::assert_unsupported;

#[test]
fn unsupported_test_consequent_alternate_and_ancestor_children_refuse_whole_documents() {
    for content in [
        "fn(...x)?b:c",
        "a?fn(...x):c",
        "a?b:fn(...x)",
        "obj?.a?b:c",
        "a?obj?.b:c",
        "a?b:obj?.c",
        "[...a]?b:c",
        "(a=b)?c:d",
        "a?b=c:d",
        "a?b:c=d",
        "a?await b:c",
        "a?b:c.#x",
        "f(a?b:++c)",
        "a?b():f?.(c)",
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let allocator = Allocator::default();
            let source = std::format!("/*kept*/ {content}");
            let original = retained(
                &allocator,
                &source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            assert_unsupported(&original, &source, &allocator);
        }
    }
    for content in ["a as T?b:c", "a?b as T:c", "a?b:c!", "a?f<T>(b):c"] {
        let allocator = Allocator::default();
        let source = std::format!("/*kept*/ {content}");
        let original = retained(
            &allocator,
            &source,
            Span::new(0, source.len() as u32),
            Lang::Ts,
            false,
        );
        assert_unsupported(&original, &source, &allocator);
    }
}

#[test]
fn admitted_ternary_with_fifteen_unary_test_nodes_supports_sixteen_but_next_refuses() {
    let supported = std::format!("{}a?b:c", "!".repeat(15));
    for lang in [Lang::Js, Lang::Ts] {
        assert_eq!(
            format(&supported, lang, false, options(200, LineEnding::Lf)),
            std::format!("{}a ? b : c", "! ".repeat(15))
        );
        let allocator = Allocator::default();
        let source = std::format!("{}a?b:c", "!".repeat(16));
        let original = retained(
            &allocator,
            &source,
            Span::new(0, source.len() as u32),
            lang,
            false,
        );
        assert_eq!(original.hole(), None);
        assert!(original.admitted_expression().is_some());
        let root = core::ptr::from_ref(original.expression().unwrap());
        assert_eq!(
            expression_document(
                &original,
                SourceRoot::new(&source).unwrap().whole_block(),
                &allocator
            )
            .unwrap_err(),
            ExpressionRefusal::DepthLimit {
                span: Span::new(16, 17)
            }
        );
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), root);
        assert_eq!(original.source().text(), source);
        assert_eq!(original.diagnostics().count(), 0);
    }
    assert_preserved(&supported, false);
}

#[test]
fn ternary_syntax_holes_and_non_ascii_gaps_keep_original_observations_without_credit() {
    for source in ["a?b", "a?:c", "a?b:", "a?b::c"] {
        let allocator = Allocator::default();
        let original = retained(
            &allocator,
            source,
            Span::new(0, source.len() as u32),
            Lang::Js,
            false,
        );
        let hole = original.hole();
        assert!(hole.is_some(), "{source}");
        let ast = original.expression().map(core::ptr::from_ref);
        let diagnostics = original.diagnostics().count();
        assert_eq!(
            expression_document(
                &original,
                SourceRoot::new(source).unwrap().whole_block(),
                &allocator
            )
            .unwrap_err(),
            ExpressionRefusal::Unadmitted { hole }
        );
        assert_eq!(original.expression().map(core::ptr::from_ref), ast);
        assert_eq!(original.source().text(), source);
        assert_eq!(original.diagnostics().count(), diagnostics);
    }
    for (source, decode) in [
        ("a\u{a0}?b:c", false),
        ("a?\u{a0}b:c", false),
        ("a?b\u{a0}:c", false),
        ("a?b:\u{a0}c", false),
        ("a?&#160;b:c", true),
    ] {
        let allocator = Allocator::default();
        let original = retained(
            &allocator,
            source,
            Span::new(0, source.len() as u32),
            Lang::Js,
            decode,
        );
        assert_eq!(original.hole(), None, "{source}");
        let ast = core::ptr::from_ref(original.expression().unwrap());
        let text = original.source().text().as_ptr();
        assert!(
            matches!(
                expression_document(
                    &original,
                    SourceRoot::new(source).unwrap().whole_block(),
                    &allocator
                ),
                Err(ExpressionRefusal::InvalidGap { .. })
            ),
            "{source}"
        );
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), ast);
        assert_eq!(original.source().text().as_ptr(), text);
    }
}
