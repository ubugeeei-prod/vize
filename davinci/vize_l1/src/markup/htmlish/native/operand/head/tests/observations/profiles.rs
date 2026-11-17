use super::{assert_own_original_root, selected};
use crate::embed::{Grammar, Lang, Shape};
use crate::markup::{NativeConditionKind, NativeTemplateGrammar};
use oxc_ast::ast::{Expression, TSType};
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};

#[test]
fn js_and_setup_ts_profiles_retain_each_routes_own_original_ast_and_spans() {
    let arena = Allocator::default();
    for (source, raw, lang, grammar, value_span) in [
        (
            "前\n<template><p v-if='(value)'/></template>",
            "(value)",
            Lang::Js,
            NativeTemplateGrammar::JavaScriptModule,
            Span::new(23, 30),
        ),
        (
            "前\n<template><p v-if='value as boolean'/></template><script setup lang=ts>const value=true</script>",
            "value as boolean",
            Lang::Ts,
            NativeTemplateGrammar::TypeScriptModule,
            Span::new(23, 39),
        ),
    ] {
        let old_owner = selected(&arena, source);
        let new_owner = selected(&arena, source);
        let old_element = old_owner.children().next().unwrap().into_element().unwrap();
        let new_element = new_owner.children().next().unwrap().into_element().unwrap();
        let head = new_owner
            .observe_attribute_head(new_element.attributes().next().unwrap())
            .unwrap();
        assert_eq!(head.name_block().span(), Span::new(17, 21));
        assert_eq!(head.name_block().source(), "v-if");
        assert_eq!(head.condition_kind(), Some(NativeConditionKind::If));
        let old = old_owner
            .observe_attribute_expression(old_element.attributes().next().unwrap())
            .unwrap();
        let new = head.observe_expression().unwrap();
        assert_eq!(old_owner.grammar(), grammar);
        assert_eq!(new_owner.grammar(), grammar);
        for operand in [&old, &new] {
            let syntax = operand.syntax();
            assert_eq!(operand.name_span(), Span::new(17, 21));
            assert_eq!(operand.value_span(), value_span);
            assert_eq!(operand.raw_value(), raw);
            assert_eq!(syntax.source().span(), value_span);
            assert_eq!(syntax.source().text(), raw);
            assert!(core::ptr::eq(syntax.source().authored_root(), source));
            assert_eq!(
                syntax.grammar(),
                Grammar {
                    shape: Shape::Expr,
                    lang
                }
            );
            assert!(syntax.source_type().is_module());
            assert_eq!(syntax.source_type().is_typescript(), lang == Lang::Ts);
            assert_eq!(syntax.parser_prefix(), 2);
            assert_eq!(syntax.hole(), None);
            assert_eq!(syntax.comments().count(), 0);
            assert_eq!(syntax.diagnostics().count(), 0);
            let root = syntax.expression().unwrap();
            assert_eq!(syntax.authored_span(root.span()), Ok(value_span));
            if lang == Lang::Js {
                let Expression::ParenthesizedExpression(parentheses) = root else {
                    panic!("authored JS parentheses must survive")
                };
                assert_eq!(parentheses.span, oxc_span::Span::new(2, 9));
                assert_eq!(syntax.decoded_span(parentheses.span), Ok(Span::new(0, 7)));
                let Expression::Identifier(identifier) = &parentheses.expression else {
                    panic!("the original JS child must be an identifier")
                };
                assert_eq!(identifier.name.as_str(), "value");
                assert_eq!(identifier.span, oxc_span::Span::new(3, 8));
                assert_eq!(syntax.authored_span(identifier.span), Ok(Span::new(24, 29)));
            } else {
                let Expression::TSAsExpression(cast) = root else {
                    panic!("the intrinsically selected TS cast must survive")
                };
                assert_eq!(cast.span, oxc_span::Span::new(2, 18));
                assert_eq!(syntax.decoded_span(cast.span), Ok(Span::new(0, 16)));
                let Expression::Identifier(identifier) = &cast.expression else {
                    panic!("the original TS child must be an identifier")
                };
                assert_eq!(identifier.name.as_str(), "value");
                assert_eq!(identifier.span, oxc_span::Span::new(2, 7));
                assert_eq!(syntax.authored_span(identifier.span), Ok(Span::new(23, 28)));
                let TSType::TSBooleanKeyword(keyword) = &cast.type_annotation else {
                    panic!("the original erased type must be boolean")
                };
                assert_eq!(keyword.span, oxc_span::Span::new(11, 18));
                assert_eq!(syntax.authored_span(keyword.span), Ok(Span::new(32, 39)));
            }
        }
        assert_own_original_root(&old, &old_owner, 0);
        assert_own_original_root(&new, &new_owner, 0);
        assert!(!core::ptr::eq(
            old.syntax().expression().unwrap(),
            new.syntax().expression().unwrap()
        ));
    }
}
