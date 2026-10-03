//! Whole static Object refusals preserve original syntax and fixed budgets.

use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey, PropertyKind};
use oxc_span::GetSpan;
use vize_glyph::native_doc::{ExpressionRefusal, LineEnding, expression_document};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::{Lang, syntax::RetainedExpression};

use super::{assert_preserved, format, options, retained};
use crate::refusals::assert_unsupported;

#[test]
fn unsupported_property_shapes_values_and_decoded_proto_keys_refuse_the_whole_document() {
    for content in [
        "{a}",
        "{a,b:c}",
        "{...a}",
        "{a:b,...c}",
        "{[a]:b}",
        "{['a']:b}",
        "{[1]:b}",
        "{1n:a}",
        "{f(){}}",
        "{async f(){}}",
        "{*f(){}}",
        "{get a(){}}",
        "{set a(v){}}",
        "{a:b?.c}",
        "{a:f?.()}",
        "{a:b=c}",
        "{a:await b}",
        "{a:++b}",
        "{a:f(...b)}",
        "{a:this}",
        "{a:1n}",
        "{a:()=>b}",
        "f({a:[...b]})",
        "obj[{a:b?.c}]",
        "a?{b:c}:{d:++e}",
        "({a:b}=c)",
        "{__proto__:a}",
        "{'__proto__':a}",
        "{\\u005f_proto__:a}",
        "{'\\x5f_proto__':a}",
        "{a:b,__proto__:c}",
        "{a:{__proto__:b}}",
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let arena = Allocator::default();
            let source = std::format!("/*kept*/ {content}");
            let original = retained(
                &arena,
                &source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            assert_unsupported(&original, &source, &arena);
        }
    }
    for content in [
        "{&#95;_proto__:a}",
        "{'&#95;_proto__':a}",
        "{__pr&#111;to__:a}",
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let arena = Allocator::default();
            let original = retained(
                &arena,
                content,
                Span::new(0, content.len() as u32),
                lang,
                true,
            );
            assert_encoded_proto_refusal(&original, content, &arena);
        }
    }
    for content in ["{a:b as T}", "{a:f<T>(b)}", "{a:b!}", "({a:b} as T)"] {
        let arena = Allocator::default();
        let source = std::format!("/*kept*/ {content}");
        let original = retained(
            &arena,
            &source,
            Span::new(0, source.len() as u32),
            Lang::Ts,
            false,
        );
        assert_unsupported(&original, &source, &arena);
    }
}

