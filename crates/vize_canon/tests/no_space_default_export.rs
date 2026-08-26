use vize_canon::{SfcTypeCheckOptions, type_check_sfc_with_options_api};
use vize_s0::String;

const LEGACY_OPTIONS_SOURCE: &str = r#"<script>
  export default{
    data(){
      return { count: 1 }
    },
    methods: {
      increment(){
        this.count++
      }
    }
  }
</script>

<template>
  <button @click="increment">{{ count }}</button>
</template>
"#;

const COMMENT_BOUNDARY_SOURCE: &str = r#"<script>
export default/* keep this comment */{
  name: 'CommentBoundary'
}
</script>
"#;

#[test]
fn indented_no_space_default_export_is_wrapped_as_options_api() {
    let virtual_ts = generate_virtual_ts(LEGACY_OPTIONS_SOURCE);

    assert!(
        virtual_ts.contains("  const __default__ =__vizeDefineComponent({"),
        "an indented no-space default export must use the Options API wrapper:\n{virtual_ts}"
    );
    assert!(
        !virtual_ts.contains("\n    default{"),
        "the export fallback must not leave an invalid `default` token:\n{virtual_ts}"
    );
    for source_member in ["data(){", "methods: {", "increment(){", "this.count++"] {
        assert!(
            virtual_ts.contains(source_member),
            "rewriting must preserve the Options API member `{source_member}`:\n{virtual_ts}"
        );
    }
}

#[test]
fn no_space_default_export_preserves_comment_boundary() {
    let virtual_ts = generate_virtual_ts(COMMENT_BOUNDARY_SOURCE);

    assert!(
        virtual_ts.contains("const __default__ =/* keep this comment */__vizeDefineComponent({"),
        "comments between `default` and the object must survive the rewrite:\n{virtual_ts}"
    );
}

#[test]
fn generated_typescript_for_no_space_default_exports_is_parseable() {
    for source in [LEGACY_OPTIONS_SOURCE, COMMENT_BOUNDARY_SOURCE] {
        let virtual_ts = generate_virtual_ts(source);
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
                .parse();

        assert!(
            !parsed.panicked && parsed.diagnostics.is_empty(),
            "generated TypeScript must parse after rewriting a no-space default export: {:#?}\n{virtual_ts}",
            parsed.diagnostics
        );
    }
}

fn generate_virtual_ts(source: &str) -> String {
    type_check_sfc_with_options_api(
        source,
        &SfcTypeCheckOptions::new("LegacyComponent.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated")
}
