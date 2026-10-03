//! Unsupported whole Sequence documents keep original admission and budgets.

use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_glyph::native_doc::{ExpressionRefusal, LineEnding, expression_document};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

use super::{assert_preserved, format, options, retained};
use crate::refusals::assert_unsupported;

#[test]
fn unsupported_ordered_children_refuse_the_whole_sequence_and_its_enclosing_consumers() {
    for content in [
        "a,f(...b)",
        "a,[...b]",
        "a,{x:b}",
        "a,b?.c",
        "a,f?.()",
        "a,b=c",
        "a,(b=c)",
        "a,await b",
        "await a,b",
        "a,++b",
        "a,(b=>b)",
        "a,this",
        "a,1n",
        "a,/x/",
        "a,`x`",
        "f((a,[...b]))",
        "a?[b]:(c,[...d])",
        "obj[(a,await b)]",
        "!(a,++b)",
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
    for content in ["a,b as T", "a,(b as T)", "a,f<T>(b)", "a,b!", "(a,b) as T"] {
        let arena = Allocator::default();
        let source = std::format!("/*kept*/ {content}");
        let original = retained(
            &arena,
            &source,
            Span::new(0, source.len() as u32),
            Lang::Ts,
            false,
        );
        assert_unsupported(&original, &source, &arena);
    }
}

#[test]
fn admitted_sequence_child_depth_keeps_sixteen_and_refuses_seventeen_without_budget_changes() {
    let supported = std::format!("a,{}b", "!".repeat(15));
    for lang in [Lang::Js, Lang::Ts] {
        assert_eq!(
            format(&supported, lang, false, options(200, LineEnding::Lf)),
            std::format!("a, {}b", "! ".repeat(15))
        );
        let arena = Allocator::default();
        let source = std::format!("a,{}b", "!".repeat(16));
        let original = retained(
            &arena,
            &source,
            Span::new(0, source.len() as u32),
            lang,
            false,
        );
        assert_eq!(original.hole(), None);
        assert!(original.admitted_expression().is_some());
        let ast = core::ptr::from_ref(original.expression().unwrap());
        let Expression::SequenceExpression(sequence) = original.expression().unwrap() else {
            panic!("original Sequence")
        };
        assert_eq!(sequence.expressions.len(), 2);
        let storage = sequence.expressions.as_ptr();
        let mut child = &sequence.expressions[1];
        for _ in 0..16 {
            let Expression::UnaryExpression(unary) = child else {
                panic!("original unary child")
            };
            child = &unary.argument;
        }
        assert!(
            matches!(child, Expression::Identifier(identifier) if identifier.name.as_str() == "b")
        );
        assert_eq!(
            original.decoded_span(child.span()).unwrap(),
            Span::new(18, 19)
        );
        let leaf = core::ptr::from_ref(child);
        assert_eq!(
            expression_document(
                &original,
                SourceRoot::new(&source).unwrap().whole_block(),
                &arena
            )
            .unwrap_err(),
            ExpressionRefusal::DepthLimit {
                span: Span::new(18, 19)
            }
        );
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), ast);
        assert_eq!(sequence.expressions.as_ptr(), storage);
        assert_eq!(core::ptr::from_ref(child), leaf);
        assert_eq!(original.source().text(), source);
        assert_eq!(original.diagnostics().count(), 0);
    }
    assert_preserved(&supported, false);
}

#[test]
fn malformed_sequence_holes_and_non_ascii_comma_gaps_leave_original_observations_intact() {
    for source in ["a,,b", "a,", "(a,)", "(,a)", "a,)", "(a,b"] {
        let arena = Allocator::default();
        let original = retained(
            &arena,
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
                &arena
            )
            .unwrap_err(),
            ExpressionRefusal::Unadmitted { hole }
        );
        assert_eq!(original.expression().map(core::ptr::from_ref), ast);
        assert_eq!(original.source().text(), source);
        assert_eq!(original.diagnostics().count(), diagnostics);
    }
    for (source, decode) in [
        ("a\u{a0},b", false),
        ("a,\u{a0}b", false),
        ("a,(\u{a0}b,c)", false),
        ("a&#160;&#44;b", true),
        ("a&#44;&#160;b", true),
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
            let ast = core::ptr::from_ref(original.expression().unwrap());
            let text = original.source().text().as_ptr();
            assert!(
                matches!(
                    expression_document(
                        &original,
                        SourceRoot::new(source).unwrap().whole_block(),
                        &arena
                    ),
                    Err(ExpressionRefusal::InvalidGap { .. })
                ),
                "{source}"
            );
            assert_eq!(core::ptr::from_ref(original.expression().unwrap()), ast);
            assert_eq!(original.source().text().as_ptr(), text);
        }
    }
}
