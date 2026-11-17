use super::{assert_own_original_root, selected};
use crate::embed::{Grammar, Lang, Shape};
use crate::markup::{
    ArgSyntax, DirectiveName, DirectivePrefix, NativeAttributeOperandError, NativeConditionKind,
};
use oxc_ast::ast::Expression;
use vize_l0::{Allocator, Span};

#[test]
fn unicode_nonzero_name_block_parts_and_actual_ordinal_preserve_both_original_routes() {
    let arena = Allocator::default();
    let source = "<!--雪🌸-->\r\n<template><p id=x v-else-if='条件'/></template>";
    let old_owner = selected(&arena, source);
    let new_owner = selected(&arena, source);
    let old_element = old_owner.children().next().unwrap().into_element().unwrap();
    let new_element = new_owner.children().next().unwrap().into_element().unwrap();
    let attribute = new_element.attributes().nth(1).unwrap();
    let head = new_owner
        .observe_attribute_head(attribute.reborrow())
        .unwrap();
    assert!(core::ptr::eq(head.selected(), &new_owner));
    assert!(core::ptr::eq(
        head.attribute().surface(),
        attribute.surface()
    ));
    assert!(core::ptr::eq(
        head.attribute().element(),
        new_element.surface()
    ));
    assert_eq!(head.attribute().ordinal(), 1);
    assert_eq!(head.name_block().span(), Span::new(34, 43));
    assert_eq!(head.name_block().source(), "v-else-if");
    assert!(core::ptr::eq(head.name_block().root_source(), source));
    assert!(core::ptr::eq(
        head.name_block().source(),
        attribute.surface().name.text
    ));
    assert_eq!(
        head.directive(),
        Some(DirectiveName {
            prefix: DirectivePrefix::Full,
            name: Span::new(36, 43),
            arg: None,
            modifiers: Span::new(43, 43),
        })
    );
    assert_eq!(head.condition_kind(), Some(NativeConditionKind::ElseIf));

    let old = old_owner
        .observe_attribute_expression(old_element.attributes().nth(1).unwrap())
        .unwrap();
    let new = head.observe_expression().unwrap();
    for operand in [&old, &new] {
        assert_eq!(operand.kind(), NativeConditionKind::ElseIf);
        assert_eq!(operand.name_span(), Span::new(34, 43));
        assert_eq!(operand.value_span(), Span::new(45, 51));
        assert_eq!(operand.raw_value(), "条件");
        assert_eq!(operand.syntax().source().text(), "条件");
        assert_eq!(operand.syntax().source().span(), Span::new(45, 51));
        assert!(operand.syntax().source().decode_map().is_none());
        assert!(core::ptr::eq(
            operand.syntax().source().text(),
            operand.raw_value()
        ));
        assert_eq!(operand.syntax().parser_prefix(), 2);
        assert_eq!(
            operand.syntax().grammar(),
            Grammar {
                shape: Shape::Expr,
                lang: Lang::Js
            }
        );
        assert_eq!(operand.syntax().hole(), None);
        assert_eq!(operand.syntax().comments().count(), 0);
        assert_eq!(operand.syntax().diagnostics().count(), 0);
        let Expression::Identifier(identifier) = operand.syntax().expression().unwrap() else {
            panic!("the actual original Unicode identifier must be retained")
        };
        assert_eq!(identifier.name.as_str(), "条件");
        assert_eq!(identifier.span, oxc_span::Span::new(2, 8));
        assert_eq!(
            operand.syntax().decoded_span(identifier.span),
            Ok(Span::new(0, 6))
        );
        assert_eq!(
            operand.syntax().authored_span(identifier.span),
            Ok(Span::new(45, 51))
        );
    }
    assert_own_original_root(&old, &old_owner, 1);
    assert_own_original_root(&new, &new_owner, 1);
    assert!(!core::ptr::eq(
        old.syntax().expression().unwrap(),
        new.syntax().expression().unwrap()
    ));
}

