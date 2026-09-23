//! A normal script beside `<script setup>` must not panic the checker document.
//!
//! Analysis joins the two blocks with one synthetic newline. The editor
//! document has to feed that same text, and map setup spans through the
//! authored block rather than the joined string.

#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use vize_maestro::VirtualCodeGenerator;

#[test]
fn split_script_checker_document_maps_both_blocks() {
    let source = r#"<script lang="ts">
export default {}
</script>

<script setup lang="ts">
const x = 1
</script>

<template>
  <div>{{ x }}</div>
</template>

<style scoped>
.a {
  color: red;
}
</style>

<style module="m">
.b {
  color: blue;
}
</style>
"#;
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
    let mut generator = VirtualCodeGenerator::new();
    let docs = generator.generate(&descriptor, "AllBlocks.vue");
    let template = docs.template.expect("template document");
    assert_eq!(
        template.content.as_str(),
        include_str!("fixtures/split_script_checker.virtual.ts")
    );

    let export_at = source.find("export default").unwrap();
    let export_generated = template
        .source_map
        .to_generated(export_at)
        .expect("normal script maps into the checker document");
    let mapped = "const __default__";
    assert_eq!(
        &template.content[export_generated..export_generated + mapped.len()],
        mapped
    );

    let setup_at = source.find("const x = 1").unwrap();
    let setup_generated = template
        .source_map
        .to_generated(setup_at)
        .expect("script setup maps into the checker document");
    assert_eq!(
        &template.content[setup_generated..setup_generated + "const x = 1".len()],
        "const x = 1"
    );
}
