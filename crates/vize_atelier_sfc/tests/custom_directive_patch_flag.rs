use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

fn compile(source: &str) -> vize_carton::String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    compile_sfc(&descriptor, SfcCompileOptions::default())
        .expect("compile SFC")
        .code
}

#[test]
fn custom_directive_with_children_keeps_need_patch() {
    let code = compile(
        r#"<script setup>
import { ref } from 'vue'
const value = ref('first')
</script>

<template>
  <div v-track="value">content</div>
</template>"#,
    );

    assert!(
        code.contains(r#"const _directive_track = _resolveDirective("track")"#),
        "custom directive must remain registered:\n{code}"
    );
    assert!(
        code.contains("512 /* NEED_PATCH */"),
        "custom directive vnode must be revisited when its value changes:\n{code}"
    );
    assert!(
        code.contains("[_directive_track, value.value]"),
        "custom directive must receive the current setup-ref value:\n{code}"
    );
}

#[test]
fn built_in_v_show_with_children_keeps_its_runtime_path() {
    let code = compile(
        r#"<script setup>
import { ref } from 'vue'
const visible = ref(true)
</script>

<template>
  <div v-show="visible">content</div>
</template>"#,
    );

    assert!(
        code.contains("vShow as _vShow"),
        "v-show must keep using the built-in runtime helper:\n{code}"
    );
    assert!(
        code.contains("[_vShow, visible.value]"),
        "v-show must receive the current setup-ref value:\n{code}"
    );
    assert!(
        code.contains("512 /* NEED_PATCH */"),
        "v-show vnode must remain patchable:\n{code}"
    );
    assert!(
        !code.contains("_resolveDirective(\"show\")"),
        "v-show must not be treated as a custom directive:\n{code}"
    );
}
