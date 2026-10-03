//! Original Call observation storage and selected authored-root custody.

use super::{options, retained};
use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{ExpressionRefusal, expression_document, print};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

#[test]
fn original_nonzero_call_callee_argument_comment_and_decode_map_storage_keep_custody() {
    let allocator = Allocator::default();
    let source = "é /*#__PURE__*/日本&#40;a&#44;b&#44;&#41; tail";
    let selected = "/*#__PURE__*/日本&#40;a&#44;b&#44;&#41;";
    let span = Span::new(3, (3 + selected.len()) as u32);
    let original = retained(&allocator, source, span, Lang::Ts, true);
    let Expression::CallExpression(call) = original.expression().unwrap() else {
        panic!("original Call")
    };
    assert!(call.pure);
    let callee = core::ptr::from_ref(&call.callee);
    let arguments = call.arguments.as_ptr();
    let children = call
        .arguments
        .iter()
        .map(|argument| core::ptr::from_ref(argument.as_expression().unwrap()))
        .collect::<std::vec::Vec<_>>();
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
    let map = original.source().decode_map().unwrap().segments().as_ptr();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(&source[3..span.end as usize], 3)
        .unwrap();
    let (same, document) = expression_document(&original, block, &allocator)
        .unwrap()
        .into_parts();
    assert_eq!(
        print(&document, &options(200)),
        "/*#__PURE__*/日本 &#40; a&#44; b&#44; &#41;"
    );
    assert!(core::ptr::eq(same, &original));
    let Expression::CallExpression(same_call) = same.expression().unwrap() else {
        panic!("same Call")
    };
    assert!(core::ptr::eq(same_call, call));
    assert_eq!(core::ptr::from_ref(&same_call.callee), callee);
    assert_eq!(same_call.arguments.as_ptr(), arguments);
    assert_eq!(
        same_call
            .arguments
            .iter()
            .map(|argument| core::ptr::from_ref(argument.as_expression().unwrap()))
            .collect::<std::vec::Vec<_>>(),
        children
    );
    assert_eq!(same.source().decode_map().unwrap().segments().as_ptr(), map);
    assert_eq!(same.source().authored_root().as_ptr(), source.as_ptr());
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
        assert_eq!(call.arguments.as_ptr(), arguments);
    }
}
