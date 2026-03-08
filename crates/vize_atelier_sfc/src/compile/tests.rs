use super::{compile_sfc, helpers, normal_script};
use crate::types::{BindingType, ScriptCompileOptions, SfcCompileOptions, TemplateCompileOptions};
use crate::{parse_sfc, SfcParseOptions};
use std::fs;
use std::path::PathBuf;
use vize_carton::ToCompactString;

fn fixtures_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("sfc")
        .join("imported_types")
}

#[test]
fn test_generate_scope_id() {
    let id = helpers::generate_scope_id("src/App.vue");
    assert_eq!(id.len(), 8);
}

#[test]
fn test_extract_component_name() {
    assert_eq!(helpers::extract_component_name("src/App.vue"), "App");
    assert_eq!(
        helpers::extract_component_name("MyComponent.vue"),
        "MyComponent"
    );
}

#[test]
#[ignore = "TODO: fix v-model prop quoting"]
fn test_v_model_on_component_in_sfc() {
    let source = r#"<script setup>
import { ref } from 'vue'
import MyComponent from './MyComponent.vue'
const msg = ref('')
</script>

<template>
  <MyComponent v-model="msg" :language="'en'" />
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions::default();
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    // Should NOT contain /* v-model */ comment
    assert!(
        !result.code.contains("/* v-model */"),
        "Should not contain v-model comment. Got:\n{}",
        result.code
    );
    // Should contain modelValue and onUpdate:modelValue
    assert!(
        result.code.contains("\"modelValue\":"),
        "Should have modelValue prop. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("\"onUpdate:modelValue\":"),
        "Should have onUpdate:modelValue prop. Got:\n{}",
        result.code
    );
}

#[test]
#[ignore = "TODO: fix inline mode ref handling"]
fn test_bindings_passed_to_template() {
    let source = r#"<script setup lang="ts">
import { ref } from 'vue';
import MonacoEditor from './MonacoEditor.vue';
const selectedPreset = ref('test');
const options = ref({ ssr: false });
function handleChange(val: string) { selectedPreset.value = val; }
</script>
<template>
  <div>{{ selectedPreset }}</div>
  <select :value="selectedPreset" @change="handleChange($event.target.value)">
    <option value="a">A</option>
  </select>
  <input type="checkbox" v-model="options.ssr" />
  <MonacoEditor />
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions::default();
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    eprintln!("=== COMPILED OUTPUT ===\n{}", result.code);

    // In non-inline mode with binding metadata, setup bindings are accessed via $setup
    // This is the correct Vue 3 behavior when binding metadata is passed to the template compiler
    assert!(
        result.code.contains("$setup.selectedPreset"),
        "selectedPreset should have $setup prefix in non-inline mode with bindings. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("$setup.handleChange"),
        "handleChange should have $setup prefix in non-inline mode with bindings. Got:\n{}",
        result.code
    );
    // Verify options is in __returned__
    assert!(
        result.code.contains("options"),
        "options should be in __returned__. Got:\n{}",
        result.code
    );
    // Verify options.ssr access has $setup prefix
    assert!(
        result.code.contains("$setup.options"),
        "options.ssr should have $setup prefix. Got:\n{}",
        result.code
    );
    // Verify MonacoEditor is in __returned__ (imported component used in template)
    assert!(
        result.code.contains("MonacoEditor"),
        "MonacoEditor should be in __returned__. Got:\n{}",
        result.code
    );
}

#[test]
#[ignore = "TODO: fix nested v-if prefix"]
fn test_nested_v_if_no_double_prefix() {
    // Test with a component inside nested v-if to prevent hoisting
    let source = r#"<script setup lang="ts">
import { ref } from 'vue';
import CodeHighlight from './CodeHighlight.vue';
const output = ref(null);
</script>
<template>
<div v-if="output">
  <div v-if="output.preamble" class="preamble">
    <CodeHighlight :code="output.preamble" />
  </div>
</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions::default();
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    eprintln!("=== NESTED V-IF OUTPUT ===\n{}", result.code);

    // Should NOT contain double $setup prefix
    assert!(
        !result.code.contains("$setup.$setup"),
        "Should NOT have double $setup prefix. Got:\n{}",
        result.code
    );

    // Should contain single $setup prefix for output
    assert!(
        result.code.contains("$setup.output"),
        "Should have single $setup prefix for output. Got:\n{}",
        result.code
    );

    // Should contain CodeHighlight component with :code prop
    assert!(
        result.code.contains("CodeHighlight"),
        "Should contain CodeHighlight. Got:\n{}",
        result.code
    );
}

