use vize_carton::String;

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

const APP: &str = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>
<template>
  <MissingWidget />
  <Child extra="1" />
  <div v-not-registered="1" />
</template>
"#;

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ label: string }>()
</script>
<template><span>{{ label }}</span><i /></template>
"#;

/// This fixture opts into the registries and a closed element map. The shared
/// Vue stub leaves both absent: a missing `GlobalComponents` stays an ignored
/// `TS2694`, and an open `NativeElements` index would accept `extra`.
const REGISTRIES: &str = r#"
export interface GlobalComponents {}
export interface GlobalDirectives {}
"#;
const CLOSED_NATIVE_ELEMENTS: &str = "\
export interface NativeElements { span: { id?: string }; i: { id?: string } }";

fn tsconfig(vue_compiler_options: &str) -> String {
    if vue_compiler_options.is_empty() {
        return vize_carton::cstr!(
            r#"{{
  "compilerOptions": {{ "strict": true, "moduleResolution": "bundler" }},
  "include": ["src/**/*"]
}}
"#,
        );
    }
    vize_carton::cstr!(
        r#"{{
  "compilerOptions": {{ "strict": true, "moduleResolution": "bundler" }},
  "vueCompilerOptions": {{ {vue_compiler_options} }},
  "include": ["src/**/*"]
}}
"#
    )
}

fn diagnostics_for(
    name: &str,
    vue_compiler_options: &str,
) -> Option<Vec<(String, Option<u32>, String)>> {
    let root = create_project_case(name, &[("src/Child.vue", CHILD), ("src/App.vue", APP)]);
    std::fs::write(
        root.join("tsconfig.json"),
        tsconfig(vue_compiler_options).as_str(),
    )
    .unwrap();
    let runtime = root.join("node_modules/@vue/runtime-dom/index.d.ts");
    let mut types = std::fs::read_to_string(&runtime).unwrap();
    types = types.replace(
        "export type NativeElements = Record<string, Record<string, unknown>>;",
        CLOSED_NATIVE_ELEMENTS,
    );
    types.push_str(REGISTRIES);
    std::fs::write(&runtime, types).unwrap();
    snapshot_project_diagnostics(&root)
}

fn has(diagnostics: &[(String, Option<u32>, String)], code: u32, needle: &str) -> bool {
    diagnostics.iter().any(|(file, found, message)| {
        file.as_str().ends_with("App.vue")
            && *found == Some(code)
            && message.as_str().contains(needle)
    })
}

#[test]
fn unknown_component_prop_and_directive_follow_vue_compiler_options() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let Some(enabled) = diagnostics_for(
        "unknown-checks-on",
        "\"checkUnknownComponents\": true, \"checkUnknownProps\": true, \"checkUnknownDirectives\": true",
    ) else {
        return;
    };
    assert!(has(&enabled, 2339, "MissingWidget"), "{enabled:?}");
    assert!(has(&enabled, 2353, "extra"), "{enabled:?}");
    assert!(has(&enabled, 2339, "vNotRegistered"), "{enabled:?}");

    let Some(absent) = diagnostics_for("unknown-checks-absent", "") else {
        return;
    };
    assert!(!has(&absent, 2339, "MissingWidget"), "{absent:?}");
    assert!(!has(&absent, 2353, "extra"), "{absent:?}");
    assert!(!has(&absent, 2339, "vNotRegistered"), "{absent:?}");

    let Some(disabled) = diagnostics_for(
        "unknown-checks-off",
        "\"strictTemplates\": true, \"checkUnknownComponents\": false, \"checkUnknownProps\": false, \"checkUnknownDirectives\": false",
    ) else {
        return;
    };
    assert!(!has(&disabled, 2339, "MissingWidget"), "{disabled:?}");
    assert!(!has(&disabled, 2353, "extra"), "{disabled:?}");
    assert!(!has(&disabled, 2339, "vNotRegistered"), "{disabled:?}");
}
