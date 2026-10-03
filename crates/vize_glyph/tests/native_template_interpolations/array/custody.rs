//! Original sparse shape, typed Elisions and source owners survive printing.

use super::{SCRIPTS, WIDTHS, assert_preserved, format, newline, options};
use crate::{operands, selected};
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_span::GetSpan;
use vize_glyph::native_doc::{LineEnding, native_template_document, print};
use vize_l0::Allocator;

fn holes(expression: &Expression<'_>) -> std::vec::Vec<bool> {
    let Expression::ArrayExpression(array) = expression else {
        panic!("actual ArrayExpression")
    };
    array
        .elements
        .iter()
        .map(|element| element.is_elision())
        .collect()
}

#[test]
fn original_hole_count_and_positions_distinguish_empty_undefined_and_trailing_comma_arrays() {
    for (content, shape) in [
        ("[]", vec![]),
        ("[undefined]", vec![false]),
        ("[,]", vec![true]),
        ("[,,]", vec![true, true]),
        ("[a,]", vec![false]),
        ("[a,,]", vec![false, true]),
        ("[,a]", vec![true, false]),
        ("[,,a,,]", vec![true, true, false, true]),
        ("[a,/*,*/,b,]", vec![false, true, false]),
        ("&#91;a&#44;&#44;&#93;", vec![false, true]),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<template>{{{{{content}}}}}</template>{script}");
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let syntax = original.first().unwrap().syntax();
            let root = core::ptr::from_ref(syntax.expression().unwrap());
            assert_eq!(holes(syntax.expression().unwrap()), shape, "{content}");
            for width in WIDTHS {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let output = format(&source, options(width, ending));
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    let second = selected(&arena, &replay);
                    let second_operands = operands(&second);
                    let retained = second_operands.first().unwrap().syntax();
                    assert_eq!(retained.hole(), None, "{replay}");
                    assert_eq!(holes(retained.expression().unwrap()), shape, "{replay}");
                    assert_eq!(format(&replay, options(width, ending)), output);
                    assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), root);
                }
            }
        }
        assert_preserved(content);
    }
}