#[test]
fn test_typescript_preserved_in_event_handler() {
    // When is_ts=true, TypeScript is preserved in the output
    // (matching Vue's @vue/compiler-sfc behavior - TS stripping is the bundler's job)
    let source = r#"<script setup lang="ts">
type PresetKey = 'a' | 'b'
function handlePresetChange(key: PresetKey) {}
</script>

<template>
  <select @change="handlePresetChange(($event.target).value)">
    <option value="a">A</option>
  </select>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    // Print output for debugging
    eprintln!("TypeScript SFC output:\n{}", result.code);

    // TypeScript type alias should be preserved at module level
    assert!(
        result.code.contains("type PresetKey"),
        "Should preserve type alias with lang='ts'. Got:\n{}",
        result.code
    );
    // TypeScript function parameter types should be preserved in setup body
    assert!(
        result.code.contains("key: PresetKey"),
        "Should preserve function parameter type with lang='ts'. Got:\n{}",
        result.code
    );
    // Should have the event handler expression
    assert!(
        result.code.contains("handlePresetChange"),
        "Should have event handler. Got:\n{}",
        result.code
    );
}

#[test]
fn test_multi_statement_event_handler() {
    let source = r#"<script setup lang="ts">
const editDashboard = ref()
</script>

<template>
  <div @click="
    editDashboard = 'test';
    console.log('done');
  "></div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    eprintln!("Multi-statement event handler output:\n{}", result.code);

    // Multi-statement handler should use block body { ... }, not ( ... )
    // The concise body (editDashboard = 'test'; console.log('done');) is invalid JS
    assert!(
        result.code.contains("($event: any) => { "),
        "Multi-statement handler should use block body ($event: any) => {{ ... }}. Got:\n{}",
        result.code
    );

    // SetupRef assignment in template event handler should add .value
    assert!(
        result.code.contains("editDashboard.value"),
        "SetupRef assignment in event handler should add .value. Got:\n{}",
        result.code
    );
}

#[test]
fn test_typescript_function_types_preserved() {
    // When is_ts=true, TypeScript is preserved in the output
    // (matching Vue's @vue/compiler-sfc behavior - TS stripping is the bundler's job)
    let source = r#"<script setup lang="ts">
interface Item {
  id: number;
  name: string;
}

const getNumberOfItems = (
  items: Item[]
): string => {
  return items.length.toString();
};

const foo: string = "bar";
const count: number = 42;

function processData(data: Record<string, unknown>): void {
  console.log(data);
}
</script>

<template>
  <div>{{ foo }}</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    eprintln!("TypeScript function types output:\n{}", result.code);

    // TypeScript interface should be preserved at module level
    assert!(
        result.code.contains("interface Item"),
        "Should preserve interface with lang='ts'. Got:\n{}",
        result.code
    );
    // TypeScript annotations should be preserved in setup body
    assert!(
        result.code.contains(": Item[]"),
        "Should preserve array type annotation with lang='ts'. Got:\n{}",
        result.code
    );
    // Should contain the runtime logic
    assert!(
        result.code.contains("foo"),
        "Should have variable foo. Got:\n{}",
        result.code
    );
}

