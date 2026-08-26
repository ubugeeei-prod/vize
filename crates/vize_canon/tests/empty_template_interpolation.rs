use vize_canon::{SfcTypeCheckOptions, type_check_sfc_with_options_api};
use vize_s0::String;

const SOURCE: &str = r#"<script setup lang="ts">
const value = 'visible'
</script>

<template>
  <SfCollectedProduct>
    <template #more-actions>{{  }}</template>
  </SfCollectedProduct>
  <p>{{ value }}</p>
</template>
"#;

#[test]
fn empty_interpolation_emits_no_typescript_expression() {
    let virtual_ts = generate_virtual_ts(SOURCE);

    assert!(
        !virtual_ts.contains("void (); // Interpolation"),
        "blank Vue interpolations must not become invalid void expressions:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("void (value); // Interpolation"),
        "skipping a blank interpolation must preserve neighboring expressions:\n{virtual_ts}"
    );
}

#[test]
fn generated_typescript_with_empty_interpolation_is_parseable() {
    let virtual_ts = generate_virtual_ts(SOURCE);
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
            .parse();

    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "blank Vue interpolations must not corrupt generated TypeScript: {:#?}\n{virtual_ts}",
        parsed.diagnostics
    );
}

fn generate_virtual_ts(source: &str) -> String {
    type_check_sfc_with_options_api(
        source,
        &SfcTypeCheckOptions::new("CartSidebar.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated")
}
