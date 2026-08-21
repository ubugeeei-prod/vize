#![cfg(feature = "compile")]

#![allow(clippy::disallowed_macros)] // `insta::assert_snapshot!` expands to `format!`.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, compile_sfc_for_adapter, parse_sfc,
};

#[test]
fn imported_with_defaults_expression_reaches_runtime_prop_options() {
    let source = r#"<script setup lang="ts">
import { checkboxDefaults } from './checkbox-defaults'

interface Props {
  label?: string | boolean
  value?: string | boolean
}

const props = withDefaults(defineProps<Props>(), checkboxDefaults)
</script>

<template>
  <span v-if="props.value !== undefined">{{ props.label }}</span>
</template>"#;
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "ImportedDefaults.vue".into(),
            ..Default::default()
        },
    )
    .expect("parse SFC");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions::default(),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    )
    .expect("compile SFC with imported prop defaults");

    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        result.code.as_str(),
        SourceType::default().with_module(true),
    )
    .parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "module output must parse as JavaScript: {:?}\n{}",
        parsed.diagnostics,
        result.code
    );
    insta::assert_snapshot!(result.code.as_str());
}