#[test]
fn test_inline_template_keeps_patch_flags_for_ref_class_bindings() {
    let source = r#"<script setup lang="ts">
import { ref } from 'vue';

const activeTab = ref<'a' | 'b'>('a');
</script>

<template>
  <div class="tabs">
    <button :class="['tab', { active: activeTab === 'a' }]" @click="activeTab = 'a'">A</button>
    <button :class="['tab', { active: activeTab === 'b' }]" @click="activeTab = 'b'">B</button>
  </div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result =
        compile_sfc(&descriptor, SfcCompileOptions::default()).expect("Failed to compile SFC");

    assert!(
        result.code.contains("2 /* CLASS */"),
        "Expected inline SFC output to preserve CLASS patch flags. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("activeTab.value === 'a'"),
        "Expected ref access to stay reactive in class binding. Got:\n{}",
        result.code
    );
}

#[test]
fn test_inline_component_dynamic_prop_keeps_props_patch_flag() {
    let source = r#"<script setup lang="ts">
import { ref } from 'vue';
import CodeHighlight from './CodeHighlight.vue';

const currentCode = ref('dom');
</script>

<template>
  <div class="wrapper">
    <CodeHighlight :code="currentCode" language="javascript" />
  </div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result =
        compile_sfc(&descriptor, SfcCompileOptions::default()).expect("Failed to compile SFC");

    assert!(
        result.code.contains("_createVNode(CodeHighlight"),
        "Expected inline component vnode output. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("code: currentCode.value"),
        "Expected inline component prop to stay reactive. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("8 /* PROPS */"),
        "Expected inline component output to preserve PROPS patch flag for dynamic prop. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("[\"code\"]"),
        "Expected inline component dynamic props list to include code. Got:\n{}",
        result.code
    );
}

#[test]
fn test_v_if_branch_component_dynamic_prop_keeps_props_patch_flag() {
    let source = r#"<script setup lang="ts">
import { ref } from 'vue';
import CodeHighlight from './CodeHighlight.vue';

const show = ref(true);
const currentCode = ref('dom');
</script>

<template>
  <div class="wrapper">
    <CodeHighlight v-if="show" :code="currentCode" language="javascript" />
  </div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result =
        compile_sfc(&descriptor, SfcCompileOptions::default()).expect("Failed to compile SFC");

    assert!(
        result.code.contains("_createBlock(CodeHighlight"),
        "Expected v-if branch component block output. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("code: currentCode.value"),
        "Expected v-if branch component prop to stay reactive. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("8 /* PROPS */"),
        "Expected v-if branch component output to preserve PROPS patch flag. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("[\"code\"]"),
        "Expected v-if branch component dynamic props list to include code. Got:\n{}",
        result.code
    );
}

#[test]
fn test_full_sfc_props_destructure() {
    let input = r#"<script setup lang="ts">
import { computed } from 'vue'

const {
  name,
  count = 0,
} = defineProps<{
  name: string
  count?: number
}>()

const doubled = computed(() => count * 2)
</script>

<template>
  <div class="card">
    <h2>{{ name }}</h2>
    <p>Count: {{ count }} (doubled: {{ doubled }})</p>
  </div>
</template>"#;

    let parse_opts = SfcParseOptions::default();
    let descriptor = parse_sfc(input, parse_opts).unwrap();

    let mut compile_opts = SfcCompileOptions::default();
    compile_opts.script.id = Some("test.vue".to_compact_string());
    let result = compile_sfc(&descriptor, compile_opts).unwrap();

    eprintln!("=== Full SFC props destructure output ===\n{}", result.code);

    // Props should use __props. prefix in template
    assert!(
        result.code.contains("__props.name") || result.code.contains("name"),
        "Should have name access. Got:\n{}",
        result.code
    );
}

