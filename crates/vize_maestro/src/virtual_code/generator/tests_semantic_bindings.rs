use super::VirtualCodeGenerator;
use vize_s0::Allocator;

const COMPLEX_BINDINGS_SFC: &str = r#"<script setup lang="ts">
import DefaultWidget, {
  helper as importedHelper,
  ignored as importedIgnored,
  type SourceShape,
} from './source'
import * as utilities from './utilities'
import type { TypeOnly } from './types'

const {
  api: client,
  label: localLabel = 'fallback',
  nested: { value: nestedValue },
  urls: [firstUrl, ...remainingUrls],
  ...runtimeRest
} = useRuntimeConfig()
const [, , { value: tailValue = 0 }, ...tailItems] = remainingUrls
const api = 'property-key-only'
const label = 'property-key-only'
const 挨拶 = 'こんにちは'
const unused = 'not referenced'
</script>

<template>
  <component :is="DefaultWidget" />
  <p>{{ importedHelper(client, firstUrl, tailValue) }}</p>
  <p>{{ utilities.format(nestedValue, runtimeRest, tailItems, 挨拶) }}</p>
  <div :config="{ api: client, label: localLabel }" />
</template>"#;

#[test]
fn generator_exports_semantic_script_bindings_used_by_template() {
    let descriptor = vize_atelier_sfc::parse_sfc(COMPLEX_BINDINGS_SFC, Default::default()).unwrap();
    let mut generator = VirtualCodeGenerator::new();
    let documents = generator.generate(&descriptor, "complex.vue");
    let script_setup = documents.script_setup.expect("script setup document");

    assert!(script_setup.content.contains(
        "export { DefaultWidget, client, firstUrl, importedHelper, localLabel, nestedValue, \
         runtimeRest, tailItems, tailValue, utilities, 挨拶 };"
    ));
    for not_exported in [
        "api",
        "label",
        "importedIgnored",
        "remainingUrls",
        "SourceShape",
        "TypeOnly",
        "unused",
    ] {
        assert!(
            !script_setup
                .content
                .split("// Exports for template")
                .nth(1)
                .unwrap_or_default()
                .contains(not_exported),
            "{not_exported} must not be exported"
        );
    }

    insta::assert_snapshot!(script_setup.content.as_str());
}

#[test]
fn allocator_and_owned_generation_keep_semantic_exports_in_sync() {
    let descriptor = vize_atelier_sfc::parse_sfc(COMPLEX_BINDINGS_SFC, Default::default()).unwrap();
    let mut generator = VirtualCodeGenerator::new();
    let owned = generator.generate(&descriptor, "owned.vue");
    let allocator = Allocator::new();
    let allocated = generator.generate_with_allocator(&descriptor, "allocated.vue", &allocator);

    assert_eq!(
        owned.script_setup.unwrap().content,
        allocated.script_setup.unwrap().content
    );
}
