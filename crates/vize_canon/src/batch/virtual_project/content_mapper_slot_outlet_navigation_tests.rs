use std::ops::Range;
use std::path::Path;

use super::{
    CONTENT_MAPPER_SPAN_FEATURES_ALL, ContentMapperSpanKind, generate_vue_content_mapper_transform,
};
use crate::batch::{ContentMapperSpan, ContentMapperTransform};

#[test]
fn maps_static_slot_outlet_payload_name_to_the_authored_value() {
    let source = r#"<script setup lang="ts">
const slots = defineSlots<{
  header(props: { title: string }): any;
}>();
</script>
<template><slot name="header" title="Hello" /></template>
"#;
    assert_static_name_mapping(
        source,
        "header",
        "__VizeSlotOutletPayload<typeof slots, \"header\">",
        "name=\"header\"",
    );
}

#[test]
fn maps_hyphenated_static_slot_outlet_names_without_aliasing() {
    let source = r#"<script setup lang="ts">
defineSlots<{
  "item-row"(props: { title: string }): any;
}>();
</script>
<template><slot name="item-row" title="Hello" /></template>
"#;
    assert_static_name_mapping(
        source,
        "item-row",
        "__VizeSlotOutletPayload<Slots, \"item-row\">",
        "name=\"item-row\"",
    );
}

#[test]
fn maps_duplicate_static_slot_outlet_names_to_each_authored_value() {
    let source = r#"<script setup lang="ts">
defineSlots<{
  header(props: { title: string }): any;
}>();
</script>
<template>
  <slot name="header" title="One" />
  <slot name="header" title="Two" />
</template>
"#;
    let result = generate_vue_content_mapper_transform(Path::new("SlotProvider.vue"), source)
        .expect("transform");
    let generated_ranges = all_static_slot_name_ranges(
        &result.text,
        "__VizeSlotOutletPayload<Slots, \"header\">",
        "header",
    );
    let source_ranges = all_authored_attr_value_ranges(source, "name=\"header\"", "header");

    assert_eq!(generated_ranges.len(), 2, "{}", result.text);
    assert_eq!(source_ranges.len(), 2);
    for (generated, source) in generated_ranges.iter().zip(source_ranges.iter()) {
        assert_has_name_mapping(&result.mappings, generated.clone(), source.clone());
    }
}

#[test]
fn implicit_default_slot_outlet_names_do_not_fabricate_a_source_mapping() {
    let source = r#"<script setup lang="ts">
defineSlots<{
  default(props: { title: string }): any;
}>();
</script>
<template><slot title="Hello" /></template>
"#;
    let result = generate_vue_content_mapper_transform(Path::new("SlotProvider.vue"), source)
        .expect("transform");
    let generated = static_slot_name_range(
        &result.text,
        "__VizeSlotOutletPayload<Slots, \"default\">",
        "default",
    );

    assert_no_mapping_overlaps_generated(&result, generated);
}

#[test]
fn valueless_static_slot_outlet_names_do_not_fabricate_a_source_mapping() {
    let source = r#"<script setup lang="ts">
defineSlots<{
  default(props: { title: string }): any;
}>();
</script>
<template><slot name title="Hello" /></template>
"#;
    let result = generate_vue_content_mapper_transform(Path::new("SlotProvider.vue"), source)
        .expect("transform");
    let generated = static_slot_name_range(
        &result.text,
        "__VizeSlotOutletPayload<Slots, \"default\">",
        "default",
    );

    assert_no_mapping_overlaps_generated(&result, generated);
}

#[test]
fn static_slot_outlet_names_without_payload_props_do_not_emit_checks() {
    let source = r#"<script setup lang="ts">
defineSlots<{
  header(): any;
}>();
</script>
<template><slot name="header" /></template>
"#;
    let result = generate_vue_content_mapper_transform(Path::new("SlotProvider.vue"), source)
        .expect("transform");

    assert!(
        !result
            .text
            .contains("__VizeSlotOutletPayload<Slots, \"header\">"),
        "{}",
        result.text
    );
}

#[test]
fn dynamic_slot_outlet_names_do_not_claim_static_value_navigation() {
    let source = r#"<script setup lang="ts">
const slotName = "header";
</script>
<template><slot :name="slotName" title="Hello" /></template>
"#;
    let result = generate_vue_content_mapper_transform(Path::new("SlotProvider.vue"), source)
        .expect("transform");
    let generated = generated_text_range(&result.text, "__VizeAnySlotOutletPayload<Slots>");

    assert!(
        !result
            .text
            .contains("__VizeSlotOutletPayload<Slots, \"header\">")
    );
    assert_no_mapping_overlaps_generated(&result, generated);
}

#[test]
fn dynamic_literal_slot_outlet_names_stay_dynamic() {
    let source = r#"<script setup lang="ts">
defineSlots<{
  header(props: { title: string }): any;
}>();
</script>
<template><slot :name="'header'" title="Hello" /></template>
"#;
    let result = generate_vue_content_mapper_transform(Path::new("SlotProvider.vue"), source)
        .expect("transform");
    let generated = generated_text_range(&result.text, "__VizeAnySlotOutletPayload<Slots>");

    assert!(
        !result
            .text
            .contains("__VizeSlotOutletPayload<Slots, \"header\">")
    );
    assert_no_mapping_overlaps_generated(&result, generated);
}

