use super::super::compile_sfc;
use crate::types::{BindingType, SfcCompileOptions};
use crate::{SfcParseOptions, parse_sfc};

#[test]
fn test_define_emits_quoted_update_event_in_sfc() {
    let source = r#"<script setup lang="ts">
const emit = defineEmits<{
  "update:open": [value: boolean]
}>()
</script>

<template>
  <button @click="emit('update:open', false)">close</button>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions::default();
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    assert!(
        result.code.contains(r#"emits: ["update:open"]"#),
        "quoted emits keys containing ':' must be preserved:\n{}",
        result.code
    );
}

#[test]
fn test_type_based_define_props_partial_destructure_keeps_template_props() {
    let source = r#"<script setup lang="ts">
const { a, b } = defineProps<{ label: string, a: string, b: string }>()
</script>

<template>
  {{ label }} - {{ a }} - {{ b }}
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("compile");

    assert!(
        result.code.contains("__props.label"),
        "non-destructured props should remain prop accesses:\n{}",
        result.code
    );
    assert!(
        result.code.contains("__props.a") && result.code.contains("__props.b"),
        "destructured props should remain reactive prop accesses:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("_ctx.label"),
        "type-only props must not fall back to instance context:\n{}",
        result.code
    );
}

#[test]
fn test_script_setup_deep_destructure_bindings_are_available_to_template() {
    let source = r#"<script setup lang="ts">
const {
  public: { contactFormUrl },
  nested: { label: inquiryLabel = "Inquiry" },
  urls: [firstUrl, { href: secondUrl }],
  ...runtimeRest
} = useRuntimeConfig()
</script>

<template>
  <a
    :href="contactFormUrl"
    :aria-label="inquiryLabel"
    :data-first="firstUrl"
    :data-second="secondUrl"
    :data-rest="runtimeRest"
  >
    {{ inquiryLabel }}
  </a>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result =
        compile_sfc(&descriptor, SfcCompileOptions::default()).expect("Failed to compile SFC");

    let bindings = result
        .bindings
        .as_ref()
        .expect("script setup output should include bindings");
    for name in [
        "contactFormUrl",
        "inquiryLabel",
        "firstUrl",
        "secondUrl",
        "runtimeRest",
    ] {
        assert!(
            matches!(
                bindings.bindings.get(name),
                Some(BindingType::SetupMaybeRef)
            ),
            "{name} should be collected from the deep destructure pattern"
        );
        assert!(
            !result.code.contains(&format!("_ctx.{name}")),
            "{name} should be compiled as a setup binding, not as an instance property:\n{}",
            result.code
        );
    }
}

#[test]
fn test_template_ternary_vbind_preserves_optional_chaining() {
    let source = r#"<script setup lang="ts">
const external = false
const to = "/login"
</script>

<template>
  <NuxtLinkLocale v-slot="scope" :to="to">
    <slot v-bind="external ? { isActive: undefined } : { isActive: scope?.isActive }" />
  </NuxtLinkLocale>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result =
        compile_sfc(&descriptor, SfcCompileOptions::default()).expect("Failed to compile SFC");

    assert!(
        result
            .code
            .contains("external ? { isActive: undefined } : { isActive: scope?.isActive }"),
        "template ternary v-bind must preserve optional chaining:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("{ isActive: scope.isActive }"),
        "template ternary v-bind must not emit an unguarded member access:\n{}",
        result.code
    );
}
