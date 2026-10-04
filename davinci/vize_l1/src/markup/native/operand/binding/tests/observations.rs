use super::{assert_original, observe, selected};
use crate::embed::{Grammar, Lang, Shape};
use crate::markup::{
    ArgSyntax, DirectiveName, DirectivePrefix, NativeConditionKind, NativeTemplateGrammar,
};
use oxc_ast::ast::{Expression, TSType};
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};

#[test]
fn unicode_nonzero_static_heads_keep_original_parts_ordinals_and_distinct_stock_roots() {
    let arena = Allocator::default();
    let source = "<!--é--><template><p title=x v-bind:雪='条件' :data-id='ready' v-if='ok'/></template><!--尾-->";
    let first = selected(&arena, source);
    let second = selected(&arena, source);
    let mut roots = alloc::vec::Vec::new();
    for owner in [&first, &second] {
        assert_eq!(owner.component().block().start(), 19);
        assert_eq!(
            owner.component().block().source(),
            "<p title=x v-bind:雪='条件' :data-id='ready' v-if='ok'/>"
        );
        let element = owner.children().next().unwrap().into_element().unwrap();
        assert_eq!(element.attributes().len(), 4);
        let plain = owner
            .observe_attribute_head(element.attributes().next().unwrap())
            .unwrap();
        assert_eq!(plain.directive(), None);
        assert!(plain.static_binding().unwrap().is_none());
        for (ordinal, name, argument, value, raw, parts) in [
            (
                1,
                Span::new(30, 40),
                Span::new(37, 40),
                Span::new(42, 48),
                "条件",
                DirectiveName {
                    prefix: DirectivePrefix::Full,
                    name: Span::new(32, 36),
                    arg: Some(ArgSyntax::Static(Span::new(37, 40))),
                    modifiers: Span::new(40, 40),
                },
            ),
            (
                2,
                Span::new(50, 58),
                Span::new(51, 58),
                Span::new(60, 65),
                "ready",
                DirectiveName {
                    prefix: DirectivePrefix::Bind,
                    name: Span::new(50, 50),
                    arg: Some(ArgSyntax::Static(Span::new(51, 58))),
                    modifiers: Span::new(58, 58),
                },
            ),
        ] {
            let attribute = element.attributes().nth(ordinal).unwrap();
            let before = arena.allocated_bytes();
            let head = owner.observe_attribute_head(attribute.reborrow()).unwrap();
            let binding = head.static_binding().unwrap().unwrap();
            assert_eq!(arena.allocated_bytes(), before);
            assert!(core::ptr::eq(binding.head(), &head));
            assert!(core::ptr::eq(head.selected(), owner));
            assert!(core::ptr::eq(
                head.attribute().surface(),
                attribute.surface()
            ));
            assert_eq!(head.attribute().ordinal(), ordinal);
            assert_eq!(head.name_block().span(), name);
            assert_eq!(head.name_block().source(), name.slice(source));
            assert_eq!(head.directive(), Some(parts));
            assert_eq!(head.condition_kind(), None);
            assert_eq!(binding.argument_span(), argument);
            let operand = binding.observe_expression().unwrap();
            assert_eq!(operand.name_span(), name);
            assert_eq!(operand.argument_span(), argument);
            assert_eq!(operand.value_span(), value);
            assert_eq!(operand.raw_value(), raw);
            let syntax = operand.syntax();
            assert_eq!(syntax.source().span(), value);
            assert_eq!(syntax.source().text(), raw);
            assert!(core::ptr::eq(syntax.source().text(), operand.raw_value()));
            assert!(syntax.source().decode_map().is_none());
            assert_eq!(syntax.parser_prefix(), 2);
            assert_eq!(syntax.hole(), None);
            assert_eq!(syntax.comments().count(), 0);
            assert_eq!(syntax.diagnostics().count(), 0);
            let Expression::Identifier(identifier) = syntax.expression().unwrap() else {
                panic!("original static binding identifier")
            };
            assert_eq!(identifier.name.as_str(), raw);
            let end = if ordinal == 1 { 8 } else { 7 };
            assert_eq!(identifier.span, oxc_span::Span::new(2, end));
            assert_eq!(
                syntax.decoded_span(identifier.span),
                Ok(Span::new(0, end - 2))
            );
            assert_eq!(syntax.authored_span(identifier.span), Ok(value));
            assert_original(&operand, owner, ordinal);
            roots.push(syntax.expression().unwrap());
        }
        let conditional = owner
            .observe_attribute_head(element.attributes().nth(3).unwrap())
            .unwrap();
        assert_eq!(conditional.name_block().span(), Span::new(67, 71));
        assert_eq!(conditional.condition_kind(), Some(NativeConditionKind::If));
        assert!(conditional.static_binding().unwrap().is_none());
    }
    assert_eq!(roots.len(), 4);
    assert!(!core::ptr::eq(roots[0], roots[2]));
    assert!(!core::ptr::eq(roots[1], roots[3]));
}

