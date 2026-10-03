//! The same original ternary, children, annotations, map and authored root.

use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{ExpressionRefusal, LineEnding, expression_document, print};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

use super::{assert_preserved, options, retained};

#[test]
fn original_nonzero_ternary_and_pure_call_children_keep_every_borrowed_observation() {
    let allocator = Allocator::default();
    let source = "é /*#__PURE__*/f()&#63;/*x*/g(a)&#58;h() tail";
    let selected = "/*#__PURE__*/f()&#63;/*x*/g(a)&#58;h()";
    let span = Span::new(3, (3 + selected.len()) as u32);
    let original = retained(&allocator, source, span, Lang::Ts, true);
    let Expression::ConditionalExpression(ternary) = original.expression().unwrap() else {
        panic!("original Conditional")
    };
    let Expression::CallExpression(test) = &ternary.test else {
        panic!("original test Call")
    };
    let Expression::CallExpression(consequent) = &ternary.consequent else {
        panic!("original consequent Call")
    };
    assert!(test.pure);
    assert!(!consequent.pure);
    let root = core::ptr::from_ref(original.expression().unwrap());
    let children = [
        core::ptr::from_ref(&ternary.test),
        core::ptr::from_ref(&ternary.consequent),
        core::ptr::from_ref(&ternary.alternate),
    ];
    let arguments = consequent.arguments.as_ptr();
    let argument = core::ptr::from_ref(consequent.arguments[0].as_expression().unwrap());
    let decoded = original.source().text().as_ptr();
    let map = original.source().decode_map().unwrap().segments().as_ptr();
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
    assert_eq!(comments.len(), 2);
    let block = SourceRoot::new(source)
        .unwrap()
        .block(&source[3..span.end as usize], 3)
        .unwrap();
    let (same, document) = expression_document(&original, block, &allocator)
        .unwrap()
        .into_parts();
    assert_eq!(
        print(&document, &options(200, LineEnding::Lf)),
        "/*#__PURE__*/f ( ) &#63;/*x*/g ( a ) &#58; h ( )"
    );
    for width in [0, 1, 7, 80, 200] {
        for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
            print(&document, &options(width, line_ending));
            assert!(core::ptr::eq(same, &original));
            assert_eq!(core::ptr::from_ref(same.expression().unwrap()), root);
            let Expression::ConditionalExpression(same_ternary) = same.expression().unwrap() else {
                panic!("same Conditional")
            };
            assert!(core::ptr::eq(same_ternary, ternary));
            assert_eq!(
                [
                    core::ptr::from_ref(&same_ternary.test),
                    core::ptr::from_ref(&same_ternary.consequent),
                    core::ptr::from_ref(&same_ternary.alternate),
                ],
                children
            );
            let Expression::CallExpression(same_call) = &same_ternary.consequent else {
                panic!("same original Call")
            };
            assert_eq!(same_call.arguments.as_ptr(), arguments);
            assert_eq!(
                core::ptr::from_ref(same_call.arguments[0].as_expression().unwrap()),
                argument
            );
            assert_eq!(same.source().text().as_ptr(), decoded);
            assert_eq!(same.source().authored_root().as_ptr(), source.as_ptr());
            assert_eq!(same.source().decode_map().unwrap().segments().as_ptr(), map);
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
    let foreign = source.to_owned();
    for wrong in [
        SourceRoot::new(&foreign).unwrap().whole_block(),
        SourceRoot::new(source)
            .unwrap()
            .block(&source[3..4], 3)
            .unwrap(),
    ] {
        assert!(matches!(
            expression_document(&original, wrong, &allocator),
            Err(ExpressionRefusal::SourceMismatch { .. })
        ));
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), root);
    }
    assert_preserved(selected, true);
}

#[test]
fn transferred_doc_outlives_only_the_observation_wrapper_with_original_source_and_arena_alive() {
    let allocator = Allocator::default();
    let source = "a?b:c";
    let document = {
        let original = retained(
            &allocator,
            source,
            Span::new(0, source.len() as u32),
            Lang::Js,
            false,
        );
        let carrier = expression_document(
            &original,
            SourceRoot::new(source).unwrap().whole_block(),
            &allocator,
        )
        .unwrap();
        assert!(core::ptr::eq(carrier.original(), &original));
        let (same, document) = carrier.into_parts();
        assert!(core::ptr::eq(same, &original));
        document
    };
    // The retained observation wrapper leaves scope; no AST destruction is asserted.
    assert_eq!(print(&document, &options(200, LineEnding::Lf)), "a ? b : c");
    assert_eq!(
        print(&document, &options(0, LineEnding::CrLf)),
        "a ?\r\n  b :\r\n  c"
    );
    assert_eq!(source, "a?b:c");
}