fn assert_encoded_proto_refusal<'a>(
    original: &RetainedExpression<'a>,
    source: &'a str,
    arena: &'a Allocator,
) {
    assert_eq!(original.hole(), None);
    assert!(original.admitted_expression().is_some());
    assert_eq!(original.diagnostics().count(), 0);
    let ast = core::ptr::from_ref(original.expression().unwrap());
    let Expression::ObjectExpression(object) = original.expression().unwrap() else {
        panic!("original encoded Object")
    };
    assert_eq!(object.properties.len(), 1);
    let properties = object.properties.as_ptr();
    let ObjectPropertyKind::ObjectProperty(property) = &object.properties[0] else {
        panic!("original encoded data property")
    };
    assert_eq!(property.kind, PropertyKind::Init);
    assert!(!property.method && !property.shorthand && !property.computed);
    let (name, spelling) = match &property.key {
        PropertyKey::StaticIdentifier(key) => (key.name.as_str(), "__proto__"),
        PropertyKey::StringLiteral(key) => (key.value.as_str(), "'__proto__'"),
        _ => panic!("original encoded static key"),
    };
    assert_eq!(name, "__proto__");
    let key_name = name.as_ptr();
    let property_root = core::ptr::from_ref(&**property);
    let key = core::ptr::from_ref(&property.key);
    let value = core::ptr::from_ref(&property.value);
    let property_span = original.decoded_span(property.span).unwrap();
    let key_span = original.decoded_span(property.key.span()).unwrap();
    let value_span = original.decoded_span(property.value.span()).unwrap();
    assert_eq!(property_span.start, key_span.start);
    assert_eq!(property_span.end, value_span.end);
    assert_eq!(key_span.end + 1, value_span.start);
    let authored_key = original.authored_span(property.key.span()).unwrap();
    assert_eq!(authored_key, Span::new(1, source.len() as u32 - 3));
    let view = original.source();
    let decoded = view.text().to_owned();
    let decoded_storage = view.text().as_ptr();
    assert_ne!(decoded, source);
    assert_eq!(
        &decoded[key_span.start as usize..key_span.end as usize],
        spelling
    );
    assert_eq!(
        &decoded[value_span.start as usize..value_span.end as usize],
        "a"
    );
    assert_eq!(view.authored_root(), source);
    assert_eq!(view.authored_root().as_ptr(), source.as_ptr());
    assert_eq!(view.span(), Span::new(0, source.len() as u32));
    let map = view.decode_map().unwrap().segments();
    assert!(!map.is_empty());
    let map_values = map.to_vec();
    let comments = || {
        original
            .comments()
            .map(|comment| {
                (
                    comment.kind(),
                    comment.decoded_span().unwrap(),
                    comment.authored_span().unwrap(),
                    comment.text().unwrap().as_ptr(),
                )
            })
            .collect::<std::vec::Vec<_>>()
    };
    let before_comments = comments();
    // Custody uses the actual authored root, while the key is the decoded AST key.
    assert_eq!(
        expression_document(
            original,
            SourceRoot::new(source).unwrap().whole_block(),
            arena
        )
        .unwrap_err(),
        ExpressionRefusal::UnsupportedNode {
            span: property_span
        }
    );
    assert_eq!(core::ptr::from_ref(original.expression().unwrap()), ast);
    let Expression::ObjectExpression(same) = original.expression().unwrap() else {
        panic!("same original encoded Object")
    };
    assert!(core::ptr::eq(same, object));
    assert_eq!(same.properties.as_ptr(), properties);
    assert_eq!(core::ptr::from_ref(&**property), property_root);
    assert_eq!(core::ptr::from_ref(&property.key), key);
    assert_eq!(core::ptr::from_ref(&property.value), value);
    assert_eq!(name.as_ptr(), key_name);
    assert_eq!(name, "__proto__");
    assert_eq!(original.decoded_span(property.span).unwrap(), property_span);
    assert_eq!(
        original.decoded_span(property.key.span()).unwrap(),
        key_span
    );
    assert_eq!(
        original.decoded_span(property.value.span()).unwrap(),
        value_span
    );
    assert_eq!(
        original.authored_span(property.key.span()).unwrap(),
        authored_key
    );
    assert_eq!(original.source().text(), decoded);
    assert_eq!(original.source().text().as_ptr(), decoded_storage);
    assert_eq!(original.source().authored_root(), source);
    assert_eq!(original.source().authored_root().as_ptr(), source.as_ptr());
    assert_eq!(original.source().span(), view.span());
    assert!(core::ptr::eq(
        original.source().decode_map().unwrap().segments(),
        map
    ));
    assert_eq!(
        original.source().decode_map().unwrap().segments(),
        map_values.as_slice()
    );
    assert_eq!(comments(), before_comments);
    assert_eq!(original.hole(), None);
    assert_eq!(original.diagnostics().count(), 0);
}

