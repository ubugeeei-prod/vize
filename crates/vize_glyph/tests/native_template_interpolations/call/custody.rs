//! Same original selected Call owner and storage across complete printing.

use super::super::{operands, selected};
use super::WIDTHS;
use super::assert_preserved;
use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{LineEnding, PrintOptions, native_template_document, print};
use vize_l0::Allocator;

#[test]
fn pure_call_documents_borrow_original_callee_arguments_comments_map_and_content_storage() {
    let arena = Allocator::default();
    let source = "<!--前--><template><p>{{/*#__PURE__*/f&#40;/*a*/a, b,/*z*/&#41;}}</p></template>";
    let owner = selected(&arena, source);
    let original = operands(&owner);
    let operand = original.first().unwrap();
    let syntax = operand.syntax();
    let ast = core::ptr::from_ref(syntax.expression().unwrap());
    let Expression::CallExpression(call) = syntax.expression().unwrap() else {
        panic!("actual Call")
    };
    assert!(call.pure);
    assert!(!call.optional);
    assert!(call.type_arguments.is_none());
    assert_eq!(call.arguments.len(), 2);
    let callee = core::ptr::from_ref(&call.callee);
    let arguments = call.arguments.as_ptr();
    let argument_roots = call
        .arguments
        .iter()
        .map(|arg| core::ptr::from_ref(arg.as_expression().unwrap()))
        .collect::<std::vec::Vec<_>>();
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
    let source_view = syntax.source();
    let map = source_view.decode_map().unwrap().segments();
    let map_values = map.to_vec();
    let raw = operand.raw_content().as_ptr();
    let content = operand.content_span();
    let full = operand.full_span();
    let refs = original.iter().collect::<std::vec::Vec<_>>();
    let document = native_template_document(&owner, &refs, &arena).unwrap();
    for width in WIDTHS {
        for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
            let newline = if line_ending == LineEnding::Lf {
                "\n"
            } else {
                "\r\n"
            };
            let body = "/*#__PURE__*/f &#40;/*a*/a, b,/*z*/&#41;";
            let expected = if width < 80 {
                vize_l0::cstr!("<p>{{{{{newline}    {body}{newline}  }}}}</p>")
            } else {
                vize_l0::cstr!("<p>{{{{ {body} }}}}</p>")
            };
            assert_eq!(
                print(
                    document.document(),
                    &PrintOptions {
                        width,
                        line_ending,
                        ..PrintOptions::default()
                    }
                ),
                expected
            );
            assert!(core::ptr::eq(document.original(), &owner));
            assert!(core::ptr::eq(document.operands(), refs.as_slice()));
            assert!(core::ptr::eq(
                *document.operands().first().unwrap(),
                operand
            ));
            assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), ast);
            assert_eq!(core::ptr::from_ref(&call.callee), callee);
            assert_eq!(call.arguments.as_ptr(), arguments);
            assert_eq!(
                call.arguments
                    .iter()
                    .map(|arg| core::ptr::from_ref(arg.as_expression().unwrap()))
                    .collect::<std::vec::Vec<_>>(),
                argument_roots
            );
            assert!(call.pure);
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
            assert!(core::ptr::eq(syntax.source().authored_root(), source));
            assert_eq!(syntax.source().text().as_ptr(), source_view.text().as_ptr());
            assert_eq!(syntax.source().span(), source_view.span());
            assert_eq!(operand.raw_content().as_ptr(), raw);
            assert_eq!(operand.content_span(), content);
            assert_eq!(operand.full_span(), full);
            assert_eq!(content.slice(source), operand.raw_content());
        }
    }
    assert_preserved("/*#__PURE__*/f&#40;/*a*/a, b,/*z*/&#41;");
}
