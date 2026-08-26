use vize_canon::{SfcTypeCheckOptions, type_check_sfc_with_options_api};
use vize_s0::String;

const SOURCE: &str = r#"<script>
export default {
  props: {
    content: {
      type: String,
      default: ''
    },
    offset: {
      type: Number,
      default: Math.PI / 4
    },
    direction: {
      type: String,
      default: 'lt' // preserve this runtime comment in value scope
    }
  }
}
</script>

<template><span>{{ direction }}</span></template>
"#;

#[test]
fn options_api_runtime_props_stay_in_value_scope() {
    let virtual_ts = generate_virtual_ts(SOURCE);

    assert!(
        !virtual_ts.contains("export type Props = __RuntimePropShape<{"),
        "a runtime object must never be copied into TypeScript type position:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("const __vize_options_props = ({"),
        "the runtime props object must be captured as a setup-scoped value:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("default: Math.PI / 4"),
        "runtime default expressions must remain byte-complete:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains(
            "export type Props = __VizeOptionsPropShape<Awaited<ReturnType<typeof __setup>>[\"__vize_options_props\"]>;"
        ),
        "Props must be derived from the captured runtime value:\n{virtual_ts}"
    );
}

#[test]
fn options_api_runtime_props_with_comments_generate_parseable_typescript() {
    let virtual_ts = generate_virtual_ts(SOURCE);
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
            .parse();

    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "runtime props defaults and comments must not corrupt generated TypeScript: {:#?}\n{virtual_ts}",
        parsed.diagnostics
    );
}

fn generate_virtual_ts(source: &str) -> String {
    type_check_sfc_with_options_api(
        source,
        &SfcTypeCheckOptions::new("MintPaletteButton.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated")
}