#[test]
fn intrinsic_js_and_setup_ts_profiles_preserve_original_parentheses_cast_and_child_spans() {
    let arena = Allocator::default();
    for (source, raw, name, argument, value, lang, grammar) in [
        (
            "前\n<template><p :id='(value)'/></template>",
            "(value)",
            Span::new(17, 20),
            Span::new(18, 20),
            Span::new(22, 29),
            Lang::Js,
            NativeTemplateGrammar::JavaScriptModule,
        ),
        (
            "前\n<template><p v-bind:id='value as boolean'/></template><script setup lang=ts>const value=true</script>",
            "value as boolean",
            Span::new(17, 26),
            Span::new(24, 26),
            Span::new(28, 44),
            Lang::Ts,
            NativeTemplateGrammar::TypeScriptModule,
        ),
    ] {
        let owner = selected(&arena, source);
        assert_eq!(owner.grammar(), grammar);
        let operand = observe(&owner, 0);
        assert_eq!(operand.name_span(), name);
        assert_eq!(operand.argument_span(), argument);
        assert_eq!(operand.value_span(), value);
        assert_eq!(operand.raw_value(), raw);
        let syntax = operand.syntax();
        assert_eq!(syntax.source().text(), raw);
        assert_eq!(syntax.source().span(), value);
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
        assert_eq!(syntax.authored_span(root.span()), Ok(value));
        if lang == Lang::Js {
            let Expression::ParenthesizedExpression(parentheses) = root else {
                panic!("original JS parentheses")
            };
            assert_eq!(parentheses.span, oxc_span::Span::new(2, 9));
            let Expression::Identifier(identifier) = &parentheses.expression else {
                panic!("original JS child")
            };
            assert_eq!(identifier.name.as_str(), "value");
            assert_eq!(identifier.span, oxc_span::Span::new(3, 8));
            assert_eq!(syntax.authored_span(identifier.span), Ok(Span::new(23, 28)));
        } else {
            let Expression::TSAsExpression(cast) = root else {
                panic!("intrinsic TS cast")
            };
            assert_eq!(cast.span, oxc_span::Span::new(2, 18));
            let Expression::Identifier(identifier) = &cast.expression else {
                panic!("original TS value")
            };
            assert_eq!(identifier.name.as_str(), "value");
            assert_eq!(identifier.span, oxc_span::Span::new(2, 7));
            assert_eq!(syntax.authored_span(identifier.span), Ok(Span::new(28, 33)));
            let TSType::TSBooleanKeyword(keyword) = &cast.type_annotation else {
                panic!("original TS type")
            };
            assert_eq!(keyword.span, oxc_span::Span::new(11, 18));
            assert_eq!(syntax.authored_span(keyword.span), Ok(Span::new(37, 44)));
        }
        assert_original(&operand, &owner, 0);
    }
}

#[test]
fn complete_unquoted_values_and_non_identifier_static_names_remain_honest_l1_receipts() {
    let arena = Allocator::default();
    let source = "<template><p :data-id=a+b v-bind:xlink:href='ready'/></template>";
    let owner = selected(&arena, source);
    for (ordinal, name, argument, value, raw, argument_text) in [
        (
            0,
            Span::new(13, 21),
            Span::new(14, 21),
            Span::new(22, 25),
            "a+b",
            "data-id",
        ),
        (
            1,
            Span::new(26, 43),
            Span::new(33, 43),
            Span::new(45, 50),
            "ready",
            "xlink:href",
        ),
    ] {
        let operand = observe(&owner, ordinal);
        assert_eq!(operand.name_span(), name);
        assert_eq!(operand.argument_span(), argument);
        assert_eq!(argument.slice(source), argument_text);
        assert_eq!(operand.value_span(), value);
        assert_eq!(operand.raw_value(), raw);
        assert_eq!(operand.syntax().source().text(), raw);
        assert_eq!(operand.syntax().hole(), None);
        assert_eq!(operand.syntax().diagnostics().count(), 0);
        assert_eq!(operand.syntax().comments().count(), 0);
        if ordinal == 0 {
            let Expression::BinaryExpression(binary) = operand.syntax().expression().unwrap()
            else {
                panic!("original complete unquoted binary")
            };
            assert_eq!(binary.operator.as_str(), "+");
            assert_eq!(binary.span, oxc_span::Span::new(2, 5));
            for (child, name, stock, authored) in [
                (
                    &binary.left,
                    "a",
                    oxc_span::Span::new(2, 3),
                    Span::new(22, 23),
                ),
                (
                    &binary.right,
                    "b",
                    oxc_span::Span::new(4, 5),
                    Span::new(24, 25),
                ),
            ] {
                let Expression::Identifier(identifier) = child else {
                    panic!("original unquoted child")
                };
                assert_eq!(identifier.name.as_str(), name);
                assert_eq!(identifier.span, stock);
                assert_eq!(
                    operand.syntax().authored_span(identifier.span),
                    Ok(authored)
                );
            }
        } else {
            let Expression::Identifier(identifier) = operand.syntax().expression().unwrap() else {
                panic!("original quoted ready identifier")
            };
            assert_eq!(identifier.name.as_str(), "ready");
            assert_eq!(identifier.span, oxc_span::Span::new(2, 7));
        }
        assert_eq!(
            operand
                .syntax()
                .authored_span(operand.syntax().expression().unwrap().span()),
            Ok(value)
        );
        assert_original(&operand, &owner, ordinal);
    }
    let element = owner.children().next().unwrap().into_element().unwrap();
    let unquoted = element.attributes().next().unwrap();
    let frame = unquoted.surface().value.as_ref().unwrap();
    assert!(frame.open_quote.is_none());
    assert!(frame.close_quote.is_none());
    assert_eq!(frame.content.text, "a+b");
}
