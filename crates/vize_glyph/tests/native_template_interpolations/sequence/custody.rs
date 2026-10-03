//! Original sequence storage and reference-related syntax survive real reparse.

use super::{SCRIPTS, WIDTHS, assert_preserved, format, newline, options};
use crate::{operands, preservation::fingerprint, selected};
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_glyph::native_doc::{LineEnding, native_template_document, print};
use vize_l0::Allocator;

#[test]
fn retained_reference_argument_and_order_structure_controls_remain_distinct_after_reparse() {
    // These are actual syntax/source controls, not executions of eval or JavaScript.
    for (left, right) in [
        ("(0,obj.method)()", "(obj.method)()"),
        ("(0,eval)('x')", "(eval)('x')"),
        ("delete (0,obj.x)", "delete (obj.x)"),
        ("typeof (a,b)", "(typeof a,b)"),
        ("f((a,b))", "f(a,b)"),
        ("[(a,b)]", "[a,b]"),
        ("obj[a,b]", "obj[b,a]"),
        ("a,(b,c)", "(a,b),c"),
        ("f(),g()", "g(),f()"),
    ] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let left_source = vize_l0::cstr!("<template>{{{{{left}}}}}</template>{script}");
            let right_source = vize_l0::cstr!("<template>{{{{{right}}}}}</template>{script}");
            let left_owner = selected(&arena, &left_source);
            let right_owner = selected(&arena, &right_source);
            let left_operands = operands(&left_owner);
            let right_operands = operands(&right_owner);
            let a = left_operands.first().unwrap().syntax();
            let b = right_operands.first().unwrap().syntax();
            assert_eq!(a.hole(), None);
            assert_eq!(b.hole(), None);
            let expected = [
                fingerprint(a, a.expression().unwrap()),
                fingerprint(b, b.expression().unwrap()),
            ];
            assert_ne!(
                expected[0], expected[1],
                "{left} must differ structurally from {right}"
            );
            for (source, shape) in [(&left_source, &expected[0]), (&right_source, &expected[1])] {
                for width in WIDTHS {
                    for ending in [LineEnding::Lf, LineEnding::CrLf] {
                        let output = format(source, options(width, ending));
                        let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                        let second = selected(&arena, &replay);
                        let second_operands = operands(&second);
                        let syntax = second_operands.first().unwrap().syntax();
                        assert_eq!(syntax.hole(), None, "{replay}");
                        assert_eq!(
                            &fingerprint(syntax, syntax.expression().unwrap()),
                            shape,
                            "{replay}"
                        );
                        assert_eq!(format(&replay, options(width, ending)), output);
                    }
                }
            }
        }
        assert_preserved(left);
        assert_preserved(right);
    }
}

#[test]
fn sequence_document_keeps_original_ordered_children_pure_call_comments_map_and_content_storage() {
    let content = "/*#__PURE__*/f&#40;/*x*/a&#41;&#44;/*,*/b";
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source =
            vize_l0::cstr!("<!--前--><template><p>{{{{ \t{content} \t}}}}</p></template>{script}");
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let operand = original.first().unwrap();
        let syntax = operand.syntax();
        assert_eq!(syntax.hole(), None);
        let ast = core::ptr::from_ref(syntax.expression().unwrap());
        let Expression::SequenceExpression(sequence) = syntax.expression().unwrap() else {
            panic!("actual SequenceExpression")
        };
        assert_eq!(sequence.expressions.len(), 2);
        let span = sequence.span;
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
        let Expression::CallExpression(call) = sequence.expressions.first().unwrap() else {
            panic!("actual pure Call first child")
        };
        assert!(call.pure);
        assert!(!call.optional);
        assert!(call.type_arguments.is_none());
        let call_root = core::ptr::from_ref(&**call);
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
                let body = "/*#__PURE__*/f &#40;/*x*/a &#41;&#44;/*,*/b";
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
                assert_eq!(sequence.span, span);
                assert_eq!(sequence.expressions.as_ptr(), storage);
                assert_eq!(sequence.expressions.len(), 2);
                assert_eq!(
                    sequence
                        .expressions
                        .iter()
                        .map(core::ptr::from_ref)
                        .collect::<std::vec::Vec<_>>(),
                    children
                );
                assert_eq!(
                    sequence
                        .expressions
                        .iter()
                        .map(GetSpan::span)
                        .collect::<std::vec::Vec<_>>(),
                    spans
                );
                assert_eq!(core::ptr::from_ref(&**call), call_root);
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
                assert!(core::ptr::eq(syntax.source().text(), view.text()));
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
