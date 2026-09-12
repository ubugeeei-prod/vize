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
