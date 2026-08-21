#![cfg(feature = "compile")]

#![allow(clippy::disallowed_macros)] // `insta::assert_snapshot!` expands to `format!`.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    compile_sfc_for_adapter, parse_sfc,
};

#[test]
fn script_setup_module_component_tag_prefers_import_over_same_name_prop() {
    let source = r#"<script setup lang="ts">
import Predefine from './components/predefine.vue'

defineProps<{ predefine?: string[] }>()
</script>

<template>
  <predefine v-if="predefine" ref="predefine" />
</template>"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            script: ScriptCompileOptions {
                is_ts: true,
                inline_template: false,
                ..Default::default()
            },
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    )
    .expect("compile module-mode SFC");

    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        result.code.as_str(),
        SourceType::ts().with_module(true),
    )
    .parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "module-mode output must parse as TypeScript: {:?}\n{}",
        parsed.diagnostics,
        result.code
    );

    insta::assert_snapshot!(result.code.as_str());
}
