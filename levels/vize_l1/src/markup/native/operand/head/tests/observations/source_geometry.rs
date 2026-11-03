use super::{assert_own_original_root, selected};
use crate::embed::DecodeSegmentKind;
use alloc::vec::Vec;
use oxc_ast::ast::{CommentKind, Expression};
use vize_l0::{Allocator, Span};

#[test]
fn complete_entity_maps_and_physical_lf_crlf_comments_keep_authored_geometry() {
    let arena = Allocator::default();
    for (source, raw, decoded, name_span, value_span, shift) in [
        (
            "前\n<template><p v-if='/*雪*/\n(α &amp;&amp; β) //尾\n'/></template>",
            "/*雪*/\n(α &amp;&amp; β) //尾\n",
            "/*雪*/\n(α && β) //尾\n",
            Span::new(17, 21),
            Span::new(23, 56),
            0,
        ),
        (
            "前\r\n<template><p v-if='/*雪*/\r\n(α &amp;&amp; β) //尾\r\n'/></template>",
            "/*雪*/\r\n(α &amp;&amp; β) //尾\r\n",
            "/*雪*/\r\n(α && β) //尾\r\n",
            Span::new(18, 22),
            Span::new(24, 59),
            1,
        ),
    ] {
        // Each tuple gives the entire stream; CRLF adds one byte before the
        // template, one before the expression, and one at the value's end.
        let expected_map = if shift == 0 {
            [
                (
                    Span::new(0, 12),
                    Span::new(23, 35),
                    DecodeSegmentKind::Identity,
                ),
                (
                    Span::new(12, 13),
                    Span::new(35, 40),
                    DecodeSegmentKind::Entity,
                ),
                (
                    Span::new(13, 14),
                    Span::new(40, 45),
                    DecodeSegmentKind::Entity,
                ),
                (
                    Span::new(14, 25),
                    Span::new(45, 56),
                    DecodeSegmentKind::Identity,
                ),
            ]
        } else {
            [
                (
                    Span::new(0, 13),
                    Span::new(24, 37),
                    DecodeSegmentKind::Identity,
                ),
                (
                    Span::new(13, 14),
                    Span::new(37, 42),
                    DecodeSegmentKind::Entity,
                ),
                (
                    Span::new(14, 15),
                    Span::new(42, 47),
                    DecodeSegmentKind::Entity,
                ),
                (
                    Span::new(15, 27),
                    Span::new(47, 59),
                    DecodeSegmentKind::Identity,
                ),
            ]
        };
        let expected_comments = if shift == 0 {
            [
                (
                    CommentKind::SingleLineBlock,
                    Span::new(0, 7),
                    Span::new(23, 30),
                    "/*雪*/",
                ),
                (
                    CommentKind::Line,
                    Span::new(19, 24),
                    Span::new(50, 55),
                    "//尾",
                ),
            ]
        } else {
            [
                (
                    CommentKind::SingleLineBlock,
                    Span::new(0, 7),
                    Span::new(24, 31),
                    "/*雪*/",
                ),
                (
                    CommentKind::Line,
                    Span::new(20, 25),
                    Span::new(52, 57),
                    "//尾",
                ),
            ]
        };
        let old_owner = selected(&arena, source);
        let new_owner = selected(&arena, source);
        let old_element = old_owner.children().next().unwrap().into_element().unwrap();
        let new_element = new_owner.children().next().unwrap().into_element().unwrap();
        let head = new_owner
            .observe_attribute_head(new_element.attributes().next().unwrap())
            .unwrap();
        assert_eq!(head.name_block().span(), name_span);
        assert_eq!(head.name_block().source(), "v-if");
        let old = old_owner
            .observe_attribute_expression(old_element.attributes().next().unwrap())
            .unwrap();
        let new = head.observe_expression().unwrap();
        for operand in [&old, &new] {
            let syntax = operand.syntax();
            assert_eq!(operand.name_span(), name_span);
            assert_eq!(operand.value_span(), value_span);
            assert_eq!(operand.raw_value(), raw);
            assert_eq!(value_span.slice(source), raw);
            assert_eq!(syntax.source().span(), value_span);
            assert_eq!(syntax.source().text(), decoded);
            assert!(core::ptr::eq(syntax.source().authored_root(), source));
            assert_eq!(syntax.hole(), None);
            assert_eq!(syntax.parser_prefix(), 2);
            assert_eq!(syntax.diagnostics().count(), 0);
            let actual_map: Vec<_> = syntax
                .source()
                .decode_map()
                .unwrap()
                .segments()
                .iter()
                .map(|segment| (segment.decoded(), segment.authored(), segment.kind()))
                .collect();
            assert_eq!(actual_map, expected_map);
            for (decoded_span, authored_span, _) in expected_map {
                assert_eq!(
                    syntax.source().authored_span(decoded_span),
                    Ok(authored_span)
                );
            }
            let comments: Vec<_> = syntax
                .comments()
                .map(|comment| {
                    (
                        comment.kind(),
                        comment.decoded_span().unwrap(),
                        comment.authored_span().unwrap(),
                        comment.text().unwrap(),
                    )
                })
                .collect();
            assert_eq!(comments, expected_comments);
            let Expression::ParenthesizedExpression(parentheses) = syntax.expression().unwrap()
            else {
                panic!("the original authored parentheses must remain")
            };
            assert_eq!(
                parentheses.span,
                oxc_span::Span::new(10 + shift, 20 + shift)
            );
            assert_eq!(
                syntax.decoded_span(parentheses.span),
                Ok(Span::new(8 + shift, 18 + shift))
            );
            assert_eq!(
                syntax.authored_span(parentheses.span),
                Ok(Span::new(31 + 2 * shift, 49 + 2 * shift))
            );
            let Expression::LogicalExpression(logical) = &parentheses.expression else {
                panic!("the once-decoded && must form the original logical expression")
            };
            assert_eq!(logical.operator.as_str(), "&&");
            assert_eq!(logical.span, oxc_span::Span::new(11 + shift, 19 + shift));
            assert_eq!(
                syntax.authored_span(logical.span),
                Ok(Span::new(32 + 2 * shift, 48 + 2 * shift))
            );
            for (child, name, parser_span, authored_span) in [
                (
                    &logical.left,
                    "α",
                    oxc_span::Span::new(11 + shift, 13 + shift),
                    Span::new(32 + 2 * shift, 34 + 2 * shift),
                ),
                (
                    &logical.right,
                    "β",
                    oxc_span::Span::new(17 + shift, 19 + shift),
                    Span::new(46 + 2 * shift, 48 + 2 * shift),
                ),
            ] {
                let Expression::Identifier(identifier) = child else {
                    panic!("original Unicode child")
                };
                assert_eq!(identifier.name.as_str(), name);
                assert_eq!(identifier.span, parser_span);
                assert_eq!(syntax.authored_span(identifier.span), Ok(authored_span));
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

#[test]
fn ambiguous_attribute_references_remain_literal_and_decoded_output_is_never_redecoded() {
    let arena = Allocator::default();
    let source = "<template><p v-if='\"&amp=1 &notit; &amp;lt;\"'/></template>";
    let old_owner = selected(&arena, source);
    let new_owner = selected(&arena, source);
    let old_element = old_owner.children().next().unwrap().into_element().unwrap();
    let new_element = new_owner.children().next().unwrap().into_element().unwrap();
    let head = new_owner
        .observe_attribute_head(new_element.attributes().next().unwrap())
        .unwrap();
    assert_eq!(head.name_block().span(), Span::new(13, 17));
    assert_eq!(head.name_block().source(), "v-if");
    let old = old_owner
        .observe_attribute_expression(old_element.attributes().next().unwrap())
        .unwrap();
    let new = head.observe_expression().unwrap();
    for operand in [&old, &new] {
        let syntax = operand.syntax();
        assert_eq!(operand.raw_value(), "\"&amp=1 &notit; &amp;lt;\"");
        assert_eq!(operand.name_span(), Span::new(13, 17));
        assert_eq!(operand.value_span(), Span::new(19, 44));
        assert_eq!(syntax.source().span(), Span::new(19, 44));
        assert_eq!(syntax.source().text(), "\"&amp=1 &notit; &lt;\"");
        assert!(core::ptr::eq(syntax.source().authored_root(), source));
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.comments().count(), 0);
        assert_eq!(syntax.diagnostics().count(), 0);
        let actual_map: Vec<_> = syntax
            .source()
            .decode_map()
            .unwrap()
            .segments()
            .iter()
            .map(|segment| (segment.decoded(), segment.authored(), segment.kind()))
            .collect();
        assert_eq!(
            actual_map,
            [
                (
                    Span::new(0, 16),
                    Span::new(19, 35),
                    DecodeSegmentKind::Identity
                ),
                (
                    Span::new(16, 17),
                    Span::new(35, 40),
                    DecodeSegmentKind::Entity
                ),
                (
                    Span::new(17, 21),
                    Span::new(40, 44),
                    DecodeSegmentKind::Identity
                ),
            ]
        );
        let Expression::StringLiteral(literal) = syntax.expression().unwrap() else {
            panic!("the complete decoded original input must remain a string literal")
        };
        assert_eq!(literal.value.as_str(), "&amp=1 &notit; &lt;");
        assert_eq!(
            literal.raw.as_ref().unwrap().as_str(),
            "\"&amp=1 &notit; &lt;\""
        );
        assert_eq!(literal.span, oxc_span::Span::new(2, 23));
        assert_eq!(syntax.decoded_span(literal.span), Ok(Span::new(0, 21)));
        assert_eq!(syntax.authored_span(literal.span), Ok(Span::new(19, 44)));
    }
    assert_own_original_root(&old, &old_owner, 0);
    assert_own_original_root(&new, &new_owner, 0);
    assert!(!core::ptr::eq(
        old.syntax().expression().unwrap(),
        new.syntax().expression().unwrap()
    ));
}
