//! Whole Call refusals and genuinely admitted independent depth controls.

use super::{assert_preserved, format, options, retained};
use crate::refusals::assert_unsupported;
use vize_glyph::native_doc::{ExpressionRefusal, expression_document};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

#[test]
fn optional_chain_spread_type_arguments_and_unsupported_call_descendants_refuse_whole_docs() {
    for content in [
        "f?.(a)",
        "obj?.f(a)",
        "(obj?.f)(a)",
        "f(...a)",
        "f(a,...b)",
        "new f(a)",
        "import('x')",
        "super(a)",
        "this(a)",
        "f(a=b)",
        "f(a?b:[...c])",
        "f([...a])",
        "f(a.#x)",
        "f(await a)",
        "(a,b)(...c)",
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
    for content in ["f<T>(a)", "f(a as T)", "f(a!)", "(f as T)(a)", "f!(a)"] {
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
fn genuine_mixed_unary_call_depth_retains_admitted_depth_sixteen_and_refuses_seventeen() {
    let supported = std::format!("{}{}a{}", "!".repeat(12), "f(".repeat(4), ")".repeat(4));
    for lang in [Lang::Js, Lang::Ts] {
        assert_eq!(
            format(&supported, lang, false, options(200)),
            std::format!("{}{}a{}", "! ".repeat(12), "f ( ".repeat(4), " )".repeat(4))
        );
        let source = std::format!("{}{}a{}", "!".repeat(13), "f(".repeat(4), ")".repeat(4));
        let allocator = Allocator::default();
        let original = retained(
            &allocator,
            &source,
            Span::new(0, source.len() as u32),
            lang,
            false,
        );
        assert_eq!(original.hole(), None);
        let root = core::ptr::from_ref(original.expression().unwrap());
        assert_eq!(
            expression_document(
                &original,
                SourceRoot::new(&source).unwrap().whole_block(),
                &allocator
            )
            .unwrap_err(),
            ExpressionRefusal::DepthLimit {
                span: Span::new(19, 20)
            }
        );
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), root);
        assert_eq!(original.source().text(), source);
        assert_eq!(original.diagnostics().count(), 0);
    }
    assert_preserved(&supported, false);
}

#[test]
fn call_syntax_holes_and_non_ascii_gaps_keep_original_observations_without_format_credit() {
    for source in ["f(a,,b)", "f(a", "f(,)", "f(...)"] {
        let allocator = Allocator::default();
        let original = retained(
            &allocator,
            source,
            Span::new(0, source.len() as u32),
            Lang::Js,
            false,
        );
        assert!(original.hole().is_some(), "{source}");
        let hole = original.hole();
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
        assert_eq!(original.source().text(), source);
        assert_eq!(original.diagnostics().count(), diagnostics);
    }
    for source in ["f\u{a0}(a)", "f(\u{a0}a)", "f(a,\u{a0}b)", "f(a\u{a0})"] {
        let allocator = Allocator::default();
        let original = retained(
            &allocator,
            source,
            Span::new(0, source.len() as u32),
            Lang::Js,
            false,
        );
        assert_eq!(original.hole(), None);
        assert!(matches!(
            expression_document(
                &original,
                SourceRoot::new(source).unwrap().whole_block(),
                &allocator
            ),
            Err(ExpressionRefusal::InvalidGap { .. })
        ));
        assert_eq!(original.source().text(), source);
    }
}
