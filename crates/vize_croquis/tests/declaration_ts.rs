//! Declaration output regressions exercised through Croquis' public API.

#![allow(clippy::disallowed_macros)]

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_croquis::declaration_ts::generate_declaration_ts;

fn generate_sfc(source: &str) -> vize_carton::String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("fixture must parse");
    let script = descriptor
        .script_setup
        .as_ref()
        .expect("fixture must contain script setup");
    let summary = analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::for_declaration());

    generate_declaration_ts(&summary, Some(script.content.as_ref())).content
}

fn assert_parseable(declaration: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, declaration, SourceType::d_ts()).parse();

    assert_eq!(
        parsed.diagnostics.len(),
        0,
        "generated declaration must parse as TypeScript: {:#?}",
        parsed.diagnostics
    );
}

#[test]
fn function_typed_prop_declaration_is_parseable() {
    let declaration = generate_sfc(
        r#"<script setup lang="ts">
defineProps<{ formatter?: (value: number) => string }>();
</script>"#,
    );

    assert_parseable(&declaration);
    insta::assert_snapshot!("function_typed_prop_declaration", declaration.as_str());
}

#[test]
fn benchmark_generic_prop_declaration_is_parseable() {
    let source = r#"<script setup lang="ts" generic="T extends { id: number }">
defineProps<{
  items: T[];
  selected?: T;
  keyOf: (row: T) => string;
}>();

defineEmits<{
  pick: [row: T];
}>();
</script>

<template>
  <ul>
    <li v-for="item in items" :key="item.id">{{ keyOf(item) }}</li>
  </ul>
</template>
"#;
    let declaration = generate_sfc(source);

    assert_parseable(&declaration);
    insta::assert_snapshot!("benchmark_generic_prop_declaration", declaration.as_str());
}