#[test]
fn plain_static_dynamic_and_all_shorthand_headers_keep_complete_typed_parts() {
    let arena = Allocator::default();
    let source = "<template><p title=x v-bind:雪.prop='x' v-on:[event].once='x' :id='x' .value='x' @click='x' #default='x' v-if='ready'/></template>";
    let old_owner = selected(&arena, source);
    let new_owner = selected(&arena, source);
    let old_element = old_owner.children().next().unwrap().into_element().unwrap();
    let new_element = new_owner.children().next().unwrap().into_element().unwrap();
    let cases = [
        ("title", Span::new(13, 18), None),
        (
            "v-bind:雪.prop",
            Span::new(21, 36),
            Some(DirectiveName {
                prefix: DirectivePrefix::Full,
                name: Span::new(23, 27),
                arg: Some(ArgSyntax::Static(Span::new(28, 31))),
                modifiers: Span::new(31, 36),
            }),
        ),
        (
            "v-on:[event].once",
            Span::new(41, 58),
            Some(DirectiveName {
                prefix: DirectivePrefix::Full,
                name: Span::new(43, 45),
                arg: Some(ArgSyntax::Dynamic(Span::new(47, 52))),
                modifiers: Span::new(53, 58),
            }),
        ),
        (
            ":id",
            Span::new(63, 66),
            Some(DirectiveName {
                prefix: DirectivePrefix::Bind,
                name: Span::new(63, 63),
                arg: Some(ArgSyntax::Static(Span::new(64, 66))),
                modifiers: Span::new(66, 66),
            }),
        ),
        (
            ".value",
            Span::new(71, 77),
            Some(DirectiveName {
                prefix: DirectivePrefix::Prop,
                name: Span::new(71, 71),
                arg: Some(ArgSyntax::Static(Span::new(72, 77))),
                modifiers: Span::new(77, 77),
            }),
        ),
        (
            "@click",
            Span::new(82, 88),
            Some(DirectiveName {
                prefix: DirectivePrefix::On,
                name: Span::new(82, 82),
                arg: Some(ArgSyntax::Static(Span::new(83, 88))),
                modifiers: Span::new(88, 88),
            }),
        ),
        (
            "#default",
            Span::new(93, 101),
            Some(DirectiveName {
                prefix: DirectivePrefix::Slot,
                name: Span::new(93, 93),
                arg: Some(ArgSyntax::Static(Span::new(94, 101))),
                modifiers: Span::new(101, 101),
            }),
        ),
        (
            "v-if",
            Span::new(106, 110),
            Some(DirectiveName {
                prefix: DirectivePrefix::Full,
                name: Span::new(108, 110),
                arg: None,
                modifiers: Span::new(110, 110),
            }),
        ),
    ];
    assert_eq!(old_element.attributes().len(), 8);
    assert_eq!(new_element.attributes().len(), 8);
    for (ordinal, (raw, span, parts)) in cases.into_iter().enumerate() {
        let attribute = new_element.attributes().nth(ordinal).unwrap();
        let before = arena.allocated_bytes();
        let head = new_owner
            .observe_attribute_head(attribute.reborrow())
            .unwrap();
        assert_eq!(arena.allocated_bytes(), before);
        assert_eq!(head.name_block().span(), span);
        assert_eq!(head.name_block().source(), raw);
        assert!(core::ptr::eq(head.name_block().root_source(), source));
        assert!(core::ptr::eq(head.selected(), &new_owner));
        assert!(core::ptr::eq(
            head.attribute().surface(),
            attribute.surface()
        ));
        assert_eq!(head.attribute().ordinal(), ordinal);
        assert_eq!(head.directive(), parts);
        let old =
            old_owner.observe_attribute_expression(old_element.attributes().nth(ordinal).unwrap());
        let new = head.observe_expression();
        if ordinal == 7 {
            assert_eq!(head.condition_kind(), Some(NativeConditionKind::If));
            let old = old.unwrap();
            let new = new.unwrap();
            for operand in [&old, &new] {
                assert_eq!(operand.name_span(), Span::new(106, 110));
                assert_eq!(operand.value_span(), Span::new(112, 117));
                assert_eq!(operand.raw_value(), "ready");
                assert_eq!(operand.syntax().source().text(), "ready");
                assert_eq!(operand.syntax().source().span(), Span::new(112, 117));
                assert_eq!(operand.syntax().hole(), None);
                assert_eq!(operand.syntax().comments().count(), 0);
                assert_eq!(operand.syntax().diagnostics().count(), 0);
                let Expression::Identifier(identifier) = operand.syntax().expression().unwrap()
                else {
                    panic!("the exact conditional header retains its original ready identifier")
                };
                assert_eq!(identifier.name.as_str(), "ready");
                assert_eq!(identifier.span, oxc_span::Span::new(2, 7));
                assert_eq!(
                    operand.syntax().authored_span(identifier.span),
                    Ok(Span::new(112, 117))
                );
            }
            assert_own_original_root(&old, &old_owner, ordinal);
            assert_own_original_root(&new, &new_owner, ordinal);
            assert!(!core::ptr::eq(
                old.syntax().expression().unwrap(),
                new.syntax().expression().unwrap()
            ));
        } else {
            assert_eq!(head.condition_kind(), None);
            for failure in [old.err().unwrap(), new.err().unwrap()] {
                assert_eq!(
                    failure.kind(),
                    NativeAttributeOperandError::UnsupportedDirective
                );
                assert!(failure.syntax().is_none());
            }
        }
    }
}
