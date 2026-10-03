//! Exact original sparse elements, annotation, decode map and authored root.

use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_span::GetSpan;
use vize_glyph::native_doc::{ExpressionRefusal, LineEnding, expression_document, print};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

use super::{assert_preserved, options, retained};

#[test]
fn original_sparse_elements_pure_call_comments_map_and_nonzero_source_keep_exact_custody() {
    let arena = Allocator::default();
    let source = "é &#91;&#44;/*#__PURE__*/f(a)&#44;&#44; &#93; tail";
    let selected = "&#91;&#44;/*#__PURE__*/f(a)&#44;&#44; &#93;";
    let span = Span::new(3, (3 + selected.len()) as u32);
    let original = retained(&arena, source, span, Lang::Ts, true);
    let Expression::ArrayExpression(array) = original.expression().unwrap() else {
        panic!("original Array")
    };
    assert_eq!(array.elements.len(), 3);
    for index in [0, 2] {
        let ArrayExpressionElement::Elision(hole) = &array.elements[index] else {
            panic!("original Elision")
        };
        let decoded = original.decoded_span(hole.span()).unwrap();
        assert_eq!(decoded.end - decoded.start, 1);
        assert_eq!(decoded.slice(original.source().text()), ",");
        assert_eq!(
            original.authored_span(hole.span()).unwrap().slice(source),
            "&#44;"
        );
    }
    let Expression::CallExpression(call) = array.elements[1].as_expression().unwrap() else {
        panic!("original Call element")
    };
    assert!(call.pure);
    let ast = core::ptr::from_ref(original.expression().unwrap());
    let elements = array.elements.as_ptr();
    let children = array
        .elements
        .iter()
        .map(core::ptr::from_ref)
        .collect::<std::vec::Vec<_>>();
    let spans = array
        .elements
        .iter()
        .map(GetSpan::span)
        .collect::<std::vec::Vec<_>>();
    let callee = core::ptr::from_ref(&call.callee);
    let arguments = call.arguments.as_ptr();
    let argument = core::ptr::from_ref(call.arguments[0].as_expression().unwrap());
    let comments = original
        .comments()
        .map(|comment| {
            (
                comment.kind(),
                comment.decoded_span().unwrap(),
                comment.authored_span().unwrap(),
                comment.text().unwrap().as_ptr(),
            )
        })
        .collect::<std::vec::Vec<_>>();
    assert_eq!(comments.len(), 1);
    let view = original.source();
    let decoded = view.text().as_ptr();
    let map = view.decode_map().unwrap().segments();
    let map_values = map.to_vec();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(&source[3..span.end as usize], 3)
        .unwrap();
    let (same, document) = expression_document(&original, block, &arena)
        .unwrap()
        .into_parts();
    for width in [0, 1, 7, 80, 200] {
        for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
            assert_eq!(
                print(&document, &options(width, line_ending)),
                "&#91; &#44;/*#__PURE__*/f ( a )&#44; &#44; &#93;"
            );
            assert!(core::ptr::eq(same, &original));
            assert_eq!(core::ptr::from_ref(same.expression().unwrap()), ast);
            let Expression::ArrayExpression(same_array) = same.expression().unwrap() else {
                panic!("same Array")
            };
            assert!(core::ptr::eq(same_array, array));
            assert_eq!(same_array.elements.as_ptr(), elements);
            assert_eq!(
                same_array
                    .elements
                    .iter()
                    .map(core::ptr::from_ref)
                    .collect::<std::vec::Vec<_>>(),
                children
            );
            assert_eq!(
                same_array
                    .elements
                    .iter()
                    .map(GetSpan::span)
                    .collect::<std::vec::Vec<_>>(),
                spans
            );
            let Expression::CallExpression(same_call) =
                same_array.elements[1].as_expression().unwrap()
            else {
                panic!("same original Call")
            };
            assert!(same_call.pure);
            assert_eq!(core::ptr::from_ref(&same_call.callee), callee);
            assert_eq!(same_call.arguments.as_ptr(), arguments);
            assert_eq!(
                core::ptr::from_ref(same_call.arguments[0].as_expression().unwrap()),
                argument
            );
            assert_eq!(same.source().text().as_ptr(), decoded);
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
            assert_eq!(
                same.comments()
                    .map(|comment| {
                        (
                            comment.kind(),
                            comment.decoded_span().unwrap(),
                            comment.authored_span().unwrap(),
                            comment.text().unwrap().as_ptr(),
                        )
                    })
                    .collect::<std::vec::Vec<_>>(),
                comments
            );
            assert_eq!(same.diagnostics().count(), 0);
        }
    }
    let copied = source.to_owned();
    for block in [
        SourceRoot::new(&copied).unwrap().whole_block(),
        SourceRoot::new(source)
            .unwrap()
            .block(&source[3..4], 3)
            .unwrap(),
    ] {
        assert!(matches!(
            expression_document(&original, block, &arena),
            Err(ExpressionRefusal::SourceMismatch { .. })
        ));
        assert_eq!(array.elements.as_ptr(), elements);
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), ast);
    }
    assert_preserved(selected, true);
}

#[test]
fn sparse_doc_transfer_remains_usable_after_observation_wrapper_scope_with_source_arena_alive() {
    let arena = Allocator::default();
    let source = "&#91;&#44;a&#44;&#44;&#93;";
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
        document
    };
    // Only the observation wrapper leaves scope; no AST destruction is asserted.
    for width in [0, 1, 7, 80, 200] {
        for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
            assert_eq!(
                print(&document, &options(width, line_ending)),
                "&#91; &#44; a&#44; &#44; &#93;"
            );
        }
    }
}
