use super::{Vue2HoleKind, lower_vue2_component};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::dialect::vue2::surface::{parse_component, parse_component_block};
use vize_l2::{expr::ExprRef, op::Op};

#[test]
fn ordinary_html_uses_the_shared_arena_ops_and_decoded_static_attributes() {
    let arena = Allocator::default();
    let source =
        "<div id='雪&amp;冬' class='a b'><span>Hello &lt;世界&gt;</span>{{ count + 1 }}</div>";
    let original = parse_component(&arena, source)
        .unwrap()
        .into_expression_pool();
    let root = original
        .bindings()
        .first()
        .unwrap()
        .admitted()
        .unwrap()
        .base()
        .expression()
        .unwrap();
    let observation = lower_vue2_component(&arena, original);
    let artifact = observation.admitted().unwrap();
    assert!(core::ptr::eq(artifact.source(), source));
    assert_eq!(artifact.node_count(), 4);
    let [Op::Element(element)] = artifact.root().ops.as_slice() else {
        panic!("shared HTML op")
    };
    assert_eq!(element.tag, "div");
    assert_eq!(element.attributes.first().unwrap().value, Some("雪&冬"));
    let [Op::Element(span), Op::Interpolation(interpolation)] = element.children.ops.as_slice()
    else {
        panic!("original child order")
    };
    let [Op::Text(text)] = span.children.ops.as_slice() else {
        panic!("text leaf")
    };
    assert_eq!(text.content, "Hello <世界>");
    let ExprRef::Js(expression) = interpolation.expression else {
        panic!("original retained root")
    };
    assert!(core::ptr::eq(expression.ast, root));
    assert_eq!(expression.source, "count + 1");
    assert!(expression.matches_authored_source(source));
}

#[test]
fn original_full_file_coordinates_and_entity_projection_survive_the_handoff() {
    let arena = Allocator::default();
    let source = "前\r\n<template><p>{{ 雪 + &#49; }}</p></template>後";
    let start = source.find("<p>").unwrap();
    let end = source.find("</template>").unwrap();
    let root = SourceRoot::new(source).unwrap();
    let block = root
        .block(source.get(start..end).unwrap(), start as u32)
        .unwrap();
    let original = parse_component_block(&arena, block).into_expression_pool();
    let ast = original
        .bindings()
        .first()
        .unwrap()
        .admitted()
        .unwrap()
        .base()
        .expression()
        .unwrap();
    let observation = lower_vue2_component(&arena, original);
    let artifact = observation.admitted().unwrap();
    let [Op::Element(p)] = artifact.root().ops.as_slice() else {
        panic!("p")
    };
    let [Op::Interpolation(interpolation)] = p.children.ops.as_slice() else {
        panic!("interpolation")
    };
    let ExprRef::Js(expression) = interpolation.expression else {
        panic!("retained JS")
    };
    assert!(core::ptr::eq(expression.ast, ast));
    assert_eq!(expression.source, "雪 + 1");
    assert_eq!(expression.span.start, source.find("雪").unwrap() as u32);
    let entity = source.find("&#49;").unwrap() as u32;
    assert_eq!(
        expression.authored_span(Span::new(6, 7)),
        Some(Span::new(entity, entity + 5))
    );
    assert!(expression.matches_authored_source(source));
    assert_eq!(observation.original().block().span(), block.span());
}

#[test]
fn filter_refusal_keeps_the_original_chain_and_later_genuine_interpolation() {
    let arena = Allocator::default();
    let source = "<div>{{ first | upper(2) }}{{ second }}</div>";
    let original = parse_component(&arena, source)
        .unwrap()
        .into_expression_pool();
    let later = original
        .bindings()
        .get(1)
        .unwrap()
        .admitted()
        .unwrap()
        .base()
        .expression()
        .unwrap();
    let observation = lower_vue2_component(&arena, original);
    assert!(observation.admitted().is_none());
    assert_eq!(observation.original().bindings().len(), 2);
    assert_eq!(
        observation
            .original()
            .bindings()
            .first()
            .unwrap()
            .admitted()
            .unwrap()
            .filters()
            .len(),
        1
    );
    assert!(
        observation
            .holes()
            .iter()
            .any(|hole| hole.kind == Vue2HoleKind::Filter)
    );
    let [Op::Element(div)] = observation.artifact().unwrap().root().ops.as_slice() else {
        panic!("retained fragment")
    };
    let [Op::Interpolation(interpolation)] = div.children.ops.as_slice() else {
        panic!("later genuine fragment")
    };
    let ExprRef::Js(expression) = interpolation.expression else {
        panic!("retained JS")
    };
    assert!(core::ptr::eq(expression.ast, later));
}

