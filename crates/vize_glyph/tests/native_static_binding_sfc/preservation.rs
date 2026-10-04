use super::{assert_output, options};
use vize_glyph::native_doc::LineEnding;
use vize_l0::{Allocator, Span};
use vize_l1::embed::DecodeSegmentKind;

#[test]
fn complete_mixed_families_keep_quote_order_outer_bytes_and_independent_observation_ordinals() {
    let arena = Allocator::default();
    let source = "<!--前-->\n<template><p :id='a+b' v-if=\"ok\" v-bind:title='x&#43;1'>{{n*2}}</p></template><!--尾-->";
    let expected = "<!--前-->\n<template><p :id='a + b' v-if=\"ok\" v-bind:title='x &#43; 1'>{{ n * 2 }}</p></template><!--尾-->";
    let owner = assert_output(&arena, source, expected, options(200, 2, LineEnding::Lf));
    assert_eq!(owner.binding_operands().len(), 2);
    assert_eq!(owner.attribute_operands().len(), 1);
    assert_eq!(owner.operands().len(), 1);
    for (operand, name, argument, value, raw) in [
        (
            &owner.binding_operands()[0],
            Span::new(24, 27),
            Span::new(25, 27),
            Span::new(29, 32),
            "a+b",
        ),
        (
            &owner.binding_operands()[1],
            Span::new(44, 56),
            Span::new(51, 56),
            Span::new(58, 65),
            "x&#43;1",
        ),
    ] {
        assert_eq!(operand.name_span(), name);
        assert_eq!(operand.argument_span(), argument);
        assert_eq!(operand.value_span(), value);
        assert_eq!(operand.raw_value(), raw);
    }
    assert_eq!(owner.attribute_operands()[0].name_span(), Span::new(34, 38));
    assert_eq!(
        owner.attribute_operands()[0].value_span(),
        Span::new(40, 42)
    );
}

#[test]
fn unicode_static_argument_and_complete_entity_map_keep_nonzero_authored_coordinates() {
    let arena = Allocator::default();
    let source = "<!--é--><template><p v-bind:雪='n&#43;1' :id=\"x+y\"/></template><!--z-->";
    let expected = "<!--é--><template><p v-bind:雪='n &#43; 1' :id=\"x + y\" /></template><!--z-->";
    let owner = assert_output(&arena, source, expected, options(200, 2, LineEnding::Lf));
    assert_eq!(owner.selected().unwrap().component().block().start(), 19);
    assert_eq!(owner.binding_operands().len(), 2);
    let first = &owner.binding_operands()[0];
    assert_eq!(first.name_span(), Span::new(22, 32));
    assert_eq!(first.argument_span(), Span::new(29, 32));
    assert_eq!(first.argument_span().slice(source), "雪");
    assert_eq!(first.value_span(), Span::new(34, 41));
    assert_eq!(first.raw_value(), "n&#43;1");
    assert_eq!(first.syntax().source().text(), "n+1");
    let map: Vec<_> = first
        .syntax()
        .source()
        .decode_map()
        .unwrap()
        .segments()
        .iter()
        .map(|segment| (segment.decoded(), segment.authored(), segment.kind()))
        .collect();
    assert_eq!(
        map,
        [
            (
                Span::new(0, 1),
                Span::new(34, 35),
                DecodeSegmentKind::Identity
            ),
            (
                Span::new(1, 2),
                Span::new(35, 40),
                DecodeSegmentKind::Entity
            ),
            (
                Span::new(2, 3),
                Span::new(40, 41),
                DecodeSegmentKind::Identity
            ),
        ]
    );
    let second = &owner.binding_operands()[1];
    assert_eq!(second.name_span(), Span::new(43, 46));
    assert_eq!(second.argument_span(), Span::new(44, 46));
    assert_eq!(second.value_span(), Span::new(48, 51));
    assert_eq!(second.raw_value(), "x+y");
    assert!(second.syntax().source().decode_map().is_none());
}

#[test]
fn explicit_conditionals_full_bindings_and_original_literal_spellings_share_the_whole_doc() {
    for (source, expected) in [
        (
            "<template><p v-if='a+b' :id='c+d'>{{1n}}</p><p v-else-if='e+f' v-bind:id='g+h'>{{2n}}</p></template>",
            "<template><p v-if='a + b' :id='c + d'>{{ 1n }}</p><p v-else-if='e + f' v-bind:id='g + h'>{{ 2n }}</p></template>",
        ),
        (
            "<template><p :id='0x10+1_000+2n' v-bind:title='\"雪\"'/></template>",
            "<template><p :id='0x10 + 1_000 + 2n' v-bind:title='\"雪\"' /></template>",
        ),
    ] {
        let arena = Allocator::default();
        let owner = assert_output(&arena, source, expected, options(200, 2, LineEnding::Lf));
        assert_eq!(owner.binding_operands().len(), 2);
    }
}
