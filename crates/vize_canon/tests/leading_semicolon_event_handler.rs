use vize_canon::{SfcTypeCheckOptions, type_check_sfc_with_options_api};
use vize_s0::String;

const SOURCE: &str = r#"<script setup lang="ts">
let currentFocus: object | null = null
let currentArea = {}
const area = {}
const pause = () => {}
const resume = (_force: boolean) => {}
</script>

<template>
  <input
    @focus=";(currentFocus = { on: 'area', area }), (currentArea = area), pause()"
    @blur=";(currentFocus = null), resume(true)"
  >
</template>
"#;

#[test]
fn v_on_asi_prefix_is_not_parenthesized_as_an_expression() {
    let virtual_ts = generate_virtual_ts(SOURCE);

    assert!(
        !virtual_ts.contains("void (;"),
        "a leading empty statement is invalid inside a parenthesized expression:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains(
            "void ((currentFocus = { on: 'area', area }), (currentArea = area), pause()); // VOn"
        ),
        "removing the ASI prefix must preserve the complete focus handler:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("void ((currentFocus = null), resume(true)); // VOn"),
        "removing the ASI prefix must preserve the complete blur handler:\n{virtual_ts}"
    );
}

#[test]
fn generated_typescript_with_v_on_asi_prefixes_is_parseable() {
    let virtual_ts = generate_virtual_ts(SOURCE);
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
            .parse();

    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "Vue event handlers with an ASI prefix must generate valid TypeScript: {:#?}\n{virtual_ts}",
        parsed.diagnostics
    );
}

#[test]
fn v_on_with_only_empty_statements_emits_no_void_expression() {
    let virtual_ts = generate_virtual_ts("<template><button @click=\"; ;\" /></template>");

    assert!(
        !virtual_ts.contains("void (); // VOn") && !virtual_ts.contains("void (;"),
        "empty event-handler statements must not become invalid TypeScript:\n{virtual_ts}"
    );
}

fn generate_virtual_ts(source: &str) -> String {
    type_check_sfc_with_options_api(
        source,
        &SfcTypeCheckOptions::new("CssCodeAreaName.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated")
}