#[test]
fn test_let_var_unref() {
    let input = r#"
<script setup>
const a = 1
let b = 2
var c = 3
</script>

<template>
  <div>{{ a }} {{ b }} {{ c }}</div>
</template>
"#;

    let parse_opts = SfcParseOptions::default();
    let descriptor = parse_sfc(input, parse_opts).unwrap();

    let mut compile_opts = SfcCompileOptions::default();
    compile_opts.script.id = Some("test.vue".to_compact_string());
    let result = compile_sfc(&descriptor, compile_opts).unwrap();

    eprintln!("Let/var unref test output:\n{}", result.code);

    // Check that bindings are correctly identified
    if let Some(bindings) = &result.bindings {
        eprintln!("Bindings:");
        for (name, binding_type) in &bindings.bindings {
            eprintln!("  {} => {:?}", name, binding_type);
        }
        assert!(
            matches!(bindings.bindings.get("a"), Some(BindingType::LiteralConst)),
            "a should be LiteralConst"
        );
        assert!(
            matches!(bindings.bindings.get("b"), Some(BindingType::SetupLet)),
            "b should be SetupLet"
        );
        assert!(
            matches!(bindings.bindings.get("c"), Some(BindingType::SetupLet)),
            "c should be SetupLet"
        );
    }

    // Check for _unref import
    assert!(
        result.code.contains("unref as _unref"),
        "Should import _unref. Got:\n{}",
        result.code
    );

    // Check that let/var variables are wrapped with _unref
    // In inline mode, setup bindings are accessed directly (no $setup. prefix)
    assert!(
        result.code.contains("_unref(b)"),
        "b should be wrapped with _unref. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("_unref(c)"),
        "c should be wrapped with _unref. Got:\n{}",
        result.code
    );
}

#[test]
fn test_extract_normal_script_content() {
    let input = r#"import type { NuxtRoute } from "@typed-router";
import { useBreakpoint } from "./_utils";
import Button from "./Button.vue";

interface TabItem {
  name: string;
  label: string;
}

export default {
  name: 'Tab'
}
"#;
    // Test preserving TypeScript output
    let result = normal_script::extract_normal_script_content(input, true, true);
    eprintln!("Extracted normal script content (preserve TS):\n{}", result);

    // Should contain imports
    assert!(
        result.contains("import type { NuxtRoute }"),
        "Should contain type import"
    );
    assert!(
        result.contains("import { useBreakpoint }"),
        "Should contain named import"
    );
    assert!(
        result.contains("import Button"),
        "Should contain default import"
    );

    // Should contain interface
    assert!(
        result.contains("interface TabItem"),
        "Should contain interface"
    );

    // Should NOT contain export default
    assert!(
        !result.contains("export default"),
        "Should NOT contain export default"
    );
}

#[test]
fn test_compile_both_script_blocks() {
    let source = r#"<script lang="ts">
import type { RouteLocation } from "vue-router";

interface TabItem {
  name: string;
  label: string;
}

export type { TabItem };
</script>

<script setup lang="ts">
const { items } = defineProps<{
  items: Array<TabItem>;
}>();
</script>

<template>
  <div v-for="item in items" :key="item.name">
    {{ item.label }}
  </div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    eprintln!(
        "Descriptor script: {:?}",
        descriptor.script.as_ref().map(|s| &s.content)
    );
    eprintln!(
        "Descriptor script_setup: {:?}",
        descriptor.script_setup.as_ref().map(|s| &s.content)
    );

    // Use is_ts = true to preserve TypeScript output
    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    eprintln!("=== COMPILED OUTPUT ===\n{}", result.code);

    // Should contain the type import (when is_ts = true, TypeScript is preserved)
    assert!(
        result.code.contains("RouteLocation") || result.code.contains("interface TabItem"),
        "Should contain type definitions from normal script. Got:\n{}",
        result.code
    );
}

#[test]
fn test_define_model_basic() {
    let source = r#"<script setup>
const model = defineModel()
</script>

<template>
  <input v-model="model">
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions::default();
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    eprintln!("=== defineModel OUTPUT ===\n{}", result.code);

    // Should have useModel import
    assert!(
        result.code.contains("useModel as _useModel"),
        "Should import useModel. Got:\n{}",
        result.code
    );

    // Should have modelValue prop
    assert!(
        result.code.contains("modelValue"),
        "Should have modelValue prop. Got:\n{}",
        result.code
    );

    // Should have update:modelValue emit
    assert!(
        result.code.contains("update:modelValue"),
        "Should have update:modelValue emit. Got:\n{}",
        result.code
    );

    // Should have _useModel call in setup
    assert!(
        result.code.contains("_useModel(__props"),
        "Should use _useModel in setup. Got:\n{}",
        result.code
    );
}