#[test]
fn v_bind_name_slot_outlet_names_stay_dynamic() {
    let source = r#"<script setup lang="ts">
const slotName = "header";
</script>
<template><slot v-bind:name="slotName" title="Hello" /></template>
"#;
    let result = generate_vue_content_mapper_transform(Path::new("SlotProvider.vue"), source)
        .expect("transform");
    let generated = generated_text_range(&result.text, "__VizeAnySlotOutletPayload<Slots>");

    assert!(
        !result
            .text
            .contains("__VizeSlotOutletPayload<Slots, \"header\">")
    );
    assert_no_mapping_overlaps_generated(&result, generated);
}

#[test]
fn same_name_shorthand_slot_outlet_names_stay_dynamic() {
    let source = r#"<script setup lang="ts">
const name = "header";
</script>
<template><slot :name title="Hello" /></template>
"#;
    let result = generate_vue_content_mapper_transform(Path::new("SlotProvider.vue"), source)
        .expect("transform");
    let generated = generated_text_range(&result.text, "__VizeAnySlotOutletPayload<Slots>");

    assert!(
        !result
            .text
            .contains("__VizeSlotOutletPayload<Slots, \"default\">")
    );
    assert_no_mapping_overlaps_generated(&result, generated);
}

#[test]
fn escaped_static_slot_outlet_names_drop_edit_capable_features() {
    let source = r#"<script setup lang="ts">
defineSlots<{
  "a\"b"(props: { title: string }): any;
}>();
</script>
<template><slot name='a"b' title="Hello" /></template>
"#;
    let result = generate_vue_content_mapper_transform(Path::new("SlotProvider.vue"), source)
        .expect("transform");
    let generated = static_slot_name_range(
        &result.text,
        "__VizeSlotOutletPayload<Slots, \"a\\\"b\">",
        r#"a\"b"#,
    );
    let original = authored_attr_value_range(source, "name='a\"b'", "a\"b");

    assert!(
        result.mappings.iter().any(|mapping| {
            mapping.0[0] == generated.start
                && mapping.0[1] == generated.len()
                && mapping.0[2] == original.start
                && mapping.0[3] == original.len()
                && mapping.0[4] == ContentMapperSpanKind::Atom as usize
                && mapping.0[5] != CONTENT_MAPPER_SPAN_FEATURES_ALL
        }),
        "{:#?}",
        result.mappings
    );
}

fn assert_static_name_mapping(source: &str, name: &str, projection: &str, authored_attr: &str) {
    let result = generate_vue_content_mapper_transform(Path::new("SlotProvider.vue"), source)
        .expect("transform");
    let generated = static_slot_name_range(&result.text, projection, name);
    let original = authored_attr_value_range(source, authored_attr, name);

    assert_has_name_mapping(&result.mappings, generated, original);
}

fn assert_has_name_mapping(
    mappings: &[ContentMapperSpan],
    generated: Range<usize>,
    original: Range<usize>,
) {
    assert!(
        mappings.iter().any(|mapping| {
            mapping.0[0] == generated.start
                && mapping.0[1] == generated.len()
                && mapping.0[2] == original.start
                && mapping.0[3] == original.len()
                && mapping.0[4] == ContentMapperSpanKind::Verbatim as usize
                && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_ALL
        }),
        "{mappings:#?}"
    );
}

fn assert_no_mapping_overlaps_generated(result: &ContentMapperTransform, generated: Range<usize>) {
    assert!(
        !result
            .mappings
            .iter()
            .any(|mapping| ranges_overlap(&content_mapper_generated_range(mapping), &generated)),
        "{:#?}",
        result.mappings
    );
}

fn static_slot_name_range(
    generated: &str,
    projection: &str,
    generated_literal_content: &str,
) -> Range<usize> {
    let start = static_slot_name_start(generated, projection);
    start..start + generated_literal_content.len()
}

fn static_slot_name_start(generated: &str, projection: &str) -> usize {
    generated.find(projection).unwrap() + projection.find('"').unwrap() + 1
}

fn all_static_slot_name_ranges(generated: &str, projection: &str, name: &str) -> Vec<Range<usize>> {
    let content_offset = projection.find('"').unwrap() + 1;
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = generated[cursor..].find(projection) {
        let start = cursor + relative_start + content_offset;
        ranges.push(start..start + name.len());
        cursor += relative_start + projection.len();
    }
    ranges
}

fn authored_attr_value_range(source: &str, authored_attr: &str, value: &str) -> Range<usize> {
    let attr_start = source.find(authored_attr).unwrap();
    let value_start = source[attr_start..].find(value).unwrap() + attr_start;
    value_start..value_start + value.len()
}

fn all_authored_attr_value_ranges(
    source: &str,
    authored_attr: &str,
    value: &str,
) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = source[cursor..].find(authored_attr) {
        let attr_start = cursor + relative_start;
        let value_start = source[attr_start..].find(value).unwrap() + attr_start;
        ranges.push(value_start..value_start + value.len());
        cursor = attr_start + authored_attr.len();
    }
    ranges
}

fn generated_text_range(generated: &str, text: &str) -> Range<usize> {
    let start = generated.find(text).unwrap();
    start..start + text.len()
}

fn content_mapper_generated_range(mapping: &ContentMapperSpan) -> Range<usize> {
    mapping.0[0]..mapping.0[0] + mapping.0[1]
}

fn ranges_overlap(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}
