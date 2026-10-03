//! Original property storage, static key leaves and source ownership custody.

use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey, PropertyKind};
use oxc_span::GetSpan;
use vize_glyph::native_doc::{ExpressionRefusal, LineEnding, expression_document, print};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

use super::{assert_preserved, options, retained};

#[test]
fn mapped_nonzero_object_keeps_ordered_properties_keys_values_pure_call_and_source_storage() {
    let arena = Allocator::default();
    let selected = "&#123;&#97;&#58;/*#__PURE__*/f&#40;b&#41;&#44;'c'&#58;d&#44;1&#58;e&#44;&#125;";
    let source = std::format!("é {selected} tail");
    let span = Span::new(3, (3 + selected.len()) as u32);
    let original = retained(&arena, &source, span, Lang::Ts, true);
    assert_eq!(original.hole(), None);
    let Expression::ObjectExpression(object) = original.expression().unwrap() else {
        panic!("original Object")
    };
    assert_eq!(object.properties.len(), 3);
    let observe = |object: &ObjectExpression<'_>| {
        object
            .properties
            .iter()
            .map(|entry| {
                let ObjectPropertyKind::ObjectProperty(property) = entry else {
                    panic!("original explicit property")
                };
                assert_eq!(property.kind, PropertyKind::Init);
                assert!(!property.method);
                assert!(!property.shorthand);
                assert!(!property.computed);
                let key_root = match &property.key {
                    PropertyKey::StaticIdentifier(key) => core::ptr::from_ref(&**key).cast::<()>(),
                    PropertyKey::StringLiteral(key) => core::ptr::from_ref(&**key).cast::<()>(),
                    PropertyKey::NumericLiteral(key) => core::ptr::from_ref(&**key).cast::<()>(),
                    _ => panic!("original static key"),
                };
                (
                    core::ptr::from_ref(&**property).cast::<()>(),
                    key_root,
                    core::ptr::from_ref(&property.value).cast::<()>(),
                    property.span,
                    property.key.span(),
                    property.value.span(),
                    property.kind,
                    property.method,
                    property.shorthand,
                    property.computed,
                )
            })
            .collect::<std::vec::Vec<_>>()
    };
    let ast = core::ptr::from_ref(original.expression().unwrap());
    let storage = object.properties.as_ptr();
    let properties = observe(object);
    let ObjectPropertyKind::ObjectProperty(first) = &object.properties[0] else {
        panic!("original first property")
    };
    let Expression::CallExpression(call) = &first.value else {
        panic!("original annotated Call value")
    };
    assert!(call.pure);
    assert!(!call.optional);
    assert!(call.type_arguments.is_none());
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
                "&#123; &#97;&#58;/*#__PURE__*/f &#40; b &#41;&#44; 'c'&#58; d&#44; 1&#58; e&#44; &#125;"
            );
            assert!(core::ptr::eq(same, &original));
            assert_eq!(core::ptr::from_ref(same.expression().unwrap()), ast);
            let Expression::ObjectExpression(same_object) = same.expression().unwrap() else {
                panic!("same original Object")
            };
            assert!(core::ptr::eq(same_object, object));
            assert_eq!(same_object.properties.as_ptr(), storage);
            assert_eq!(observe(same_object), properties);
            assert!(call.pure);
            assert_eq!(core::ptr::from_ref(&call.callee), callee);
            assert_eq!(call.arguments.as_ptr(), arguments);
            assert_eq!(
                core::ptr::from_ref(call.arguments[0].as_expression().unwrap()),
                argument
            );
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
        assert_eq!(object.properties.as_ptr(), storage);
        assert_eq!(observe(object), properties);
    }
    assert_preserved(selected, true);
}

#[test]
fn object_doc_transfer_survives_observation_wrapper_scope_with_source_and_arena_alive() {
    let arena = Allocator::default();
    let source = "&#123;a&#58;b&#125;";
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
    // The arena-owned AST stays alive; no destructor behavior is asserted.
    for width in [0, 1, 7, 80, 200] {
        for ending in [LineEnding::Lf, LineEnding::CrLf] {
            assert_eq!(
                print(&document, &options(width, ending)),
                "&#123; a&#58; b &#125;"
            );
        }
    }
}
