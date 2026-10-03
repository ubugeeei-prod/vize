//! Original Object property order, key spellings and borrowed source custody.

use super::{SCRIPTS, WIDTHS, assert_preserved, format, newline, options};
use crate::{operands, preservation::fingerprint, selected};
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey, PropertyKind};
use oxc_span::GetSpan;
use vize_glyph::native_doc::{LineEnding, native_template_document, print};
use vize_l0::Allocator;

#[test]
fn original_key_spelling_duplicate_value_order_and_pure_call_controls_survive_reparse() {
    // Syntax/source comparisons do not execute getters or certify early errors.
    for (left, right) in [
        ("{a:b}", "{'a':b}"),
        ("{'a':b}", "{'\\u0061':b}"),
        ("{a:b}", "{\\u0061:b}"),
        ("{1:b}", "{1.0:b}"),
        ("{get:a}", "{'get':a}"),
        ("{a:b,a:c}", "{a:c,a:b}"),
        ("{a:obj.x,b:f()}", "{b:f(),a:obj.x}"),
        ("{a:(b,c)}", "{a:(c,b)}"),
        ("{a:/*#__PURE__*/f()}", "{a:f()}"),
    ] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let left_source = vize_l0::cstr!("<template>{{{{{left} }}}}</template>{script}");
            let right_source = vize_l0::cstr!("<template>{{{{{right} }}}}</template>{script}");
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
                "{left} must remain distinct from {right}"
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
        assert_preserved(&vize_l0::cstr!("{left} "));
        assert_preserved(&vize_l0::cstr!("{right} "));
    }
}

#[test]
fn object_document_borrows_original_property_key_value_comment_map_and_content_storage() {
    let content = "{a:/*#__PURE__*/f&#40;/*x*/b&#41;&#44;'c':obj.x}";
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source =
            vize_l0::cstr!("<!--前--><template><p>{{{{ \t{content} \t}}}}</p></template>{script}");
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let operand = original.first().unwrap();
        let syntax = operand.syntax();
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.source_type().is_typescript(), !script.is_empty());
        let ast = core::ptr::from_ref(syntax.expression().unwrap());
        let Expression::ObjectExpression(object) = syntax.expression().unwrap() else {
            panic!("actual Object")
        };
        assert_eq!(object.properties.len(), 2);
        let span = object.span;
        let storage = object.properties.as_ptr();
        let properties = object
            .properties
            .iter()
            .map(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    panic!("explicit property")
                };
                &**property
            })
            .collect::<std::vec::Vec<_>>();
        let pointers = properties
            .iter()
            .map(|property| core::ptr::from_ref(*property))
            .collect::<std::vec::Vec<_>>();
        let spans = properties
            .iter()
            .map(|property| property.span)
            .collect::<std::vec::Vec<_>>();
        let keys = properties
            .iter()
            .map(|property| core::ptr::from_ref(&property.key))
            .collect::<std::vec::Vec<_>>();
        let values = properties
            .iter()
            .map(|property| core::ptr::from_ref(&property.value))
            .collect::<std::vec::Vec<_>>();
        let key_spans = properties
            .iter()
            .map(|property| property.key.span())
            .collect::<std::vec::Vec<_>>();
        let value_spans = properties
            .iter()
            .map(|property| property.value.span())
            .collect::<std::vec::Vec<_>>();
        let spellings = properties
            .iter()
            .map(|property| {
                syntax
                    .authored_span(property.key.span())
                    .unwrap()
                    .slice(&source)
                    .to_owned()
            })
            .collect::<std::vec::Vec<_>>();
        assert_eq!(spellings, ["a", "'c'"]);
        assert!(
            matches!(&properties[0].key, PropertyKey::StaticIdentifier(key) if key.name.as_str() == "a")
        );
        assert!(
            matches!(&properties[1].key, PropertyKey::StringLiteral(key) if key.value.as_str() == "c")
        );
        for property in &properties {
            assert_eq!(property.kind, PropertyKind::Init);
            assert!(!property.method && !property.shorthand && !property.computed);
        }
        let Expression::CallExpression(call) = &properties[0].value else {
            panic!("actual pure Call")
        };
        assert!(call.pure && !call.optional && call.type_arguments.is_none());
        let call_root = core::ptr::from_ref(&**call);
        let callee = core::ptr::from_ref(&call.callee);
        let arguments = call.arguments.as_ptr();
        let argument =
            core::ptr::from_ref(call.arguments.first().unwrap().as_expression().unwrap());
        let Expression::StaticMemberExpression(member) = &properties[1].value else {
            panic!("actual Member")
        };
        let member_root = core::ptr::from_ref(&**member);
        let object_child = core::ptr::from_ref(&member.object);
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
        assert_eq!(comments.len(), 2);
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
                let body = "{ a:/*#__PURE__*/f &#40;/*x*/b &#41;&#44; 'c': obj . x }";
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
                assert_eq!(object.span, span);
                assert_eq!(object.properties.as_ptr(), storage);
                assert_eq!(object.properties.len(), 2);
                for (index, property) in properties.iter().enumerate() {
                    let ObjectPropertyKind::ObjectProperty(same) = &object.properties[index] else {
                        panic!("same property")
                    };
                    assert_eq!(core::ptr::from_ref(&**same), pointers[index]);
                    assert!(core::ptr::eq(&**same, *property));
                    assert_eq!(same.span, spans[index]);
                    assert_eq!(core::ptr::from_ref(&same.key), keys[index]);
                    assert_eq!(core::ptr::from_ref(&same.value), values[index]);
                    assert_eq!(same.key.span(), key_spans[index]);
                    assert_eq!(same.value.span(), value_spans[index]);
                    assert_eq!(same.kind, PropertyKind::Init);
                    assert!(!same.method && !same.shorthand && !same.computed);
                    assert_eq!(
                        syntax
                            .authored_span(same.key.span())
                            .unwrap()
                            .slice(&source),
                        spellings[index]
                    );
                }
                assert_eq!(core::ptr::from_ref(&**call), call_root);
                assert!(call.pure);
                assert_eq!(core::ptr::from_ref(&call.callee), callee);
                assert_eq!(call.arguments.as_ptr(), arguments);
                assert_eq!(
                    core::ptr::from_ref(call.arguments.first().unwrap().as_expression().unwrap()),
                    argument
                );
                assert_eq!(core::ptr::from_ref(&**member), member_root);
                assert_eq!(core::ptr::from_ref(&member.object), object_child);
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
    assert_preserved(&vize_l0::cstr!("{content} "));
}
