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
fn module_mode_returns_script_setup_bindings_for_the_public_instance_proxy() {
    let source = r#"<script setup>
import { computed } from 'vue'

const nodes = [1, 2, 3]
const flattenTree = computed(() => nodes)

defineExpose({ nodes })
</script>

<template>
  <div v-for="node in flattenTree" :key="node">{{ node }}</div>
</template>"#;
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "ElementTree.vue".into(),
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

#[test]
fn module_mode_keeps_bare_define_emits_on_the_public_instance_proxy() {
    let source = r#"<script setup>
defineEmits(['destroy'])
</script>

<template>
  <Transition @after-leave="$emit('destroy')">
    <div />
  </Transition>
</template>"#;
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "Notification.vue".into(),
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

#[test]
fn module_mode_omits_imports_used_only_in_types() {
    let source = r#"<script setup lang="ts">
import { BadgeType } from './types'

interface Props {
  badges: BadgeType[]
}

const props = defineProps<Props>()
</script>

<template>
  <div>{{ props.badges.length }}</div>
</template>"#;
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "TypeOnlyImport.vue".into(),
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