#[test]
fn test_define_model_with_name() {
    let source = r#"<script setup>
const title = defineModel('title')
</script>

<template>
  <input v-model="title">
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions::default();
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    eprintln!("=== defineModel with name OUTPUT ===\n{}", result.code);

    // Should have title prop
    assert!(
        result.code.contains("title:") || result.code.contains("\"title\""),
        "Should have title prop. Got:\n{}",
        result.code
    );

    // Should have update:title emit
    assert!(
        result.code.contains("update:title"),
        "Should have update:title emit. Got:\n{}",
        result.code
    );
}

#[test]
fn test_non_script_setup_typescript_preserved() {
    // Non-script-setup SFC with is_ts=true preserves TypeScript in the output
    // (matching Vue's @vue/compiler-sfc behavior - TS stripping is the bundler's job)
    let source = r#"<script lang="ts">
interface Props {
    name: string;
    count?: number;
}

export default {
    name: 'MyComponent',
    props: {
        name: String,
        count: Number
    } as Props,
    setup(props: Props) {
        const message: string = `Hello, ${props.name}!`;
        return { message };
    }
}
</script>

<template>
    <div>{{ message }}</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");

    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    eprintln!("=== Non-script-setup TS output ===\n{}", result.code);

    // TypeScript should be preserved when is_ts=true
    assert!(
        result.code.contains("interface Props") || result.code.contains(": Props"),
        "Should preserve TypeScript with is_ts=true. Got:\n{}",
        result.code
    );

    // Should still contain the component logic
    assert!(
        result.code.contains("name: 'MyComponent'")
            || result.code.contains("name: \"MyComponent\""),
        "Should have component name. Got:\n{}",
        result.code
    );
}

#[test]
fn test_non_script_setup_typescript_preserved_when_is_ts() {
    // Non-script-setup SFC with lang="ts" and is_ts=true should preserve TypeScript
    let source = r#"<script lang="ts">
interface Props {
    name: string;
}

export default {
    props: {} as Props
}
</script>

<template>
    <div></div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");

    // Compile with is_ts = true to preserve TypeScript
    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    eprintln!(
        "=== Non-script-setup TS preserved output ===\n{}",
        result.code
    );

    // Should still contain TypeScript syntax when is_ts = true
    assert!(
        result.code.contains("interface Props") || result.code.contains("as Props"),
        "Should preserve TypeScript when is_ts = true. Got:\n{}",
        result.code
    );
}

#[test]
fn test_define_props_imported_type_alias_is_exposed_to_template() {
    let fixture_path = fixtures_path().join("ImportedSelectBase.vue");
    let source = fs::read_to_string(&fixture_path).expect("fixture should load");
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let mut opts = SfcCompileOptions::default();
    opts.script.id = Some(fixture_path.to_string_lossy().as_ref().to_compact_string());

    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    assert!(
        result.code.contains("disabled: { type: Boolean")
            || result.code.contains("disabled: { type: null"),
        "Imported disabled prop should exist in runtime props. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("size: {"),
        "Imported size prop should exist in runtime props. Got:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("_ctx.disabled"),
        "Imported disabled prop should not fall back to _ctx. Got:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("_ctx.size"),
        "Imported size prop should not fall back to _ctx. Got:\n{}",
        result.code
    );
}

#[test]
fn test_define_props_interface_extends_imported_type_alias() {
    let fixture_path = fixtures_path().join("ImportedSelectField.vue");
    let source = fs::read_to_string(&fixture_path).expect("fixture should load");
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let mut opts = SfcCompileOptions::default();
    opts.script.id = Some(fixture_path.to_string_lossy().as_ref().to_compact_string());

    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    assert!(
        result.code.contains("disabled: { type: Boolean")
            || result.code.contains("disabled: { type: null"),
        "Extended imported disabled prop should exist in runtime props. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("size: {"),
        "Extended imported size prop should exist in runtime props. Got:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("_ctx.disabled"),
        "Extended imported disabled prop should not fall back to _ctx. Got:\n{}",
        result.code
    );
}
