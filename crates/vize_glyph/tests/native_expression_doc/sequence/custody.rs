//! Original Sequence vector, child annotations and authored source custody.

use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_glyph::native_doc::{ExpressionRefusal, LineEnding, expression_document, print};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

use super::{assert_preserved, options, retained};

#[test]
fn mapped_nonzero_sequence_keeps_original_ordered_children_pure_call_comments_and_source() {
    let arena = Allocator::default();
    let selected = "&#97;&#44;/*#__PURE__*/f&#40;b&#41;&#44;obj&#46;x";
    let source = std::format!("é {selected} tail");
    let span = Span::new(3, (3 + selected.len()) as u32);
    let original = retained(&arena, &source, span, Lang::Ts, true);
    assert_eq!(original.hole(), None);
    let Expression::SequenceExpression(sequence) = original.expression().unwrap() else {
        panic!("original Sequence")
    };
    assert_eq!(sequence.expressions.len(), 3);
    assert!(
        matches!(&sequence.expressions[0], Expression::Identifier(identifier) if identifier.name.as_str() == "a")
    );
    let Expression::CallExpression(call) = &sequence.expressions[1] else {
        panic!("original annotated Call")
    };
    assert!(call.pure);
    assert!(!call.optional);
    assert!(call.type_arguments.is_none());
    let Expression::StaticMemberExpression(member) = &sequence.expressions[2] else {
        panic!("original final Member")
    };
    assert_eq!(member.property.name.as_str(), "x");
    let ast = core::ptr::from_ref(original.expression().unwrap());
    let storage = sequence.expressions.as_ptr();
    let children = sequence
        .expressions
        .iter()
        .map(core::ptr::from_ref)
        .collect::<std::vec::Vec<_>>();
    let spans = sequence
        .expressions
        .iter()
        .map(GetSpan::span)
        .collect::<std::vec::Vec<_>>();
    let callee = core::ptr::from_ref(&call.callee);
    let arguments = call.arguments.as_ptr();
    let argument = core::ptr::from_ref(call.arguments[0].as_expression().unwrap());
    let object = core::ptr::from_ref(&member.object);
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
            assert_eq!(
                print(&document, &options(width, ending)),
                "&#97;&#44;/*#__PURE__*/f &#40; b &#41;&#44; obj &#46; x"
            );
            assert!(core::ptr::eq(same, &original));
            assert_eq!(core::ptr::from_ref(same.expression().unwrap()), ast);
            let Expression::SequenceExpression(same_sequence) = same.expression().unwrap() else {
                panic!("same original Sequence")
            };
            assert!(core::ptr::eq(same_sequence, sequence));
            assert_eq!(same_sequence.expressions.as_ptr(), storage);
            assert_eq!(
                same_sequence
                    .expressions
                    .iter()
                    .map(core::ptr::from_ref)
                    .collect::<std::vec::Vec<_>>(),
                children
            );
            assert_eq!(
                same_sequence
                    .expressions
                    .iter()
                    .map(GetSpan::span)
                    .collect::<std::vec::Vec<_>>(),
                spans
            );
            assert!(call.pure);
            assert_eq!(core::ptr::from_ref(&call.callee), callee);
            assert_eq!(call.arguments.as_ptr(), arguments);
            assert_eq!(
                core::ptr::from_ref(call.arguments[0].as_expression().unwrap()),
                argument
            );
            assert_eq!(core::ptr::from_ref(&member.object), object);
            assert_eq!(
                same.comments()
                    .map(|comment| (
                        comment.kind(),
                        comment.decoded_span().unwrap(),
                        comment.authored_span().unwrap(),
                        comment.text().unwrap().as_ptr()
                    ))
                    .collect::<std::vec::Vec<_>>(),
                comments
            );
            assert!(core::ptr::eq(
                same.source().decode_map().unwrap().segments(),
                map
            ));
            assert_eq!(
                same.source().decode_map().unwrap().segments(),
                map_values.as_slice()
            );
            assert_eq!(same.source().text().as_ptr(), view.text().as_ptr());
            assert_eq!(same.source().authored_root().as_ptr(), source.as_ptr());
            assert_eq!(same.source().span(), span);
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
        assert_eq!(sequence.expressions.as_ptr(), storage);
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), ast);
    }
    assert_preserved(selected, true);
}

#[test]
fn sequence_doc_transfer_survives_observation_wrapper_scope_with_authored_source_and_arena_alive() {
    let arena = Allocator::default();
    let source = "&#40;a&#44;b&#41;";
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
    // The arena-owned AST stays alive; this is ordinary wrapper scope only.
    for width in [0, 1, 7, 80, 200] {
        for ending in [LineEnding::Lf, LineEnding::CrLf] {
            assert_eq!(
                print(&document, &options(width, ending)),
                "&#40;a&#44; b&#41;"
            );
        }
    }
}
