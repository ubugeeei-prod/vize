use super::*;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    markup::SurfaceParseOptions,
};

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

#[test]
fn conditional_original_stock_ast_and_authored_coordinates_survive_moves_and_rejoins() {
    let arena = Allocator::default();
    let source =
        "\r\n<script setup>let prêt=true</script><template><div v-if='prêt'>雪🌸</div></template>";
    let selected = selected(&arena, source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    let operand = selected
        .observe_attribute_head(attribute.reborrow())
        .unwrap()
        .observe_expression()
        .unwrap();
    let ast = operand.syntax().expression().unwrap();
    let value = operand.raw_value();
    let name_span = operand.name_span();
    let value_span = operand.value_span();
    let mut input = NativeConditionalInput::from_operand(operand);
    let expression = input.admitted_for(&selected, attribute.reborrow()).unwrap();
    assert!(core::ptr::eq(expression.ast, ast));
    assert!(core::ptr::eq(expression.source, value));
    assert_eq!(name_span.slice(source), "v-if");
    assert_eq!(value_span.slice(source), "prêt");
    assert!(expression.matches_authored_source(source));
    assert!(input.matches_expression(expression));
    let coordinates = expression.coordinates.unwrap();
    let mut moved = alloc::vec::Vec::new();
    moved.push(input);
    moved.reserve(16);
    let input = moved.first_mut().unwrap();
    let repeated = input.admitted_for(&selected, attribute).unwrap();
    assert!(core::ptr::eq(repeated, expression));
    assert!(core::ptr::eq(repeated.coordinates.unwrap(), coordinates));
    assert!(input.matches_expression(repeated));
    assert_eq!(input.operand().syntax().comments().count(), 0);
    assert_eq!(input.operand().syntax().diagnostics().count(), 0);
}

#[test]
fn equal_reparse_header_and_copied_expression_do_not_satisfy_the_original_receipt() {
    let arena = Allocator::default();
    let source = "<template><div v-if='ready'/></template>";
    let own = selected(&arena, source);
    let foreign = selected(&arena, source);
    let own_element = own.children().next().unwrap().into_element().unwrap();
    let foreign_element = foreign.children().next().unwrap().into_element().unwrap();
    let attribute = own_element.attributes().next().unwrap();
    let foreign_attribute = foreign_element.attributes().next().unwrap();
    let operand = own
        .observe_attribute_expression(attribute.reborrow())
        .unwrap();
    let mut input = NativeConditionalInput::from_operand(operand);
    assert!(matches!(
        input.admitted_for(&foreign, foreign_attribute),
        Err(NativeConditionalInputError::UnadmittedOriginal)
    ));
    assert!(input.expression().is_none());
    let expression = input.admitted_for(&own, attribute).unwrap();
    assert!(input.matches_expression(expression));
    let copy = *expression;
    assert!(!input.matches_expression(&copy));
    let neutral = JsExpr {
        coordinates: None,
        ..*expression
    };
    assert!(!input.matches_expression(&neutral));
    let foreign_attribute = foreign_element.attributes().next().unwrap();
    let mut other = NativeConditionalInput::from_operand(
        foreign
            .observe_attribute_expression(foreign_attribute.reborrow())
            .unwrap(),
    );
    let other_expression = other.admitted_for(&foreign, foreign_attribute).unwrap();
    assert!(!input.matches_expression(other_expression));
}

#[test]
fn rejected_condition_syntax_retains_complete_original_observations_and_holes() {
    for value in [
        "ready && ready",
        "(ready)",
        "ready /* kept */",
        "r&#101;ady",
        "\\u0072eady",
        "ready +",
    ] {
        let arena = Allocator::default();
        let source = alloc::format!("<template><div v-if='{value}'/></template>");
        let selected = selected(&arena, &source);
        let element = selected.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().next().unwrap();
        let operand = selected
            .observe_attribute_expression(attribute.reborrow())
            .unwrap();
        let hole = operand.syntax().hole();
        let comments = operand.syntax().comments().count();
        let diagnostics = operand.syntax().diagnostics().count();
        let mut input = NativeConditionalInput::from_operand(operand);
        assert!(input.admitted_for(&selected, attribute).is_err(), "{value}");
        assert!(input.expression().is_none());
        assert_eq!(input.operand().raw_value(), value);
        assert_eq!(input.operand().syntax().hole(), hole);
        assert_eq!(input.operand().syntax().comments().count(), comments);
        assert_eq!(input.operand().syntax().diagnostics().count(), diagnostics);
        assert!(core::ptr::eq(
            input.operand().syntax().source().authored_root(),
            source.as_str()
        ));
    }
}
