//! Whole Array refusals keep original sparse observations and fixed budgets.

use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_span::GetSpan;
use vize_glyph::native_doc::{ExpressionRefusal, LineEnding, expression_document};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

use super::{assert_preserved, format, options, retained};
use crate::refusals::assert_unsupported;

#[test]
fn spreads_optional_and_unsupported_array_elements_refuse_every_enclosing_document() {
    for content in [
        "[...a]",
        "[a,...b]",
        "[, ...a]",
        "[a?.b]",
        "[(a?.b)]",
        "[f?.()]",
        "[a=b]",
        "[a?b:++c]",
        "[a,b=>b]",
        "[this]",
        "[new A]",
        "[import('x')]",
        "[[a],obj.#x]",
        "[a,await b]",
        "f([a,...b])",
        "obj[[a,await b]]",
        "[a,++b].key",
        "![a,...b]",
        "a?[b]:[...c]",
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
    for content in ["[a as T]", "[a!]", "[f<T>(a)]", "([a] as T)"] {
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
fn admitted_dense_and_sparse_child_depth_keeps_sixteen_and_refuses_seventeen() {
    for (array_source, printed, is_elision) in [("[a]", "[ a ]", false), ("[,]", "[ , ]", true)] {
        let supported = std::format!("{}{array_source}", "!".repeat(15));
        for lang in [Lang::Js, Lang::Ts] {
            assert_eq!(
                format(&supported, lang, false, options(200, LineEnding::Lf)),
                std::format!("{}{printed}", "! ".repeat(15))
            );
            let arena = Allocator::default();
            let source = std::format!("{}{array_source}", "!".repeat(16));
            let original = retained(
                &arena,
                &source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            assert_eq!(original.hole(), None);
            assert!(original.admitted_expression().is_some());
            let root = core::ptr::from_ref(original.expression().unwrap());
            let mut expression = original.expression().unwrap();
            for _ in 0..16 {
                let Expression::UnaryExpression(unary) = expression else {
                    panic!("expected original unary prefix");
                };
                expression = &unary.argument;
            }
            let Expression::ArrayExpression(array) = expression else {
                panic!("expected original array");
            };
            assert_eq!(array.elements.len(), 1);
            let child = &array.elements[0];
            assert_eq!(
                matches!(child, ArrayExpressionElement::Elision(_)),
                is_elision
            );
            assert_eq!(
                original.decoded_span(child.span()).unwrap(),
                Span::new(17, 18)
            );
            let child_pointer = core::ptr::from_ref(child);
            assert_eq!(
                expression_document(
                    &original,
                    SourceRoot::new(&source).unwrap().whole_block(),
                    &arena
                )
                .unwrap_err(),
                ExpressionRefusal::DepthLimit {
                    span: Span::new(17, 18)
                }
            );
            assert_eq!(core::ptr::from_ref(original.expression().unwrap()), root);
            assert_eq!(core::ptr::from_ref(&array.elements[0]), child_pointer);
            assert_eq!(original.source().text(), source);
            assert_eq!(original.diagnostics().count(), 0);
        }
        assert_preserved(&supported, false);
    }
}

#[test]
fn malformed_arrays_and_non_ascii_gaps_preserve_original_holes_ast_and_source() {
    for source in ["[a", "[a,,", "[... ]", "[a b]", "[,", "[a, ;b]"] {
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
        ("[\u{a0}a]", false),
        ("[a\u{a0},b]", false),
        ("[a,\u{a0}b]", false),
        ("[a\u{a0}]", false),
        ("[\u{a0},]", false),
        ("[,\u{a0}]", false),
        ("[&#160;a]", true),
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
            assert_eq!(original.hole(), None, "{source}");
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
