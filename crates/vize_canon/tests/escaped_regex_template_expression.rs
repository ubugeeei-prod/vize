use vize_canon::{SfcTypeCheckOptions, type_check_sfc_with_options_api};
use vize_s0::String;

const SOURCE: &str = r#"<script setup lang="ts">
const baseUrl = 'http://example.test'
function translate(key: string, args: { url: string }) {
  return `${key}: ${args.url}`
}
</script>

<template>
  <p>
    {{
      translate('description', {
        url: baseUrl.replace(/^http:\/\//, ''),
      })
    }}
  </p>
</template>
"#;

#[test]
fn escaped_slashes_in_template_regex_survive_comment_stripping() {
    let virtual_ts = generate_virtual_ts(SOURCE);

    assert!(
        virtual_ts.contains("url: baseUrl.replace(/^http:\\/\\//, ''),"),
        "the regular expression must remain byte-complete:\n{virtual_ts}"
    );
}

#[test]
fn generated_typescript_with_escaped_regex_is_parseable() {
    let virtual_ts = generate_virtual_ts(SOURCE);
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
            .parse();

    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "generated TypeScript must parse after preserving escaped regex slashes: {:#?}\n{virtual_ts}",
        parsed.diagnostics
    );
}

fn generate_virtual_ts(source: &str) -> String {
    type_check_sfc_with_options_api(
        source,
        &SfcTypeCheckOptions::new("CraterModule.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated")
}
