use crate::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    compile_sfc_for_adapter_with_experimental_options, parse_sfc,
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};

#[test]
fn test_compile_sfc_experimental_self_component_resolves_template_only() {
    let source = r#"
<template>
  <Self />
</template>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc_for_adapter_with_experimental_options(
        &descriptor,
        SfcCompileOptions {
            parse: SfcParseOptions {
                filename: "TreeNode.vue".into(),
                ..Default::default()
            },
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions {
            self_component: true,
        },
    )
    .unwrap();

    assert!(
        result
            .code
            .contains(r#"_resolveComponent("TreeNode", true)"#),
        "template-only SFC `<Self>` should resolve to the current component:\n{}",
        result.code
    );
}

#[test]
fn test_separate_template_preserves_user_render_binding() {
    let source = r#"
<script setup lang="ts">
import { render } from "./lib";
import { ref } from "vue";
const props = defineProps<{ text: string }>();
const html = ref("");
html.value = render(props.text);
</script>
<template><div v-html="html" /></template>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc_for_adapter_with_experimental_options(
        &descriptor,
        SfcCompileOptions {
            parse: SfcParseOptions {
                filename: "Foo.vue".into(),
                ..Default::default()
            },
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    )
    .unwrap();

    assert!(
        result.code.contains("import { render } from \"./lib\""),
        "user import must remain in the compiled module:\n{}",
        result.code
    );
    assert!(
        result.code.contains("function _sfc_render("),
        "generated client render needs its own name:\n{}",
        result.code
    );
    assert!(
        result.code.contains("render: _sfc_render"),
        "component must point to the renamed function:\n{}",
        result.code
    );
    assert!(
        result.code.contains("html.value = render(props.text)"),
        "setup must continue to call the user helper:\n{}",
        result.code
    );
}