#[test]
fn static_key_leaves_and_values_keep_real_depth_sixteen_with_no_property_wrapper_level() {
    for (object_source, printed) in [
        ("{a:b}", "{ a: b }"),
        ("{'a':b}", "{ 'a': b }"),
        ("{1:b}", "{ 1: b }"),
    ] {
        let supported = std::format!("{}{object_source}", "!".repeat(15));
        for lang in [Lang::Js, Lang::Ts] {
            assert_eq!(
                format(&supported, lang, false, options(200, LineEnding::Lf)),
                std::format!("{}{printed}", "! ".repeat(15))
            );
            let arena = Allocator::default();
            let source = std::format!("{}{object_source}", "!".repeat(16));
            let original = retained(
                &arena,
                &source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            assert_eq!(original.hole(), None);
            assert!(original.admitted_expression().is_some());
            let ast = core::ptr::from_ref(original.expression().unwrap());
            let mut expression = original.expression().unwrap();
            for _ in 0..16 {
                let Expression::UnaryExpression(unary) = expression else {
                    panic!("original unary prefix")
                };
                expression = &unary.argument;
            }
            let Expression::ObjectExpression(object) = expression else {
                panic!("original Object at depth sixteen")
            };
            let ObjectPropertyKind::ObjectProperty(property) = &object.properties[0] else {
                panic!("original static property")
            };
            let key_span = original.decoded_span(property.key.span()).unwrap();
            assert_eq!(key_span.start, 17);
            let key = core::ptr::from_ref(&property.key);
            let value = core::ptr::from_ref(&property.value);
            assert_eq!(
                expression_document(
                    &original,
                    SourceRoot::new(&source).unwrap().whole_block(),
                    &arena
                )
                .unwrap_err(),
                ExpressionRefusal::DepthLimit { span: key_span }
            );
            assert_eq!(core::ptr::from_ref(original.expression().unwrap()), ast);
            assert_eq!(core::ptr::from_ref(&property.key), key);
            assert_eq!(core::ptr::from_ref(&property.value), value);
            assert_eq!(original.source().text(), source);
            assert_eq!(original.diagnostics().count(), 0);
        }
        assert_preserved(&supported, false);
    }
    let empty = std::format!("{}{{}}", "!".repeat(16));
    for lang in [Lang::Js, Lang::Ts] {
        assert_eq!(
            format(&empty, lang, false, options(200, LineEnding::Lf)),
            std::format!("{}{{ }}", "! ".repeat(16))
        );
    }
    assert_preserved(&empty, false);
}

#[test]
fn actual_object_syntax_holes_and_non_ascii_property_gaps_preserve_original_observations() {
    for source in [
        "{", "{a:}", "{:b}", "{,}", "{a:b,,}", "{'a' b}", "{a=1}", "{#a:b}", "{a?:b}",
    ] {
        let arena = Allocator::default();
        let original = retained(
            &arena,
            source,
            Span::new(0, source.len() as u32),
            Lang::Js,
            false,
        );
        let hole = original.hole();
        assert!(hole.is_some(), "{source}");
        let ast = original.expression().map(core::ptr::from_ref);
        let diagnostics = original.diagnostics().count();
        assert_eq!(
            expression_document(
                &original,
                SourceRoot::new(source).unwrap().whole_block(),
                &arena
            )
            .unwrap_err(),
            ExpressionRefusal::Unadmitted { hole }
        );
        assert_eq!(original.expression().map(core::ptr::from_ref), ast);
        assert_eq!(original.source().text(), source);
        assert_eq!(original.diagnostics().count(), diagnostics);
    }
    for (source, decode) in [
        ("{\u{a0}a:b}", false),
        ("{a\u{a0}:b}", false),
        ("{a:\u{a0}b}", false),
        ("{a:b\u{a0},c:d}", false),
        ("{a:b,\u{a0}}", false),
        ("{a&#160;:b}", true),
        ("{a:&#160;b}", true),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let arena = Allocator::default();
            let original = retained(
                &arena,
                source,
                Span::new(0, source.len() as u32),
                lang,
                decode,
            );
            assert_eq!(original.hole(), None);
            let ast = core::ptr::from_ref(original.expression().unwrap());
            let text = original.source().text().as_ptr();
            assert!(
                matches!(
                    expression_document(
                        &original,
                        SourceRoot::new(source).unwrap().whole_block(),
                        &arena
                    ),
                    Err(ExpressionRefusal::InvalidGap { .. })
                ),
                "{source}"
            );
            assert_eq!(core::ptr::from_ref(original.expression().unwrap()), ast);
            assert_eq!(original.source().text().as_ptr(), text);
        }
    }
}
