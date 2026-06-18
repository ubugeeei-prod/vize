use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc,
    parse_sfc,
};

const SOURCE: &str = r#"<script setup lang="ts">
defineProps<{ href?: string }>()
</script>

<template>
  <RouterLink v-slot="{ href }">
    <slot v-bind="{ href }" />
  </RouterLink>
</template>"#;

#[test]
fn script_setup_prop_does_not_shadow_scoped_slot_outlet_vbind() {
    let descriptor = parse_sfc(SOURCE, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
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
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    assert!(
        result.code.contains("href: href"),
        "slot outlet should forward RouterLink's scoped href:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("__props.href")
            && !result.code.contains("$props.href")
            && !result.code.contains("_ctx.href"),
        "slot outlet href must not resolve to component props:\n{}",
        result.code
    );
}

#[test]
fn script_setup_prop_does_not_shadow_scoped_slot_outlet_vbind_in_ssr() {
    let descriptor = parse_sfc(SOURCE, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            ssr: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    assert!(
        result.code.contains("href: href"),
        "SSR slot outlet should forward RouterLink's scoped href:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("__props.href")
            && !result.code.contains("$props.href")
            && !result.code.contains("_ctx.href"),
        "SSR slot outlet href must not resolve to component props:\n{}",
        result.code
    );
}
