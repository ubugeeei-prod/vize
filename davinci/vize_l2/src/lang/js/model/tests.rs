//! Whole original operands, independently of File mutation admission.

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
fn original_js_ts_model_ast_coordinates_and_full_quoted_attribute_survive_moves_and_rejoins() {
    for (source, expected_attribute) in [
        (
            "\r\n<script setup>let café='雪'</script><template>\r\n<input v-model = 'café' />\r\n</template>\r\n",
            "v-model = 'café'",
        ),
        (
            "<script setup lang='ts'>let café:string='雪'</script><template><input v-model=\"café\"></template>",
            "v-model=\"café\"",
        ),
        (
            "<script setup>var café=0</script><template><input v-model=café /></template>",
            "v-model=café",
        ),
    ] {
        let arena = Allocator::default();
        let selected = selected(&arena, source);
        let element = selected
            .children()
            .find_map(|child| child.into_element())
            .unwrap();
        let attribute = element.attributes().next().unwrap();
        let operand = selected
            .observe_attribute_head(attribute.reborrow())
            .unwrap()
            .model()
            .unwrap()
            .unwrap()
            .observe_expression()
            .unwrap();
        let ast = operand.syntax().expression().unwrap();
        let raw = operand.raw_value();
        assert_eq!(operand.name_span().slice(source), "v-model");
        assert_eq!(operand.value_span().slice(source), "café");
        let mut input = NativeModelInput::from_operand(operand);
        assert!(input.expression().is_none());
        assert!(input.attribute_span().is_none());
        let expression = input.admitted_for(&selected, attribute.reborrow()).unwrap();
        assert!(core::ptr::eq(expression.ast, ast));
        assert!(core::ptr::eq(expression.source, raw));
        assert_eq!(expression.span.slice(source), "café");
        assert!(expression.matches_authored_source(source));
        assert!(input.matches_expression(expression));
        assert_eq!(
            input.attribute_span().unwrap().slice(source),
            expected_attribute
        );
        let identifier = ast.get_identifier_reference().unwrap();
        let decoded = expression.ast_span_to_source(identifier.span).unwrap();
        assert_eq!(
            expression
                .source
                .get(decoded.start as usize..decoded.end as usize),
            Some("café")
        );
        assert_eq!(
            expression.authored_span(decoded).unwrap(),
            input.operand().value_span()
        );
        let coordinates = expression.coordinates.unwrap();
        let mut moved = [input];
        let [input] = &mut moved;
        let repeated = input.admitted_for(&selected, attribute).unwrap();
        assert!(core::ptr::eq(repeated, expression));
        assert!(core::ptr::eq(repeated.coordinates.unwrap(), coordinates));
        assert!(input.matches_expression(repeated));
        assert_eq!(input.operand().syntax().comments().count(), 0);
        assert_eq!(input.operand().syntax().diagnostics().count(), 0);
    }
}

#[test]
fn same_buffer_reparse_sibling_and_header_ordinal_cannot_substitute_the_original_model() {
    let arena = Allocator::default();
    let source =
        "<template><input v-model='draft' v-model='draft'/><input v-model='draft'/></template>";
    let own = selected(&arena, source);
    let foreign = selected(&arena, source);
    let mut children = own.children();
    let first = children.next().unwrap().into_element().unwrap();
    let sibling = children.next().unwrap().into_element().unwrap();
    let mut attributes = first.attributes();
    let attribute = attributes.next().unwrap();
    let duplicate = attributes.next().unwrap();
    let sibling_attribute = sibling.attributes().next().unwrap();
    let other_element = foreign.children().next().unwrap().into_element().unwrap();
    let other_attribute = other_element.attributes().next().unwrap();
    let operand = own
        .observe_attribute_head(attribute.reborrow())
        .unwrap()
        .model()
        .unwrap()
        .unwrap()
        .observe_expression()
        .unwrap();
    let mut input = NativeModelInput::from_operand(operand);
    assert!(matches!(
        input.admitted_for(&foreign, other_attribute),
        Err(NativeModelInputError::UnadmittedOriginal)
    ));
    assert!(matches!(
        input.admitted_for(&own, duplicate),
        Err(NativeModelInputError::UnadmittedOriginal)
    ));
    assert!(matches!(
        input.admitted_for(&own, sibling_attribute),
        Err(NativeModelInputError::UnadmittedOriginal)
    ));
    assert!(input.expression().is_none() && input.attribute_span().is_none());
    let original = input.admitted_for(&own, attribute.reborrow()).unwrap();
    assert!(input.matches_expression(original));
    let copied = *original;
    assert!(!input.matches_expression(&copied));
    let without_coordinates = JsExpr {
        coordinates: None,
        ..*original
    };
    assert!(!input.matches_expression(&without_coordinates));
    let other_attribute = other_element.attributes().next().unwrap();
    let other_operand = foreign
        .observe_attribute_head(other_attribute.reborrow())
        .unwrap()
        .model()
        .unwrap()
        .unwrap()
        .observe_expression()
        .unwrap();
    let mut other_input = NativeModelInput::from_operand(other_operand);
    let other_expression = other_input.admitted_for(&foreign, other_attribute).unwrap();
    assert!(!input.matches_expression(other_expression));
    assert!(input.admitted_for(&own, attribute).is_ok());
}

#[test]
fn model_member_ts_wrapper_comments_entities_padding_and_parse_holes_keep_original_observations() {
    for value in [
        "draft.value",
        "draft['value']",
        "(draft)",
        "draft as string",
        "draft!",
        "draft /* kept */",
        "d&#114;aft",
        "\\u0064raft",
        " draft",
        "draft ",
        "true",
        "draft +",
    ] {
        let arena = Allocator::default();
        let source = alloc::format!(
            "<script setup lang='ts'>let draft=''</script><template><input v-model=\"{value}\"/></template>"
        );
        let selected = selected(&arena, &source);
        let element = selected.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().next().unwrap();
        let operand = selected
            .observe_attribute_head(attribute.reborrow())
            .unwrap()
            .model()
            .unwrap()
            .unwrap()
            .observe_expression()
            .unwrap();
        let hole = operand.syntax().hole();
        let comments = operand.syntax().comments().count();
        let diagnostics = operand.syntax().diagnostics().count();
        let ast = operand.syntax().expression().map(core::ptr::from_ref);
        let mut input = NativeModelInput::from_operand(operand);
        let error = input.admitted_for(&selected, attribute).unwrap_err();
        if let Some(hole) = hole {
            assert_eq!(error, NativeModelInputError::Hole(hole), "{value}");
        } else {
            assert_eq!(error, NativeModelInputError::UnsupportedOperand, "{value}");
        }
        assert!(input.expression().is_none());
        assert!(input.attribute_span().is_none());
        assert_eq!(input.operand().raw_value(), value);
        assert_eq!(input.operand().syntax().hole(), hole);
        assert_eq!(input.operand().syntax().comments().count(), comments);
        assert_eq!(input.operand().syntax().diagnostics().count(), diagnostics);
        assert_eq!(
            input
                .operand()
                .syntax()
                .expression()
                .map(core::ptr::from_ref),
            ast
        );
        assert!(core::ptr::eq(
            input.operand().syntax().source().authored_root(),
            source.as_str()
        ));
    }
}
