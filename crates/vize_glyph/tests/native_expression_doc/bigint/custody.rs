//! Same original scalar nodes, values, raw spellings and mapped source custody.

use oxc_ast::ast::{BigintBase, Expression};
use vize_glyph::native_doc::{ExpressionRefusal, LineEnding, expression_document, print};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

use super::{assert_preserved, options, retained};

#[test]
fn original_bigint_decimal_values_bases_raw_options_and_spans_survive_document_consumption() {
    for (spelling, base, decimal) in [
        ("0n", BigintBase::Decimal, "0"),
        ("1_000n", BigintBase::Decimal, "1000"),
        ("0b10_10n", BigintBase::Binary, "10"),
        ("0B1_0n", BigintBase::Binary, "2"),
        ("0o7_7n", BigintBase::Octal, "63"),
        ("0O1_0n", BigintBase::Octal, "8"),
        ("0xA_Fn", BigintBase::Hex, "175"),
        ("0Xf_Fn", BigintBase::Hex, "255"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let arena = Allocator::default();
            let source = std::format!(" \t{spelling}\r\n ");
            let original = retained(
                &arena,
                &source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            assert_eq!(original.hole(), None);
            assert_eq!(original.diagnostics().count(), 0);
            let Expression::BigIntLiteral(literal) = original.expression().unwrap() else {
                panic!("original BigInt atom")
            };
            assert_eq!(literal.base, base);
            assert_eq!(literal.value.as_str(), decimal);
            assert_eq!(literal.raw.as_ref().unwrap().as_str(), spelling);
            assert_eq!(
                original.decoded_span(literal.span).unwrap(),
                Span::new(2, 2 + spelling.len() as u32)
            );
            let ast = core::ptr::from_ref(original.expression().unwrap());
            let root = core::ptr::from_ref(&**literal);
            let value = literal.value.as_str().as_ptr();
            let raw = literal.raw.as_ref().unwrap().as_str().as_ptr();
            let span = literal.span;
            let (same, document) = expression_document(
                &original,
                SourceRoot::new(&source).unwrap().whole_block(),
                &arena,
            )
            .unwrap()
            .into_parts();
            for width in [0, 1, 7, 80, 200] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(print(&document, &options(width, ending)), spelling);
                    assert!(core::ptr::eq(same, &original));
                    assert_eq!(core::ptr::from_ref(same.expression().unwrap()), ast);
                    let Expression::BigIntLiteral(unchanged) = same.expression().unwrap() else {
                        panic!("same original BigInt atom")
                    };
                    assert_eq!(core::ptr::from_ref(&**unchanged), root);
                    assert_eq!(unchanged.base, base);
                    assert_eq!(unchanged.value.as_str(), decimal);
                    assert_eq!(unchanged.value.as_str().as_ptr(), value);
                    assert_eq!(unchanged.raw.as_ref().unwrap().as_str(), spelling);
                    assert_eq!(unchanged.raw.as_ref().unwrap().as_str().as_ptr(), raw);
                    assert_eq!(unchanged.span, span);
                    assert_eq!(same.source().text(), source);
                    assert_eq!(same.source().text().as_ptr(), source.as_ptr());
                    assert!(same.source().decode_map().is_none());
                }
            }
        }
    }
}

#[test]
fn mapped_nonzero_bigint_keeps_original_node_raw_value_comments_and_source_map_storage() {
    let arena = Allocator::default();
    let selected = "/*l*/&#48;xA_F&#110;/*t*/";
    let source = std::format!("é {selected} tail");
    let span = Span::new(3, (3 + selected.len()) as u32);
    let original = retained(&arena, &source, span, Lang::Ts, true);
    assert_eq!(original.hole(), None);
    let Expression::BigIntLiteral(literal) = original.expression().unwrap() else {
        panic!("original mapped BigInt")
    };
    assert_eq!(literal.base, BigintBase::Hex);
    assert_eq!(literal.value.as_str(), "175");
    assert_eq!(literal.raw.as_ref().unwrap().as_str(), "0xA_Fn");
    let ast = core::ptr::from_ref(original.expression().unwrap());
    let scalar = core::ptr::from_ref(&**literal);
    let value = literal.value.as_str().as_ptr();
    let raw = literal.raw.as_ref().unwrap().as_str().as_ptr();
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
    let before_comments = comments();
    assert_eq!(before_comments.len(), 2);
    let view = original.source();
    let map = view.decode_map().unwrap().segments();
    let map_values = map.to_vec();
    let block = SourceRoot::new(&source)
        .unwrap()
        .block(&source[3..span.end as usize], 3)
        .unwrap();
    let (same, document) = expression_document(&original, block, &arena)
        .unwrap()
        .into_parts();
    for width in [0, 1, 7, 80, 200] {
        for ending in [LineEnding::Lf, LineEnding::CrLf] {
            assert_eq!(print(&document, &options(width, ending)), selected);
            assert!(core::ptr::eq(same, &original));
            assert_eq!(core::ptr::from_ref(same.expression().unwrap()), ast);
            assert_eq!(core::ptr::from_ref(&**literal), scalar);
            assert_eq!(literal.value.as_str().as_ptr(), value);
            assert_eq!(literal.raw.as_ref().unwrap().as_str().as_ptr(), raw);
            assert_eq!(same.source().text().as_ptr(), view.text().as_ptr());
            assert_eq!(same.source().text(), "/*l*/0xA_Fn/*t*/");
            assert_eq!(same.source().authored_root().as_ptr(), source.as_ptr());
            assert_eq!(same.source().span(), span);
            assert!(core::ptr::eq(
                same.source().decode_map().unwrap().segments(),
                map
            ));
            assert_eq!(
                same.source().decode_map().unwrap().segments(),
                map_values.as_slice()
            );
            assert_eq!(comments(), before_comments);
            assert_eq!(same.diagnostics().count(), 0);
        }
    }
    let copied = source.clone();
    for foreign in [
        SourceRoot::new(&copied).unwrap().whole_block(),
        SourceRoot::new(&source)
            .unwrap()
            .block(&source[3..4], 3)
            .unwrap(),
    ] {
        assert!(matches!(
            expression_document(&original, foreign, &arena),
            Err(ExpressionRefusal::SourceMismatch { .. })
        ));
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), ast);
        assert_eq!(comments(), before_comments);
        assert!(core::ptr::eq(
            original.source().decode_map().unwrap().segments(),
            map
        ));
    }
    assert_preserved(selected, true);
}

#[test]
fn transferred_bigint_doc_survives_wrapper_scope_with_original_source_and_arena_alive() {
    let arena = Allocator::default();
    let source = "&#49;&#110;";
    let document = {
        let original = retained(
            &arena,
            source,
            Span::new(0, source.len() as u32),
            Lang::Js,
            true,
        );
        let carrier = expression_document(
            &original,
            SourceRoot::new(source).unwrap().whole_block(),
            &arena,
        )
        .unwrap();
        assert!(core::ptr::eq(carrier.original(), &original));
        let (same, document) = carrier.into_parts();
        assert!(core::ptr::eq(same, &original));
        assert!(matches!(
            same.expression().unwrap(),
            Expression::BigIntLiteral(_)
        ));
        document
    };
    // Original arena storage remains alive; no instrumented destructor is claimed.
    for width in [0, 1, 7, 80, 200] {
        for ending in [LineEnding::Lf, LineEnding::CrLf] {
            assert_eq!(print(&document, &options(width, ending)), source);
        }
    }
}
