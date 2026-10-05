use super::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[test]
fn test_script_setup_imported_define_page_meta_from_typed_router_is_compile_time_only() {
    let source = r#"<script setup lang="ts">
import { definePageMeta } from '@typed-router'

definePageMeta({
  layout: 'no-header',
})

const msg = 'ready'
</script>
<template>
  <div>{{ msg }}</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions::default();
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    assert!(
        !result.code.contains("definePageMeta"),
        "definePageMeta should be removed from runtime output:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("@typed-router"),
        "macro-only typed-router import should be removed from runtime output:\n{}",
        result.code
    );
    assert_eq!(result.macro_artifacts.len(), 1);

    let artifact = &result.macro_artifacts[0];
    assert_eq!(artifact.kind.as_str(), "nuxt.definePageMeta");
    assert!(artifact.content.contains("no-header"));
    assert!(
        artifact
            .module_code
            .as_ref()
            .is_some_and(|code| code.starts_with("const __nuxt_page_meta = {")
                && code.contains("export default __nuxt_page_meta"))
    );
}

#[test]
fn test_nuxt_explicit_page_meta_import_produces_route_artifact() {
    let source =
        include_str!("../../../../../tests/_fixtures/differential/nuxt/explicit-page-meta/App.vue");
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    let result = compile_nuxt(&descriptor, SfcCompileOptions::default()).expect("compile SFC");

    assert_eq!(result.macro_artifacts.len(), 1);
    let artifact = &result.macro_artifacts[0];
    assert_eq!(artifact.kind.as_str(), "nuxt.definePageMeta");
    assert_eq!(artifact.content.as_str(), "{ layout: 'bare' }");
    assert_eq!(
        artifact.module_code.as_deref(),
        Some("const __nuxt_page_meta = { layout: 'bare' }\nexport default __nuxt_page_meta\n")
    );
    assert!(!result.code.contains("definePageMeta"), "{}", result.code);
    assert!(!result.code.contains("#imports"), "{}", result.code);
}

#[test]
fn test_nuxt_mixed_page_meta_import_retains_runtime_binding() {
    let source = r#"<script setup lang="ts">
import { definePageMeta, useRoute } from '#imports'
definePageMeta({ layout: 'bare' })
const route = useRoute()
</script>
<template><div>{{ route.path }}</div></template>"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    let result = compile_nuxt(&descriptor, SfcCompileOptions::default()).expect("compile SFC");
    assert_eq!(result.macro_artifacts.len(), 1);
    assert!(
        result
            .code
            .contains("import { useRoute } from \"#imports\""),
        "{}",
        result.code
    );
    assert!(!result.code.contains("definePageMeta"), "{}", result.code);
}

pub(super) fn compile_nuxt(
    descriptor: &crate::SfcDescriptor,
    options: SfcCompileOptions,
) -> Result<crate::SfcCompileResult, crate::SfcError> {
    crate::compile_sfc_for_adapter_with_nuxt_page_meta(
        descriptor,
        options,
        vize_atelier_core::TemplateSyntaxMode::Standard,
        Default::default(),
        Default::default(),
        crate::SfcScriptOutputMode::InlineTemplate,
        Default::default(),
    )
}