#[test]
fn unsupported_dialect_semantics_are_holes_and_never_modern_reinterpretations() {
    for (source, expected) in [
        (
            "<div @click='count++'>{{ count }}</div>",
            Vue2HoleKind::UnsupportedAttribute,
        ),
        (
            "<div v-for='x in items'>{{ x }}</div>",
            Vue2HoleKind::UnsupportedAttribute,
        ),
        (
            "<div :id.sync='value'>{{ value }}</div>",
            Vue2HoleKind::UnsupportedAttribute,
        ),
        ("<widget>{{ count }}</widget>", Vue2HoleKind::UnsupportedTag),
        ("<slot/>", Vue2HoleKind::UnsupportedTag),
        ("<svg><circle/></svg>", Vue2HoleKind::UnsupportedTag),
        ("<div>\n  text\n</div>", Vue2HoleKind::WhitespacePolicy),
        (
            "<div><!-- comment --><span>text</span></div>",
            Vue2HoleKind::UnsupportedMarkup,
        ),
    ] {
        let arena = Allocator::default();
        let original = parse_component(&arena, source)
            .unwrap()
            .into_expression_pool();
        let observation = lower_vue2_component(&arena, original);
        assert!(observation.admitted().is_none(), "{source}");
        assert!(
            observation.holes().iter().any(|hole| hole.kind == expected),
            "{source}: {:?}",
            observation.holes()
        );
        assert_eq!(observation.original().block().root_source(), source);
    }
}

#[test]
fn recovery_and_encoded_delimiters_preserve_the_original_observations() {
    for source in [
        "<div>{{ bad + }}{{ later }}</div>",
        "<div>&#123;&#123; value &#125;&#125;</div>",
        "<div>{{ first }}</p>",
    ] {
        let arena = Allocator::default();
        let original = parse_component(&arena, source)
            .unwrap()
            .into_expression_pool();
        let callbacks = original.bindings().len();
        let errors = original.errors().len();
        let boundaries = original.unsupported().len();
        let observation = lower_vue2_component(&arena, original);
        assert!(observation.admitted().is_none(), "{source}");
        assert!(!observation.holes().is_empty());
        assert_eq!(observation.original().bindings().len(), callbacks);
        assert_eq!(observation.original().errors().len(), errors);
        assert_eq!(observation.original().unsupported().len(), boundaries);
    }
}

#[test]
fn static_attribute_dialect_differences_and_runtime_data_roles_are_refused() {
    for (source, kind) in [
        (
            "<div id='&copy;'>{{ x }}</div>",
            Vue2HoleKind::AttributeEncoding,
        ),
        (
            "<div id='&apos;'>{{ x }}</div>",
            Vue2HoleKind::AttributeEncoding,
        ),
        (
            "<div id='&#10;'>{{ x }}</div>",
            Vue2HoleKind::AttributeEncoding,
        ),
        (
            "<div id='&#9;'>{{ x }}</div>",
            Vue2HoleKind::AttributeEncoding,
        ),
        (
            "<div id='&#x41;'>{{ x }}</div>",
            Vue2HoleKind::AttributeEncoding,
        ),
        (
            "<video muted>{{ x }}</video>",
            Vue2HoleKind::UnsupportedAttribute,
        ),
        (
            "<div inline-template><span>x</span></div>",
            Vue2HoleKind::UnsupportedAttribute,
        ),
    ] {
        let arena = Allocator::default();
        let original = parse_component(&arena, source)
            .unwrap()
            .into_expression_pool();
        let observation = lower_vue2_component(&arena, original);
        assert!(observation.admitted().is_none(), "{source}");
        assert_eq!(
            observation
                .holes()
                .iter()
                .filter(|hole| hole.kind == kind)
                .count(),
            1,
            "{source}"
        );
        assert_eq!(observation.original().block().source(), source);
    }
}
