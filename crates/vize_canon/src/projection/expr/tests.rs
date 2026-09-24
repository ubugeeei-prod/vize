//! S4 projection rows preserve the authored expression, including byte offsets.
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use super::{project_template_expression_document, project_template_expressions};
use crate::virtual_ts::{ProjectionMapping, VizeMapping};

#[test]
fn prelowered_region_emits_the_same_document_and_mapping() {
    let source =
        "<div v-if=\"状態\" :[field]=\"value\">{{ count }}</div>\r\n<slot :name=\"slotName\" />";
    let allocator = vize_carton::Allocator::new();
    let (tree, errors) = vize_s1::parse(&allocator, source);
    let lowered = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    let document = project_template_expression_document(&lowered.root);

    assert_eq!(document.as_str(), "状態\nfield\nvalue\ncount\nslotName");
    assert_eq!(document.links().len(), 5);
    assert_eq!(
        ProjectionMapping::from_emit_document(&document),
        project_template_expressions(source)
    );
}

fn assert_expressions(source: &str, expressions: &[&str]) {
    let mapping = project_template_expressions(source);
    let mut generated = 0;
    let mut search_from = 0;
    let expected: Vec<_> = expressions
        .iter()
        .map(|expr| {
            let start = search_from + source[search_from..].find(expr).unwrap();
            let end = start + expr.len();
            let row = VizeMapping::new(generated..generated + expr.len(), start..end);
            assert_eq!(
                mapping.diagnostic_range_to_authored(generated, generated + expr.len()),
                Some((start, end))
            );
            generated += expr.len() + 1;
            search_from = end;
            row
        })
        .collect();
    assert_eq!(mapping.spans(), expected, "{source}");
}

#[test]
fn interpolation_and_binding_keep_their_expression_spans() {
    assert_expressions("{{ count }}", &["count"]);
    assert_expressions(r#"<div :title="name"></div>"#, &["name"]);
}

#[test]
fn conditional_branches_project_conditions_before_their_bodies() {
    assert_expressions(
        r#"<p v-if="ready">{{ first }}</p><p v-else-if="pending">{{ second }}</p><p v-else>{{ fallback }}</p>"#,
        &["ready", "first", "pending", "second", "fallback"],
    );
}

#[test]
fn loops_project_the_source_but_not_binding_patterns() {
    for source in [
        r#"<p v-for="(item, index) in entries" :key="item.id">{{ item.name }}</p>"#,
        r#"<p v-for="({ id, name }, index) of entries" :key="id">{{ name }}</p>"#,
    ] {
        let expected = if source.contains("item") {
            &["entries", "item.id", "item.name"][..]
        } else {
            &["entries", "id", "name"][..]
        };
        assert_expressions(source, expected);
    }
}

#[test]
fn nested_structural_directives_keep_each_expression_once() {
    assert_expressions(
        r#"<template v-if="visible"><p v-for="item in list" :title="item.label">{{ item.text }}</p></template>"#,
        &["visible", "list", "item.label", "item.text"],
    );
}

#[test]
fn dynamic_arguments_handlers_and_object_spreads_are_projected() {
    assert_expressions(
        r#"<Comp :[field]="value" @[event]="handler" v-bind="attrs" v-on="listeners" />"#,
        &["field", "value", "event", "handler", "attrs", "listeners"],
    );
    assert_expressions(
        r#"<button @click="count++; save(count)" @blur.stop />"#,
        &["count++; save(count)"],
    );
}

#[test]
fn vue_value_directives_are_projected_without_duplicate_model_reads() {
    assert_expressions(
        r#"<Comp v-model:[field]="model" v-show="visible" v-memo="[dep]" v-pin:[side]="offset" />"#,
        &["field", "model", "visible", "[dep]", "side", "offset"],
    );
    assert_expressions(
        r#"<p v-html="rawHtml"/><p v-text="caption"/>"#,
        &["rawHtml", "caption"],
    );
}

#[test]
fn dynamic_slots_project_names_and_children_without_scope_patterns() {
    assert_expressions(
        r#"<slot :name="slotName" :title="heading">{{ fallback }}</slot>"#,
        &["slotName", "heading", "fallback"],
    );
    assert_expressions(
        r#"<Comp><template #[slotName]="{ item }">{{ caption }}</template></Comp>"#,
        &["slotName", "caption"],
    );
}

#[test]
fn unicode_and_crlf_do_not_shift_authored_bytes() {
    assert_expressions(
        "<p>日本語😀</p>\r\n<div v-if=\"状態\" :title=\"名前\">{{ 数 }}</div>",
        &["状態", "名前", "数"],
    );
}

#[test]
fn static_and_v_pre_content_create_no_projection_rows() {
    assert_expressions(
        r#"<div title="plain" v-once v-cloak><p v-pre :title="raw">{{ untouched }}</p></div>"#,
        &[],
    );
}
