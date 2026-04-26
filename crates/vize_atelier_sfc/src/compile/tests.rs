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
fn test_script_setup_self_component_resolves_for_recursion() {
    let source = r#"<script setup lang="ts">
const items = [{ name: 'dist', children: [{ name: 'file.js', children: [] }] }]
</script>

<template>
  <ul>
    <li v-for="item in items" :key="item.name">
      <FileTree v-if="item.children.length" />
    </li>
  </ul>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let mut opts = SfcCompileOptions::default();
    opts.script.id = Some("src/components/diff/FileTree.vue".into());
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    assert!(
        result
            .code
            .contains(r#"_resolveComponent("FileTree", true)"#),
        "recursive SFC should resolve its own component name with maybeSelfReference. Got:\n{}",
        result.code
    );
}

#[test]
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
fn test_script_setup_sfc_demotes_reactive_const_used_in_v_model() {
    let source = r#"<template>
  <Comp v-model="reactiveObject" />
</template>

<script lang="ts" setup>
import { reactive } from 'vue';

const reactiveObject = reactive({ foo: 'bar' });
</script>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result =
        compile_sfc(&descriptor, SfcCompileOptions::default()).expect("Failed to compile SFC");

    assert!(
        result
            .code
            .contains("let reactiveObject = reactive({ foo: \"bar\" });"),
        "compiled output should demote the binding to let"
    );
    assert!(
        result.code.contains("reactiveObject = $event"),
        "compiled output should assign directly to the demoted binding"
    );
    assert_eq!(result.warnings.len(), 1, "expected exactly one warning");
    assert_eq!(
        result.warnings[0].code.as_deref(),
        Some("V_MODEL_CONST_REACTIVE_DEMOTED")
    );
    assert!(
        result.warnings[0]
            .message
            .contains("const reactive binding `reactiveObject`"),
        "warning should explain the reactive const demotion"
    );

    let bindings = result
        .bindings
        .as_ref()
        .expect("script setup output should include bindings");
    assert!(
        matches!(
            bindings.bindings.get("reactiveObject"),
            Some(BindingType::SetupLet)
        ),
        "reactiveObject should be exposed as SetupLet after demotion"
    );
}

#[test]
fn test_ssr_vapor_request_falls_back_with_warning() {
    let source = r#"<script setup>
const count = 1
</script>

<template>
  <div>{{ count }}</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            template: TemplateCompileOptions {
                ssr: true,
                ..Default::default()
            },
            vapor: true,
            ..Default::default()
        },
    )
    .expect("Failed to compile SFC");

    assert!(result.code.contains("ssrRender"));
    assert!(!result.code.contains("__vapor"));
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(
        result.warnings[0].code.as_deref(),
        Some("VAPOR_SSR_FALLBACK")
    );
    assert_eq!(
        result.warnings[0].message.as_str(),
        "SFC Vapor SSR is not supported yet; falling back to standard SSR output."
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
fn test_script_setup_typescript_downcompiles_to_javascript_by_default() {
    let source = r#"<script setup lang="ts">
const props = withDefaults(defineProps<{
  first?: boolean;
}>(), {
  first: false,
});

async function updatePasswordLessLogin(value: boolean): Promise<void> {
  console.log(value);
}
</script>

<template>
  <div>{{ props.first }}</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result =
        compile_sfc(&descriptor, SfcCompileOptions::default()).expect("Failed to compile SFC");

    assert!(
        result.code.contains("setup(__props)"),
        "default JS output should not preserve typed setup params: {}",
        result.code
    );
    assert!(
        !result.code.contains("__props: any"),
        "default JS output should strip typed setup params: {}",
        result.code
    );
    assert!(
        !result.code.contains("(_ctx: any,_cache: any)"),
        "default JS output should strip typed render params: {}",
        result.code
    );
    assert!(
        !result.code.contains(": Promise<void>"),
        "default JS output should strip TypeScript return types: {}",
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
fn test_script_setup_sfc_uses_setup_bindings_for_kebab_case_imported_components() {
    let source = r#"<script setup lang="ts">
import DashTest from './dash-test.vue'
</script>

<template>
  <dash-test />
  <DashTest />
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
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
fn test_script_setup_sfc_uses_setup_member_bindings_for_dotted_components() {
    let source = r#"<script setup lang="ts">
import { Form, Input } from 'ant-design-vue'
</script>

<template>
  <Form.Item label="Teacher">
    <Input />
  </Form.Item>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
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
fn test_script_setup_sfc_ssr_uses_setup_bindings_for_kebab_case_imported_components() {
    let source = r#"<script setup lang="ts">
import DashTest from './dash-test.vue'
</script>

<template>
  <dash-test />
  <DashTest />
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
fn test_script_setup_sfc_ssr_uses_setup_member_bindings_for_dotted_components() {
    let source = r#"<script setup lang="ts">
import { Form, Input } from 'ant-design-vue'
</script>

<template>
  <Form.Item label="Teacher">
    <Input />
  </Form.Item>
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
fn test_script_setup_sfc_ssr_uses_setup_bindings_for_lowercase_imported_components() {
    let source = r#"<script setup lang="ts">
import { Primitive } from '@tresjs/core'
</script>

<template>
  <primitive />
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
fn test_script_setup_sfc_ssr_returns_template_only_imports_used_in_expressions() {
    let source = r#"<script setup lang="ts">
import { valibotResolver } from '@primevue/forms/resolvers/valibot'
const schema = {}
</script>

<template>
  <Form :resolver="schema ? valibotResolver(schema) : undefined" />
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

    assert!(result.code.contains("resolver:"), "{}", result.code);
    assert!(
        result
            .code
            .contains("_unref($setup.valibotResolver)($setup.schema)"),
        "{}",
        result.code
    );
    assert!(
        result
            .code
            .contains("const __returned__ = { valibotResolver, schema }"),
        "{}",
        result.code
    );
    assert!(
        result
            .code
            .contains("Object.defineProperty(__returned__, '__isScriptSetup'"),
        "{}",
        result.code
    );
}

#[test]
fn test_script_setup_sfc_ssr_returns_normal_script_imports_used_in_template_expressions() {
    let source = r#"<script lang="ts">
import {
  type FormFieldState,
  Form as PForm,
} from '@primevue/forms'
import { valibotResolver } from '@primevue/forms/resolvers/valibot'

export interface FormProps {
  schema?: unknown
}
</script>

<script setup lang="ts">
const { schema } = defineProps<FormProps>()
const emit = defineEmits<{ submit: [] }>()
</script>

<template>
  <PForm :resolver="schema ? valibotResolver(schema) : undefined" @submit="emit('submit')" />
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
    let setup_return = result
        .code
        .split("const __returned__ = {")
        .nth(1)
        .expect("setup should return bindings");

    assert!(setup_return.contains("emit"), "{}", result.code);
    assert!(setup_return.contains("PForm"), "{}", result.code);
    assert!(setup_return.contains("valibotResolver"), "{}", result.code);
    assert!(
        result
            .code
            .contains("Object.defineProperty(__returned__, '__isScriptSetup'"),
        "{}",
        result.code
    );
    assert!(
        result
            .code
            .contains("$setup.valibotResolver($props.schema)")
            || result
                .code
                .contains("_unref($setup.valibotResolver)($props.schema)"),
        "{}",
        result.code
    );
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
fn test_script_setup_sfc_vapor_uses_ctx_bindings_for_lowercase_imported_components() {
    let source = r#"<script setup lang="ts">
import { Primitive } from '@tresjs/core'
</script>

<template>
  <primitive />
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
fn test_script_setup_sfc_vapor_custom_renderer_preserves_intrinsics_and_lowercase_imports() {
    let source = r#"<script setup lang="ts">
import { Primitive } from '@tresjs/core'
const visible = true
</script>

<template>
  <mesh>
    <group v-if="visible">
      <primitive />
    </group>
  </mesh>
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
            custom_renderer: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_script_setup_sfc_vapor_ssr_custom_renderer_falls_back_without_losing_intrinsics() {
    let source = r#"<script setup lang="ts" vapor>
import { Primitive } from '@tresjs/core'
const visible = true
</script>

<template>
  <mesh>
    <group v-if="visible">
      <primitive />
    </group>
  </mesh>
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
            custom_renderer: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    assert_eq!(result.warnings.len(), 1);
    assert_eq!(
        result.warnings[0].code.as_deref(),
        Some("VAPOR_SSR_FALLBACK")
    );
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