#[test]
fn sparse_array_document_keeps_original_elisions_pure_call_comments_map_and_full_content_storage() {
    let content = "&#91;,/*h*/&#44;/*#__PURE__*/f&#40;a&#41;,,/*t*/&#93;";
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source =
            vize_l0::cstr!("<!--前--><template><p>{{{{{content}}}}}</p></template>{script}");
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let operand = original.first().unwrap();
        let syntax = operand.syntax();
        assert_eq!(syntax.hole(), None);
        let ast = core::ptr::from_ref(syntax.expression().unwrap());
        let Expression::ArrayExpression(array) = syntax.expression().unwrap() else {
            panic!("actual ArrayExpression")
        };
        assert_eq!(
            holes(syntax.expression().unwrap()),
            [true, true, false, true]
        );
        let storage = array.elements.as_ptr();
        let element_roots = array
            .elements
            .iter()
            .map(|element| match element {
                ArrayExpressionElement::Elision(hole) => core::ptr::from_ref(&**hole).cast::<()>(),
                _ => core::ptr::from_ref(element.as_expression().unwrap()).cast::<()>(),
            })
            .collect::<std::vec::Vec<_>>();
        let element_spans = array
            .elements
            .iter()
            .map(GetSpan::span)
            .collect::<std::vec::Vec<_>>();
        let hole_spellings = array
            .elements
            .iter()
            .filter_map(|element| {
                let ArrayExpressionElement::Elision(hole) = element else {
                    return None;
                };
                let decoded = syntax.decoded_span(hole.span).unwrap();
                assert_eq!(decoded.slice(syntax.source().text()), ",");
                assert_eq!(decoded.len(), 1);
                Some(syntax.authored_span(hole.span).unwrap().slice(&source))
            })
            .collect::<std::vec::Vec<_>>();
        assert_eq!(hole_spellings, [",", "&#44;", ","]);
        let Expression::CallExpression(call) =
            array.elements.get(2).unwrap().as_expression().unwrap()
        else {
            panic!("actual pure Call element")
        };
        assert!(call.pure);
        assert!(!call.optional);
        assert!(call.type_arguments.is_none());
        let callee = core::ptr::from_ref(&call.callee);
        let arguments = call.arguments.as_ptr();
        let argument =
            core::ptr::from_ref(call.arguments.first().unwrap().as_expression().unwrap());
        let comments = syntax
            .comments()
            .map(|comment| {
                (
                    comment.kind(),
                    comment.decoded_span().unwrap(),
                    comment.authored_span().unwrap(),
                    comment.text().unwrap().as_ptr(),
                    comment.authored_span().unwrap().slice(&source).to_owned(),
                )
            })
            .collect::<std::vec::Vec<_>>();
        assert_eq!(comments.len(), 3);
        let view = syntax.source();
        let map = view.decode_map().unwrap().segments();
        let map_values = map.to_vec();
        let raw = operand.raw_content().as_ptr();
        let content_span = operand.content_span();
        let full = operand.full_span();
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        let document = native_template_document(&owner, &refs, &arena).unwrap();
        for width in WIDTHS {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                let generated = newline(ending);
                let body = "&#91; ,/*h*/&#44;/*#__PURE__*/f &#40; a &#41;, ,/*t*/&#93;";
                let expected = if width < 80 {
                    vize_l0::cstr!("<p>{{{{{generated}    {body}{generated}  }}}}</p>")
                } else {
                    vize_l0::cstr!("<p>{{{{ {body} }}}}</p>")
                };
                let output = print(document.document(), &options(width, ending));
                assert_eq!(output, expected, "{source} / {width} / {ending:?}");
                assert!(core::ptr::eq(document.original(), &owner));
                assert!(core::ptr::eq(document.operands(), refs.as_slice()));
                assert!(core::ptr::eq(
                    *document.operands().first().unwrap(),
                    operand
                ));
                assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), ast);
                assert_eq!(array.elements.as_ptr(), storage);
                assert_eq!(array.elements.len(), 4);
                assert_eq!(
                    array
                        .elements
                        .iter()
                        .map(|element| match element {
                            ArrayExpressionElement::Elision(hole) =>
                                core::ptr::from_ref(&**hole).cast::<()>(),
                            _ => core::ptr::from_ref(element.as_expression().unwrap()).cast::<()>(),
                        })
                        .collect::<std::vec::Vec<_>>(),
                    element_roots
                );
                assert_eq!(
                    array
                        .elements
                        .iter()
                        .map(GetSpan::span)
                        .collect::<std::vec::Vec<_>>(),
                    element_spans
                );
                assert!(call.pure);
                assert_eq!(core::ptr::from_ref(&call.callee), callee);
                assert_eq!(call.arguments.as_ptr(), arguments);
                assert_eq!(
                    core::ptr::from_ref(call.arguments.first().unwrap().as_expression().unwrap()),
                    argument
                );
                assert_eq!(
                    syntax
                        .comments()
                        .map(|comment| (
                            comment.kind(),
                            comment.decoded_span().unwrap(),
                            comment.authored_span().unwrap(),
                            comment.text().unwrap().as_ptr(),
                            comment.authored_span().unwrap().slice(&source).to_owned(),
                        ))
                        .collect::<std::vec::Vec<_>>(),
                    comments
                );
                assert!(core::ptr::eq(
                    syntax.source().decode_map().unwrap().segments(),
                    map
                ));
                assert_eq!(
                    syntax.source().decode_map().unwrap().segments(),
                    map_values.as_slice()
                );
                assert!(core::ptr::eq(
                    syntax.source().authored_root(),
                    source.as_str()
                ));
                assert_eq!(syntax.source().text().as_ptr(), view.text().as_ptr());
                assert_eq!(syntax.source().span(), view.span());
                assert_eq!(operand.raw_content().as_ptr(), raw);
                assert_eq!(operand.content_span(), content_span);
                assert_eq!(operand.full_span(), full);
                assert_eq!(content_span.slice(&source), operand.raw_content());
                let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                assert_eq!(format(&replay, options(width, ending)), output);
            }
        }
    }
    assert_preserved(content);
}
