use super::super::compile_sfc;
use crate::types::{
    BindingType, ScriptCompileOptions, SfcCompileOptions, SfcCompileResult, TemplateCompileOptions,
};
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

#[test]
fn test_with_defaults_rejects_setup_local_default_reference() {
    let source = r#"<script setup lang="ts">
const items = []

withDefaults(defineProps<{
  items?: string[]
}>(), { items })
</script>

<template>{{ items.join() }}</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse reported SFC");
    let error = compile_sfc(&descriptor, SfcCompileOptions::default())
        .expect_err("compiler must reject a setup-local withDefaults reference");

    assert_eq!(error.code.as_deref(), Some("SCRIPT_SETUP_MACRO_SCOPE"));
    assert!(error.message.contains("`withDefaults()`"), "{error:?}");
    assert!(error.message.contains("`items`"), "{error:?}");
    let loc = error
        .loc
        .expect("compiler error should point at the reference");
    assert_eq!(&source[loc.start..loc.end], "items");
    assert_eq!((loc.start_line, loc.start_column), (6, 9));
    assert_eq!((loc.end_line, loc.end_column), (6, 14));
}

fn compile_vapor_ts_sfc(source: &str) -> SfcCompileResult {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        ..Default::default()
    };
    compile_sfc(&descriptor, opts).expect("Failed to compile SFC")
}

// Regression test for #3072: a destructured prop is compiled out of the setup
// return object, so the Vapor render must read it through `$props`, not `_ctx`.
#[test]
fn test_script_setup_sfc_vapor_props_destructure_reads_dollar_props() {
    let result = compile_vapor_ts_sfc(
        r#"<script setup lang="ts">
const { tag = "div" } = defineProps<{ tag?: string }>();
</script>

<template>
  <component :is="tag" data-probe>hi</component>
</template>"#,
    );

    insta::assert_snapshot!(result.code.as_str());
}

// Regression test for #3072: an aliased destructured prop resolves through the
// original prop key on `$props`.
#[test]
fn test_script_setup_sfc_vapor_aliased_props_destructure_reads_prop_key() {
    let result = compile_vapor_ts_sfc(
        r#"<script setup lang="ts">
const { tag: theTag = "div" } = defineProps<{ tag?: string }>();
</script>

<template>
  <component :is="theTag" data-probe>hi</component>
</template>"#,
    );

    insta::assert_snapshot!(result.code.as_str());
}

// Regression test for #3072 (comment): `props.x` template references stay on
// the setup context, and `props` is part of the setup return object.
#[test]
fn test_script_setup_sfc_vapor_props_object_reference_stays_on_ctx() {
    let result = compile_vapor_ts_sfc(
        r#"<script setup lang="ts">
const props = defineProps<{ itemKey: (item: string) => PropertyKey }>();
</script>

<template>
  <li v-for="item in ['a']" :key="props.itemKey(item)" />
</template>"#,
    );

    insta::assert_snapshot!(result.code.as_str());
}
