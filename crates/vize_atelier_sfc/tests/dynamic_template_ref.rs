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
fn module_mode_marks_dynamic_refs_in_v_for_as_ref_for() {
    let source = r#"<script setup lang="ts">
import { ref } from 'vue'

type Menu = { id: number }

const menus = ref<Menu[][]>([[{ id: 1 }]])
const menuList = ref<unknown[]>([])
</script>

<template>
  <Child
    v-for="(menu, index) in menus"
    :key="index"
    :ref="(item) => (menuList[index] = item as unknown)"
    :nodes="[...menu]"
  />
</template>"#;
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "CascaderPanel.vue".into(),
            ..Default::default()
        },
    )
    .expect("parse SFC");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            script: ScriptCompileOptions {
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
        SourceType::default().with_module(true),
    )
    .parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "module-mode output must parse as JavaScript: {:?}\n{}",
        parsed.diagnostics,
        result.code
    );

    insta::assert_snapshot!(result.code.as_str());
}
