use super::{assert_output, options};
use oxc_ast::ast::Expression;
use vize_glyph::native_doc::LineEnding;
use vize_l0::{Allocator, Span};
use vize_l1::embed::DecodeSegmentKind;
use vize_l1::markup::NativeConditionKind;

mod expressions;
mod geometry;

#[test]
fn mixed_interpolations_and_both_conditionals_keep_actual_original_source_order() {
    let source = "<template>{{1n}}<p v-if='a+b' id=x>{{c+d}}</p><p v-else-if=\"ready&&ok\">{{2n}}</p></template>";
    let expected = "<template>{{ 1n }}<p v-if='a + b' id=x>{{ c + d }}</p><p v-else-if=\"ready && ok\">{{ 2n }}</p></template>";
    let arena = Allocator::default();
    let owner = assert_output(&arena, source, expected, options(200, 2, LineEnding::Lf));
    assert_eq!(
        owner
            .operands()
            .iter()
            .map(|operand| operand.raw_content())
            .collect::<Vec<_>>(),
        ["1n", "c+d", "2n"]
    );
    assert_eq!(
        owner
            .attribute_operands()
            .iter()
            .map(|operand| (
                operand.kind(),
                operand.name_span(),
                operand.value_span(),
                operand.raw_value(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                NativeConditionKind::If,
                Span::new(19, 23),
                Span::new(25, 28),
                "a+b"
            ),
            (
                NativeConditionKind::ElseIf,
                Span::new(49, 58),
                Span::new(60, 69),
                "ready&&ok"
            ),
        ]
    );
    let Expression::BinaryExpression(first) =
        owner.attribute_operands()[0].syntax().expression().unwrap()
    else {
        panic!("the first original header has its own binary root")
    };
    assert_eq!(first.operator.as_str(), "+");
    let Expression::LogicalExpression(second) =
        owner.attribute_operands()[1].syntax().expression().unwrap()
    else {
        panic!("the second original header has its own logical root")
    };
    assert_eq!(second.operator.as_str(), "&&");
}

#[test]
fn original_numeric_entity_operator_outer_tag_and_tail_spelling_survive_whole_replay() {
    let source = "<!--前-->\r\n<template lang='html'><p title='raw &amp; text' v-if='(0xCA_FE+1_000)&amp;&amp;true'>{{1&#110;}}</p></template><!--尾-->\r\n";
    let expected = "<!--前-->\r\n<template lang='html'><p title='raw &amp; text' v-if='(0xCA_FE + 1_000) &amp;&amp; true'>{{ 1&#110; }}</p></template><!--尾-->\r\n";
    for ending in [LineEnding::Lf, LineEnding::CrLf] {
        let arena = Allocator::default();
        let owner = assert_output(&arena, source, expected, options(200, 2, ending));
        assert_eq!(owner.operands().len(), 1);
        assert_eq!(owner.attribute_operands().len(), 1);
        let operand = &owner.attribute_operands()[0];
        assert_eq!(operand.name_span(), Span::new(60, 64));
        assert_eq!(operand.value_span(), Span::new(66, 95));
        assert_eq!(operand.raw_value(), "(0xCA_FE+1_000)&amp;&amp;true");
        let syntax = operand.syntax();
        assert_eq!(syntax.source().text(), "(0xCA_FE+1_000)&&true");
        assert_eq!(
            syntax
                .source()
                .decode_map()
                .unwrap()
                .segments()
                .iter()
                .map(|segment| (segment.decoded(), segment.authored(), segment.kind(),))
                .collect::<Vec<_>>(),
            [
                (
                    Span::new(0, 15),
                    Span::new(66, 81),
                    DecodeSegmentKind::Identity
                ),
                (
                    Span::new(15, 16),
                    Span::new(81, 86),
                    DecodeSegmentKind::Entity
                ),
                (
                    Span::new(16, 17),
                    Span::new(86, 91),
                    DecodeSegmentKind::Entity
                ),
                (
                    Span::new(17, 21),
                    Span::new(91, 95),
                    DecodeSegmentKind::Identity
                ),
            ]
        );
        let Expression::LogicalExpression(logical) = syntax.expression().unwrap() else {
            panic!("original &&")
        };
        assert_eq!(logical.operator.as_str(), "&&");
        assert_eq!(logical.span, oxc_span::Span::new(2, 23));
        let Expression::ParenthesizedExpression(parentheses) = &logical.left else {
            panic!("authored parentheses")
        };
        let Expression::BinaryExpression(binary) = &parentheses.expression else {
            panic!("original binary")
        };
        assert_eq!(binary.operator.as_str(), "+");
        for (child, value, authored) in [
            (&binary.left, 51_966.0_f64, "0xCA_FE"),
            (&binary.right, 1_000.0_f64, "1_000"),
        ] {
            let Expression::NumericLiteral(number) = child else {
                panic!("original numeric atom")
            };
            assert_eq!(number.value.to_bits(), value.to_bits());
            assert_eq!(
                syntax.authored_span(number.span).unwrap().slice(source),
                authored
            );
        }
        assert_eq!(owner.operands()[0].raw_content(), "1&#110;");
        assert_eq!(owner.operands()[0].syntax().source().text(), "1n");
        assert_eq!(owner.operands()[0].content_span(), Span::new(99, 106));
    }
}

#[test]
fn attribute_ambiguous_references_keep_once_decoded_string_and_exact_authored_value() {
    let source = "<template><p v-if='\"&amp=1 &notit; &amp;lt;\"'/></template>";
    let expected = "<template><p v-if='\"&amp=1 &notit; &amp;lt;\"' /></template>";
    let arena = Allocator::default();
    let owner = assert_output(&arena, source, expected, options(200, 2, LineEnding::Lf));
    assert!(owner.operands().is_empty());
    assert_eq!(owner.attribute_operands().len(), 1);
    let operand = &owner.attribute_operands()[0];
    assert_eq!(operand.name_span(), Span::new(13, 17));
    assert_eq!(operand.value_span(), Span::new(19, 44));
    assert_eq!(operand.raw_value(), "\"&amp=1 &notit; &amp;lt;\"");
    let syntax = operand.syntax();
    assert_eq!(syntax.source().text(), "\"&amp=1 &notit; &lt;\"");
    assert_eq!(
        syntax
            .source()
            .decode_map()
            .unwrap()
            .segments()
            .iter()
            .map(|segment| (segment.decoded(), segment.authored(), segment.kind(),))
            .collect::<Vec<_>>(),
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
        panic!("actual attribute string")
    };
    assert_eq!(literal.value.as_str(), "&amp=1 &notit; &lt;");
    assert_eq!(literal.span, oxc_span::Span::new(2, 23));
    assert_eq!(syntax.authored_span(literal.span), Ok(Span::new(19, 44)));
}

#[test]
fn original_quotes_attribute_order_and_outer_bytes_remain_whole_source_custody() {
    let source = "\u{feff}<!--pre-->\r\n<template><p z = 'raw' v-else-if = \" a+b \" a='second'/></template><!--post-->\r\n";
    let expected = "\u{feff}<!--pre-->\r\n<template><p z='raw' v-else-if=\"a + b\" a='second' /></template><!--post-->\r\n";
    let arena = Allocator::default();
    let owner = assert_output(&arena, source, expected, options(200, 2, LineEnding::CrLf));
    assert!(owner.operands().is_empty());
    assert_eq!(owner.attribute_operands().len(), 1);
    let element = owner
        .selected()
        .unwrap()
        .children()
        .next()
        .unwrap()
        .into_element()
        .unwrap();
    assert_eq!(
        element
            .attributes()
            .map(|attribute| (
                attribute.surface().name.text,
                attribute
                    .surface()
                    .value
                    .as_ref()
                    .unwrap()
                    .open_quote
                    .as_ref()
                    .unwrap()
                    .text,
                attribute.surface().value.as_ref().unwrap().content.text,
                attribute
                    .surface()
                    .value
                    .as_ref()
                    .unwrap()
                    .close_quote
                    .as_ref()
                    .unwrap()
                    .text,
            ))
            .collect::<Vec<_>>(),
        [
            ("z", "'", "raw", "'"),
            ("v-else-if", "\"", " a+b ", "\""),
            ("a", "'", "second", "'")
        ]
    );
    assert_eq!(owner.attribute_operands()[0].raw_value(), " a+b ");
    assert_eq!(
        owner.attribute_operands()[0].kind(),
        NativeConditionKind::ElseIf
    );
}
