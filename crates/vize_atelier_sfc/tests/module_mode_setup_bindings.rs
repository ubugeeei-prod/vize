#![expect(
    clippy::disallowed_macros,
    reason = "`insta::assert_snapshot!` expands to `format!`"
)]
#![expect(
    clippy::expect_used,
    reason = "test setup failures should stop the case"
)]

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    compile_sfc_for_adapter, parse_sfc,
};
use vize_carton::String;

fn compile_module(
    source: &str,
    filename: &str,
    inline_template: bool,
    output_mode: SfcScriptOutputMode,
) -> String {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        },
    )
    .expect("parse SFC");
    compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            script: ScriptCompileOptions {
                inline_template,
                ..Default::default()
            },
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        output_mode,
    )
    .expect("compile SFC")
    .code
}

fn assert_javascript_function_body(code: &str) {
    let allocator = Allocator::default();
    let wrapped = format!("function __compiled_sfc() {{\n{code}\n}}\n");
    let parsed = Parser::new(
        &allocator,
        &wrapped,
        SourceType::default().with_module(true),
    )
    .parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "output must parse as a JavaScript function body: {:?}\n{code}",
        parsed.diagnostics
    );
}

#[test]
fn setup_only_bare_define_emits_does_not_return_synthetic_state() {
    let code = compile_module(
        "<script setup>\ndefineEmits(['destroy'])\n</script>",
        "SetupOnlyEmits.vue",
        true,
        SfcScriptOutputMode::InlineTemplate,
    );
    assert_javascript_function_body(&code);
    insta::assert_snapshot!(code.as_str());
}

#[test]
fn inline_mode_keeps_bare_define_emits_in_render_scope() {
    let code = compile_module(
        r#"<script setup>
defineEmits(['destroy'])
</script>
<template><button @click="$emit('destroy')" /></template>"#,
        "InlineEmits.vue",
        true,
        SfcScriptOutputMode::InlineTemplate,
    );
    assert_javascript_function_body(&code);
    insta::assert_snapshot!(code.as_str());
}

#[test]
fn inline_mode_preserves_authored_dollar_emit() {
    let code = compile_module(
        r#"<script setup>
const $emit = () => 'authored'
defineEmits(['destroy'])
</script>
<template>{{ $emit() }}</template>"#,
        "AuthoredDollarEmit.vue",
        true,
        SfcScriptOutputMode::InlineTemplate,
    );
    assert_javascript_function_body(&code);
    insta::assert_snapshot!(code.as_str());
}

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

#[test]
fn module_mode_does_not_keep_type_import_for_prefixed_component_name() {
    let source = r#"<script setup lang="ts">
import { Item } from './types'
import ItemTab from './ItemTab.vue'
const { item = {} as Item } = defineProps<{ item?: Item }>()
</script>
<template><ItemTab :label="item.label ?? 'none'" /></template>"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions::default(),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    )
    .expect("compile module-mode SFC");

    insta::assert_snapshot!(result.code.as_str());
}
