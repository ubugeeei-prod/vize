//! Genuine three-child and pure-test Call storage survives every printed layout.

use super::{SCRIPTS, WIDTHS, assert_preserved, newline, options};
use crate::{operands, selected};
use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{LineEnding, native_template_document, print};
use vize_l0::Allocator;

#[test]
fn conditional_document_keeps_original_owner_three_children_pure_call_and_all_source_storage() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!(
            "<!--前--><template><p>{{{{/*#__PURE__*/f&#40;a&#41; &#63; /*yes*/b &#58; /*no*/c}}}}</p></template>{script}"
        );
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let operand = original.first().unwrap();
        let syntax = operand.syntax();
        let ast = core::ptr::from_ref(syntax.expression().unwrap());
        let Expression::ConditionalExpression(conditional) = syntax.expression().unwrap() else {
            panic!("actual Conditional");
        };
        let children = [
            core::ptr::from_ref(&conditional.test),
            core::ptr::from_ref(&conditional.consequent),
            core::ptr::from_ref(&conditional.alternate),
        ];
        let Expression::CallExpression(call) = &conditional.test else {
            panic!("actual Call test")
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
                )
            })
            .collect::<std::vec::Vec<_>>();
        assert_eq!(comments.len(), 3);
        let view = syntax.source();
        let map = view.decode_map().unwrap().segments();
        let map_values = map.to_vec();
        let raw = operand.raw_content().as_ptr();
        let content = operand.content_span();
        let full = operand.full_span();
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        let document = native_template_document(&owner, &refs, &arena).unwrap();
        for width in WIDTHS {
            for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                let generated = newline(line_ending);
                let body = "/*#__PURE__*/f &#40; a &#41; &#63; /*yes*/b &#58; /*no*/c";
                let expected = if width < 80 {
                    vize_l0::cstr!("<p>{{{{{generated}    {body}{generated}  }}}}</p>")
                } else {
                    vize_l0::cstr!("<p>{{{{ {body} }}}}</p>")
                };
                assert_eq!(
                    print(document.document(), &options(width, line_ending)),
                    expected
                );
                assert!(core::ptr::eq(document.original(), &owner));
                assert!(core::ptr::eq(document.operands(), refs.as_slice()));
                assert!(core::ptr::eq(
                    *document.operands().first().unwrap(),
                    operand
                ));
                assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), ast);
                assert_eq!(
                    [
                        core::ptr::from_ref(&conditional.test),
                        core::ptr::from_ref(&conditional.consequent),
                        core::ptr::from_ref(&conditional.alternate),
                    ],
                    children
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
                assert_eq!(operand.content_span(), content);
                assert_eq!(operand.full_span(), full);
                assert_eq!(content.slice(&source), operand.raw_content());
            }
        }
    }
    assert_preserved("/*#__PURE__*/f&#40;a&#41; &#63; /*yes*/b &#58; /*no*/c");
}
