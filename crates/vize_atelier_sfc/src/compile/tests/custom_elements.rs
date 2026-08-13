//! Custom-element tag pattern coverage for SFC compilation.
//!
//! Kept separate from `tests.rs` so that already large file does not grow past
//! the source-file-length limit.

use crate::{
    SfcParseOptions, parse_sfc,
    types::{ScriptCompileOptions, SfcCompileOptions, TemplateCompileOptions},
};

use super::super::compile_sfc_with_custom_elements_template_syntax_and_codegen_options;

#[test]
fn script_setup_preserves_imports_and_pascal_case_intrinsics() {
    let source = r#"<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
const visible = true
</script>

<template>
  <TresCanvas>
    <TresMesh v-if="visible">
      <TresSpotLight></TresSpotLight>
    </TresMesh>
  </TresCanvas>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            custom_renderer: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc_with_custom_elements_template_syntax_and_codegen_options(
        &descriptor,
        opts,
        vize_atelier_core::TemplateSyntaxMode::Standard,
        vize_atelier_core::options::CustomElementMatcher::from_patterns(vec!["Tres*".into()]),
        vize_atelier_core::CodegenOptions::default(),
    )
    .expect("Failed to compile SFC");

    assert!(
        result.code.contains(r#"_createBlock(_unref(TresCanvas)"#),
        "{}",
        result.code
    );
    assert!(result.code.contains("import { TresCanvas }"));
    assert!(
        result.code.contains(r#"_createElementBlock("TresMesh""#),
        "{}",
        result.code
    );
    assert!(
        result
            .code
            .contains(r#"_createElementVNode("TresSpotLight""#),
        "{}",
        result.code
    );
    assert!(!result.code.contains(r#"_resolveComponent("TresCanvas")"#));
    assert!(!result.code.contains(r#"_resolveComponent("TresMesh")"#));
    assert!(
        !result
            .code
            .contains(r#"_resolveComponent("TresSpotLight")"#)
    );
}
