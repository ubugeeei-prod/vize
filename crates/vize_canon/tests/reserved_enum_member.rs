use vize_canon::{SfcTypeCheckOptions, type_check_sfc_with_options_api};
use vize_s0::String;

const SOURCE: &str = r#"<template>
  <p v-if="mode === MODES.export">{{ mode }}</p>
</template>

<script setup lang="ts">
enum MODES {
  tag = "tag",
  category = "category",
  export = "export",
  delete = "delete",
}

const mode = MODES.export
</script>
"#;

#[test]
fn reserved_enum_members_survive_setup_script_rewriting() {
    let virtual_ts = generate_virtual_ts(SOURCE);

    assert!(
        virtual_ts.contains("export = \"export\","),
        "an enum member named export must not be mistaken for a declaration modifier:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("delete = \"delete\","),
        "preserving export must not disturb neighboring reserved enum members:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("const mode = MODES.export"),
        "the enum member reference must remain available to setup and template code:\n{virtual_ts}"
    );
}

#[test]
fn generated_typescript_with_reserved_enum_members_is_parseable() {
    let virtual_ts = generate_virtual_ts(SOURCE);
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
            .parse();

    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "reserved enum members must generate valid TypeScript: {:#?}\n{virtual_ts}",
        parsed.diagnostics
    );
}

fn generate_virtual_ts(source: &str) -> String {
    type_check_sfc_with_options_api(
        source,
        &SfcTypeCheckOptions::new("recipes.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated")
}
