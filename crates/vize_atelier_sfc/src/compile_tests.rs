//! SFC compilation: macros, HTML validation, and `v-for` lowering.
//!
//! Split out of `lib.rs` so that module stays inside the per-file
//! source-length budget.

use super::{SfcCompileOptions, compile_sfc, compile_sfc_with_template_syntax, parse_sfc};
use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::config::VueVersion;

#[test]
fn test_compile_sfc_with_define_emits() {
    let source = r#"
<template>
  <button @click="onClick">{{ count }}</button>
</template>

<script setup>
import { ref } from 'vue'
const emit = defineEmits(['update'])
const count = ref(0)
function onClick() {
    emit('update', count.value)
}
</script>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
    // `@vue/compiler-sfc` keeps the defineEmits runtime argument verbatim
    // (single quotes from source), it does not re-serialize to double quotes.
    assert!(
        result.code.contains(r#"emits: ['update']"#),
        "unexpected code:\n{}",
        result.code
    );
    assert!(
        result.code.contains("const emit = __emit"),
        "unexpected code:\n{}",
        result.code
    );
    assert!(
        result.code.contains("emit('update', count.value)")
            || result.code.contains("emit(\"update\", count.value)"),
        "unexpected code:\n{}",
        result.code
    );

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_compile_sfc_standard_warns_for_invalid_html_self_closing() {
    let source = r#"
<template>
  <div />
  <span></span>
</template>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();

    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.message.contains("Invalid self-closing syntax")),
        "expected invalid self-closing warning: {:?}",
        result.warnings
    );
    assert!(result.code.contains("_createElementVNode(\"div\""));
}

#[test]
fn test_compile_sfc_strict_errors_for_invalid_html_self_closing() {
    let source = r#"
<template>
  <div />
</template>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let error = compile_sfc_with_template_syntax(
        &descriptor,
        SfcCompileOptions::default(),
        TemplateSyntaxMode::Strict,
    )
    .expect_err("strict syntax should reject invalid self-closing HTML");

    assert!(error.message.contains("Invalid self-closing syntax"));
}

#[test]
fn test_compile_sfc_define_model_with_type_args_preserves_body() {
    // Regression test: defineModel<Type>('name', { opts }); was wrongly detected
    // as a multi-line macro call because the line ends with `;` not `)`.
    // This caused all subsequent setup code to be swallowed by the macro tracker.
    let source = r#"
<template>
  <div>{{ fx }}</div>
</template>

<script setup lang="ts">
interface Layer { fxId: string }
const layer = defineModel<Layer>('layer', { required: true });

const fx = layer.value.fxId;
if (fx == null) {
  throw new Error('not found');
}
</script>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_compile_sfc_imported_component_numeric_v_for() {
    let source = r#"
<template>
  <Child
    v-for="(id, index) in 4"
    :key="id"
    :label="String(index)"
  />
</template>

<script setup lang="ts">
import { Child } from "./components";
</script>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();

    assert!(
        result
            .code
            .contains(r#"import { Child } from "./components";"#),
        "script setup import should be preserved. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("_createBlock(_unref(Child)"),
        "numeric component v-for should render the imported component. Got:\n{}",
        result.code
    );
    assert!(
        !result.code.contains(r#"_createElementVNode("Child""#),
        "numeric component v-for must not render Child as a native element. Got:\n{}",
        result.code
    );
}

#[test]
fn test_compile_sfc_v_for_dynamic_prop_without_children_emits_null_children() {
    let source = r#"
<template>
  <div
    v-for="_, i in ary"
    :prop="val"
  ></div>
</template>

<script setup lang="ts">
import { ref } from "vue"
const ary = ref([])
const val = ref(0)
</script>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
    let normalized = result.code.replace('\n', " ");

    assert!(
        normalized.contains(r#"_createElementBlock("div", { prop: val.value }, null, 8"#),
        "v-for element with dynamic props and no children must emit a null children argument. Got:\n{}",
        result.code
    );
    assert!(
        !normalized.contains(r#"_createElementBlock("div", { prop: val.value }, 8"#),
        "patch flag must not occupy the children argument. Got:\n{}",
        result.code
    );
}
