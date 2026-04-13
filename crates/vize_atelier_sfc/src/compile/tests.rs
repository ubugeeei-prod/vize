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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    // Check that bindings are correctly identified
    if let Some(bindings) = &result.bindings {
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

    insta::assert_snapshot!(result.code.as_str());
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
    insta::assert_snapshot!(result.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
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

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_define_props_imported_type_alias_is_exposed_to_template() {
    let fixture_path = fixtures_path().join("ImportedSelectBase.vue");
    let source = fs::read_to_string(&fixture_path).expect("fixture should load");
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let mut opts = SfcCompileOptions::default();
    opts.script.id = Some(fixture_path.to_string_lossy().as_ref().to_compact_string());

    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_define_props_interface_extends_imported_type_alias() {
    let fixture_path = fixtures_path().join("ImportedSelectField.vue");
    let source = fs::read_to_string(&fixture_path).expect("fixture should load");
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let mut opts = SfcCompileOptions::default();
    opts.script.id = Some(fixture_path.to_string_lossy().as_ref().to_compact_string());

    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_template_only_sfc_vapor_output_mode() {
    let source = r#"<template><div>{{ msg }}</div></template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_script_setup_sfc_vapor_output_mode() {
    let source = r#"<script setup lang="ts">
import { computed, ref } from 'vue'

const count = ref(1)
const doubled = computed(() => count.value * 2)
</script>

<template>
  <div>{{ count }} {{ doubled }}</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
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

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_script_setup_sfc_ssr_uses_server_renderer_output() {
    let source = r#"<script setup lang="ts">
const msg = 'hello'
</script>

<template>
  <div>{{ msg }}</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            ssr: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_script_setup_sfc_ssr_uses_setup_bindings_for_components_and_slots() {
    let source = r##"<script setup lang="ts">
import { NuxtLayout, NuxtPage } from "#components"
</script>

<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>"##;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            ssr: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_normal_script_sfc_ssr_attaches_ssr_render() {
    let source = r#"<script lang="ts">
export default {
  name: 'HelloSsr'
}
</script>

<template>
  <div>Hello</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            ssr: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_template_only_sfc_ssr_exports_default_component() {
    let source = r#"<template>
  <div>Hello</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        template: TemplateCompileOptions {
            ssr: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_script_setup_sfc_vapor_output_avoids_local_render_collision() {
    let source = r#"<script setup lang="ts">
function render() {
  return 'local'
}
</script>

<template>
  <div>Hello</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
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

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_script_setup_sfc_vapor_output_keeps_render_block_statements() {
    let source = r#"<script setup lang="ts">
import { ref } from 'vue'

const count = ref(1)
</script>

<template>
  <div>{{ count }}</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
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

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_script_setup_sfc_vapor_uses_ctx_bindings_for_imported_components() {
    let source = r#"<script setup lang="ts">
import FooPanel from './FooPanel.vue'
</script>

<template>
  <FooPanel />
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
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

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_normal_script_sfc_vapor_output_mode() {
    let source = r#"<script>
export default {
  name: 'NormalVapor'
}
</script>

<template>
  <div>Hello</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}
