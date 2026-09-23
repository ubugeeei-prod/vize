//! A normal script beside `<script setup>` must not panic the checker document.
//!
//! Analysis joins the two blocks with one synthetic newline. The editor
//! document has to feed that same text, and map setup spans through the
//! authored block rather than the joined string.

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
    assert!(
        template.content.contains("void __default__"),
        "normal script is part of the checker document:\n{}",
        template.content
    );
    assert!(
        template.content.contains("const x = 1"),
        "script setup is part of the checker document:\n{}",
        template.content
    );

    let export_at = source.find("export default").unwrap();
    let export_generated = template
        .source_map
        .to_generated(export_at)
        .expect("normal script maps into the checker document");
    assert!(
        template.content[export_generated..].starts_with("const __default__")
            || template.content[export_generated..].starts_with("export default")
            || template.content[export_generated..].starts_with("defineComponent"),
        "mapped export default at {export_generated}:\n{}",
        template.content
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
