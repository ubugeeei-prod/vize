use std::{
    io::{BufRead, Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use vize_carton::cstr;

#[test]
fn check_json_reports_type_errors_via_project_typechecker() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "json-type-errors",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
        )],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let snapshot = serde_json::json!({
        "status": output.status.code(),
        "errorCount": json["errorCount"],
        "fileCount": json["fileCount"],
        "diagnostics": json["files"][0]["diagnostics"],
    });

    insta::with_settings!({
        snapshot_path => "snapshots"
    }, {
        insta::assert_snapshot!(
            "check_json_reports_type_errors_via_project_typechecker",
            serde_json::to_string_pretty(&snapshot).unwrap()
        );
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_json_reports_ts_importing_vue_errors() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "json-ts-vue-import",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
defineProps<{
  count: number
}>()
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
            ),
            (
                "src/main.ts",
                r#"import App from './App.vue'

type AppProps = InstanceType<typeof App>['$props']

const props: AppProps = {
  count: 'oops',
}

void props
"#,
            ),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let snapshot = serde_json::json!({
        "status": output.status.code(),
        "errorCount": json["errorCount"],
        "files": json["files"].as_array().unwrap().iter().map(|file| {
            serde_json::json!({
                "file": file["file"],
                "diagnostics": file["diagnostics"],
            })
        }).collect::<Vec<_>>(),
    });

    insta::with_settings!({
        snapshot_path => "snapshots"
    }, {
        insta::assert_snapshot!(
            "check_json_reports_ts_importing_vue_errors",
            serde_json::to_string_pretty(&snapshot).unwrap()
        );
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_without_patterns_uses_parent_relative_tsconfig_includes() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "default-tsconfig-inputs",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count = 1
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
        )],
    );
    std::fs::create_dir_all(project_root.join(".nuxt")).unwrap();
    std::fs::write(
        project_root.join(".nuxt/tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["../src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "extends": "./.nuxt/tsconfig.json"
}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["fileCount"], 1, "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert_eq!(
        json["files"][0]["file"], "src/App.vue",
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_json_reports_empty_result_when_no_files_match() {
    let project_root = create_cli_project("json-empty-inputs", &[]);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["check", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["fileCount"], 0);
    assert_eq!(json["errorCount"], 0);
    assert_eq!(json["warningCount"], 0);
    assert_eq!(json["files"].as_array().unwrap().len(), 0);

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_directory_pattern_resolves_json_modules_imported_by_ts() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "json-module-imports",
        &[
            (
                "tsconfig.json",
                r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "resolveJsonModule": true
  },
  "include": ["src/**/*"]
}"#,
            ),
            (
                "src/tokens.ts",
                r#"import colors from './tokens/source/colors.tokens.json'

const primary: string = colors.primary
void primary
"#,
            ),
            (
                "src/tokens/source/colors.tokens.json",
                r##"{
  "primary": "#0057ff"
}
"##,
            ),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    let diagnostics = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap().iter())
        .filter_map(|diagnostic| diagnostic.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["errorCount"], 0, "diagnostics: {diagnostics:#?}");
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.contains("Cannot find module")),
        "JSON modules should resolve with resolveJsonModule enabled: {diagnostics:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_vfor_over_object_types_key_as_keyof_not_number() {
    // Regression for vuejs/language-tools#5978 (#767): iterating an object with
    // `v-for="(value, key) in obj"` must type `value` as `T[keyof T]` and `key`
    // as `keyof T`. Vize used to emit `(obj).forEach((value: typeof obj[number],
    // key: number) => ...)`, which raised a spurious "forEach does not exist on
    // object" error and mis-typed `key` as `number`. vue-tsc reports zero
    // diagnostics here.
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "vfor-object-key",
        &[(
            "src/VForObject.vue",
            r#"<script setup lang="ts">
const obj: { foo: number; bar: number } = { foo: 1, bar: 2 }
function wantValue(n: number) { return n }
function wantKey(k: 'foo' | 'bar') { return k }
</script>

<template>
  <div v-for="(value, key) in obj">{{ wantValue(value) }} {{ wantKey(key) }}</div>
</template>
"#,
        )],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/VForObject.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        json["errorCount"], 0,
        "object v-for should type value as T[keyof T] and key as keyof T; stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_subset_resolves_relative_sibling_types() {
    // `vize check src/App.vue` only registers App.vue, but its relative import
    // `./types` must still resolve precisely (issue #766) — the sibling's types
    // should be pulled into the virtual project rather than degrading to `any`
    // or surfacing a false TS2307.
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "subset-sibling-types",
        &[
            (
                "src/types.ts",
                "export interface Sibling {\n  count: number\n}\n",
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import type { Sibling } from './types'
const value: Sibling = { count: 'not a number' }
</script>

<template>
  <div>{{ value.count }}</div>
</template>
"#,
            ),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/App.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    // The cross-file type resolved, so the real assignability error surfaces
    // (`'not a number'` is not assignable to `count: number`) — and crucially
    // NOT a `TS2307 Cannot find module './types'`.
    let diagnostics = json["files"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|file| file["diagnostics"].as_array().cloned().unwrap_or_default())
        .map(|d| d.as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();

    assert!(
        diagnostics
            .iter()
            .any(|message| message.contains("not assignable")),
        "expected a cross-file assignability error proving the sibling resolved; got {diagnostics:?}\nstderr:\n{stderr}"
    );
    assert!(
        !diagnostics
            .iter()
            .any(|message| message.contains("Cannot find module")),
        "import './types' should resolve, not degrade to TS2307; got {diagnostics:?}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_options_api_can_import_define_component_from_stubbed_vue() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "options-api-define-component",
        &[(
            "src/OptionsApi.vue",
            r#"<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "OptionsApi",
});
</script>

<template>
  <div>hello</div>
</template>
"#,
        )],
    );

    // Force the virtual project to use vize's fallback Vue stub instead of a
    // workspace-linked full Vue installation.
    remove_path_if_exists(&project_root.join("node_modules").join("@vue")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/OptionsApi.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_resolves_named_exports_from_vue_imported_via_path_alias() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "vue-alias-named-export",
        &[
            (
                "src/AliasProvider.vue",
                r#"<script setup lang="ts">
export interface AliasConfig {
  key: string;
  label: string;
}

defineProps<{ configs: AliasConfig[] }>();
</script>

<template>
  <div />
</template>
"#,
            ),
            (
                "src/AliasConsumer.vue",
                r#"<script setup lang="ts">
import AliasProvider, { type AliasConfig } from "@/AliasProvider.vue";

const configs: AliasConfig[] = [{ key: "a", label: "A" }];
</script>

<template>
  <AliasProvider :configs="configs" />
</template>
"#,
            ),
        ],
    );

    // The default helper tsconfig has no path aliases; rewrite it with the
    // `@/*` -> `src/*` mapping that drives the repro.
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "baseUrl": ".",
    "paths": { "@/*": ["src/*"] }
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/AliasProvider.vue",
            "src/AliasConsumer.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_scoped_slot_props_typed_from_child_define_slots() {
    // #764: a scoped slot on a child component should type its props from the
    // child's `defineSlots`, so misusing a slot prop raises a real diagnostic
    // (here `item` is `number`, so `.toUpperCase()` is TS2339) instead of `any`.
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "scoped-slot-prop-types",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineSlots<{ default(props: { item: number }): any }>()
</script>

<template>
  <slot :item="1" />
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child v-slot="{ item }">{{ item.toUpperCase() }}</Child>
</template>
"#,
            ),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/Parent.vue",
            "src/Child.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    let diagnostics = json["files"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|file| file["diagnostics"].as_array().cloned().unwrap_or_default())
        .map(|d| d.as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();

    assert!(
        diagnostics
            .iter()
            .any(|message| message.contains("toUpperCase") && message.contains("number")),
        "expected the slot prop `item` to be typed `number` from the child's defineSlots; got {diagnostics:?}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_explicit_file_loads_ambient_declare_global_types() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "ambient-declare-global",
        &[
            (
                "src/@types/globals.d.ts",
                r#"export {};

declare global {
  type GlobalTabType = "default" | "wireframes" | "liked";
}
"#,
            ),
            (
                "src/UseGlobalType.vue",
                r#"<script setup lang="ts">
const tab: GlobalTabType = "default";
</script>

<template>
  <div>{{ tab }}</div>
</template>
"#,
            ),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/UseGlobalType.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_generic_script_setup_resolves_type_referencing_generic_param() {
    // `<script setup generic="T">` declares a type that references `T`. The
    // type is lifted to module scope, so the generic parameter must be
    // re-declared on it; otherwise `T` is unbound there (TS2304).
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "generic-script-setup-hoisted-type",
        &[(
            "src/Generic.vue",
            r#"<script setup lang="ts" generic="T extends string">
type Option = { key: T; label: string }

defineProps<{
  options: Option[]
  current: T | undefined
}>()
</script>

<template>
  <ul>
    <li v-for="o in options" :key="o.key">{{ o.label }}</li>
  </ul>
</template>
"#,
        )],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/Generic.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_v_else_branch_narrows_discriminated_union() {
    // Regression: a flat `v-if` / `v-else` pair (sibling elements, not grouped
    // into an `IfNode`) must narrow a discriminated union in the `v-else`
    // branch. Vize previously gave the else branch no guard, so accessing the
    // other variant's property raised a spurious TS2339. vue-tsc reports zero
    // diagnostics for this template.
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "v-else-narrowing",
        &[(
            "src/VElse.vue",
            r#"<script setup lang="ts">
type U = { kind: 'a'; x: number } | { kind: 'b'; y: string }
const props = defineProps<{ data: U }>()
</script>

<template>
  <div v-if="props.data.kind === 'a'">{{ props.data.x }}</div>
  <div v-else>{{ props.data.y }}</div>
</template>
"#,
        )],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/VElse.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_lifts_script_setup_type_reexports() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "script-setup-type-reexport",
        &[
            (
                "src/ReExportType.ts",
                "export type FilterType = 'image' | 'text'\n",
            ),
            (
                "src/ReExportType.vue",
                r#"<script setup lang="ts">
import { type FilterType } from './ReExportType'

export type { FilterType }

defineProps<{ kind?: FilterType }>()
</script>

<template>
  <div />
</template>
"#,
            ),
            (
                "src/ReExportTypeConsumer.vue",
                r#"<script setup lang="ts">
import ReExportType, { type FilterType } from './ReExportType.vue'

const v: FilterType = 'image'
</script>

<template>
  <ReExportType :kind="v" />
</template>
"#,
            ),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/ReExportType.vue",
            "src/ReExportTypeConsumer.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_respects_explicit_corsa_path() {
    let project_root = create_cli_project(
        "explicit-corsa-path",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count = 1;
</script>
"#,
        )],
    );
    let missing_corsa = project_root.join("__missing_corsa__");
    let missing_corsa_arg = missing_corsa.to_string_lossy().into_owned();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "check",
            "src/App.vue",
            "--quiet",
            "--corsa-path",
            missing_corsa_arg.as_str(),
        ])
        .output()
        .unwrap();

    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(
        stderr.contains("Configured Corsa executable does not exist"),
        "{stderr}"
    );
    assert!(stderr.contains("__missing_corsa__"), "{stderr}");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_respects_configured_tsgo_path() {
    let project_root = create_cli_project(
        "configured-tsgo-path",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count = 1;
</script>
"#,
        )],
    );
    let missing_corsa = project_root.join("__missing_configured_tsgo__");
    std::fs::write(
        project_root.join("vize.config.json"),
        cstr!(
            r#"{{
  "typeChecker": {{
    "tsgoPath": "{}"
  }}
}}"#,
            missing_corsa.display()
        )
        .as_str(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["check", "src/App.vue", "--quiet"])
        .output()
        .unwrap();

    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(
        stderr.contains("Configured Corsa executable does not exist"),
        "{stderr}"
    );
    assert!(stderr.contains("__missing_configured_tsgo__"), "{stderr}");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_respects_configured_corsa_path() {
    let project_root = create_cli_project(
        "configured-corsa-path",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count = 1;
</script>
"#,
        )],
    );
    let config_dir = project_root.join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("vize.config.json"),
        r#"{
  "typeChecker": {
    "corsaPath": "__missing_configured_corsa__"
  }
}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "check",
            "--config",
            "config/vize.config.json",
            "src/App.vue",
            "--quiet",
        ])
        .output()
        .unwrap();

    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(
        stderr.contains("Configured Corsa executable does not exist"),
        "{stderr}"
    );
    assert!(stderr.contains("__missing_configured_corsa__"), "{stderr}");
    assert!(
        stderr.contains("config/__missing_configured_corsa__"),
        "{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn lint_type_aware_rules_respect_configured_corsa_path() {
    let project_root = create_cli_project(
        "lint-configured-corsa-path",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}

loadData()
</script>
"#,
        )],
    );
    let missing_corsa = project_root.join("__missing_lint_corsa__");
    std::fs::write(
        project_root.join("vize.config.json"),
        cstr!(
            r#"{{
  "typeChecker": {{
    "corsaPath": "{}"
  }}
}}"#,
            missing_corsa.display()
        )
        .as_str(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--preset",
            "opinionated",
            "--help-level",
            "none",
            "src/App.vue",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());
    assert!(
        stdout.contains("Configured Corsa executable does not exist"),
        "{stdout}"
    );
    assert!(stdout.contains("__missing_lint_corsa__"), "{stdout}");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn lint_json_output_is_valid_when_error_exit_runs() {
    let project_root = create_cli_project(
        "lint-json-error-exit",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const items = [1, 2]
</script>

<template>
  <div v-for="item in items">{{ item }}</div>
</template>
"#,
        )],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--format",
            "json",
            "--help-level",
            "none",
            "src/App.vue",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    let files = json
        .as_array()
        .expect("lint JSON output should be an array");
    assert_eq!(files.len(), 1, "{stdout}");
    assert_eq!(files[0]["errorCount"], 1, "{stdout}");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_rejects_unsupported_corsa_server_count() {
    let project_root = create_cli_project(
        "unsupported-corsa-servers",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count = 1;
</script>
"#,
        )],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["check", "src/App.vue", "--quiet", "--servers", "2"])
        .output()
        .unwrap();

    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr.contains("typeChecker.servers=2 is not supported"),
        "{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn lsp_corsa_smoke_publishes_diagnostics_and_hover() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        eprintln!("skipping LSP Corsa smoke: @typescript/native-preview runtime is unavailable");
        return;
    };
    let project_root = create_cli_project(
        "lsp-corsa-smoke",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count: number = 'oops'
</script>

<template>
  <div>{{ count.toFixed(1) }}</div>
</template>
"#,
        )],
    );
    std::fs::write(
        project_root.join("vize.config.json"),
        cstr!(
            r#"{{
  "typeChecker": {{
    "corsaPath": "{}"
  }},
  "lsp": {{
    "lint": true,
    "typecheck": true,
    "hover": true
  }}
}}"#,
            corsa_path
        )
        .as_str(),
    )
    .unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    std::thread::spawn(move || {
        let mut stderr = std::io::BufReader::new(stderr);
        let mut buffer = Vec::new();
        let _ = stderr.read_to_end(&mut buffer);
    });
    let (messages_tx, messages_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout);
        while let Ok(message) = read_lsp_message(&mut reader) {
            if messages_tx.send(message).is_err() {
                break;
            }
        }
    });

    let root_uri = file_uri(&project_root);
    let app_path = project_root.join("src/App.vue");
    let app_uri = file_uri(&app_path);
    let source = std::fs::read_to_string(&app_path).unwrap();

    write_lsp_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": null,
                "rootUri": root_uri,
                "capabilities": {},
                "initializationOptions": {
                    "lint": true,
                    "typecheck": true,
                    "hover": true
                }
            }
        }),
    );
    let initialize = recv_lsp_response(&messages_rx, 1);
    assert!(
        initialize["result"]["capabilities"]["hoverProvider"]
            .as_bool()
            .unwrap_or(false),
        "{initialize}"
    );

    write_lsp_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        }),
    );
    write_lsp_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": app_uri,
                    "languageId": "vue",
                    "version": 1,
                    "text": source
                }
            }
        }),
    );
    let diagnostics = recv_lsp_notification(&messages_rx, "textDocument/publishDiagnostics");
    assert!(
        diagnostics["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("number") || message.contains("TS2322"))),
        "{diagnostics}"
    );

    write_lsp_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": app_uri },
                "position": { "line": 5, "character": 11 }
            }
        }),
    );
    let hover = recv_lsp_response(&messages_rx, 2);
    assert!(!hover["result"].is_null(), "{hover}");

    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn lint_cross_file_reports_ssr_browser_api_risk() {
    let project_root = create_cli_project(
        "cross-file-ssr-browser-api",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import BrowserBadge from './BrowserBadge.vue'
</script>

<template>
  <BrowserBadge />
</template>
"#,
            ),
            (
                "src/BrowserBadge.vue",
                r#"<template>
  <span>{{ window.innerWidth }}</span>
</template>
"#,
            ),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--cross-file", "--help-level", "none", "src"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("vize:croquis/cf/browser-api-ssr"),
        "{stdout}"
    );
    assert!(stdout.contains("window"), "{stdout}");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_json_reports_broken_sfc_parse_errors_without_secondary_noise() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "json-broken-sfc",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count =
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
        )],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let diagnostics = json["files"][0]["diagnostics"].as_array().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["errorCount"], 1);
    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]
            .as_str()
            .unwrap()
            .contains("Script parse error")
    );
    assert!(
        !diagnostics[0]
            .as_str()
            .unwrap()
            .contains("Cannot find name")
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[cfg(unix)]
#[test]
fn check_socket_json_preserves_json_output_contract() {
    use std::os::unix::net::UnixListener;

    let project_root = create_cli_project(
        "socket-json",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
        )],
    );
    let socket_path = std::path::PathBuf::from(
        cstr!("/tmp/vize-check-{}-socket-json.sock", std::process::id()).as_str(),
    );
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut line = std::string::String::new();
        std::io::BufReader::new(stream.try_clone().unwrap())
            .read_line(&mut line)
            .unwrap();
        let request: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(request["method"], "check");

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request["id"],
            "result": {
                "diagnostics": [
                    {
                        "message": "Type 'number' is not assignable to type 'string'.",
                        "severity": "error",
                        "line": 2,
                        "column": 7,
                        "code": "2322"
                    },
                    {
                        "message": "Unused binding.",
                        "severity": "warning",
                        "line": 2,
                        "column": 7,
                        "code": null
                    }
                ],
                "virtualTs": "const count: string = 0;",
                "errorCount": 1
            }
        });
        writeln!(stream, "{response}").unwrap();
    });

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "check",
            "src",
            "--socket",
            socket_path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    server.join().unwrap();
    let _ = std::fs::remove_file(&socket_path);
    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["errorCount"], 1);
    assert_eq!(json["warningCount"], 1);
    assert_eq!(json["fileCount"], 1);
    assert_eq!(json["files"][0]["file"], "src/App.vue");
    assert_eq!(json["files"][0]["virtualTs"], "const count: string = 0;");
    assert_eq!(
        json["files"][0]["diagnostics"],
        serde_json::json!([
            "error:2:7 [TS2322] Type 'number' is not assignable to type 'string'.",
            "warning:2:7 Unused binding."
        ])
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_can_emit_declarations() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "emit-declarations",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
export interface PublicProps {
  count: number
}

const props = defineProps<PublicProps>()
</script>

<template>
  <div>{{ props.count }}</div>
</template>
"#,
            ),
            (
                "src/index.ts",
                r#"export { default as App } from './App.vue'
"#,
            ),
        ],
    );
    let declaration_dir = project_root.join("types");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            ".",
            "--format",
            "json",
            "--declaration",
            "--declaration-dir",
            "types",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let declarations = collect_declaration_snapshot(&declaration_dir);
    let snapshot = serde_json::json!({
        "status": output.status.code(),
        "errorCount": json["errorCount"],
        "declarations": json["declarations"],
        "files": declarations,
    });

    insta::with_settings!({
        snapshot_path => "snapshots"
    }, {
        insta::assert_snapshot!(
            "check_can_emit_declarations",
            serde_json::to_string_pretty(&snapshot).unwrap()
        );
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_json_reports_nuxt_auto_imports_and_preserves_builtin_components() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "nuxt-auto-imports",
        &[
            ("nuxt.config.ts", "export default {}\n"),
            (
                "app.vue",
                r#"<script setup lang="ts">
const count = 'oops'
</script>

<template>
  <AutoCard :count="count" />
  <NuxtLink to="/">Home</NuxtLink>
  <ClientOnly>
    {{ useCounter().count.toUpperCase() }}
  </ClientOnly>
</template>
"#,
            ),
            (
                "components/AutoCard.vue",
                r#"<script setup lang="ts">
defineProps<{
  count: number
}>()
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
            ),
            (
                "app/composables/counter.ts",
                r#"export function useCounter(): { count: number } {
  return { count: 1 }
}
"#,
            ),
        ],
    );
    std::fs::create_dir_all(project_root.join(".nuxt")).unwrap();
    std::fs::write(
        project_root.join(".nuxt/components.d.ts"),
        r#"declare module 'vue' {
  export interface GlobalComponents {
    AutoCard: typeof import('../components/AutoCard.vue')['default']
  }
}
export {}
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/imports.d.ts"),
        r#"export { useCounter } from '../app/composables/counter';
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let diagnostics = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap().iter())
        .filter_map(|diagnostic| diagnostic.as_str())
        .collect::<Vec<_>>();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("[TS2322]") && diagnostic.contains("number")),
        "expected AutoCard prop type error, got: {diagnostics:#?}"
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("[TS2339]") && diagnostic.contains("toUpperCase")),
        "expected typed Nuxt auto-import composable error, got: {diagnostics:#?}"
    );
    assert!(
        diagnostics.iter().all(|diagnostic| {
            !diagnostic.contains("NuxtLink") && !diagnostic.contains("ClientOnly")
        }),
        "Nuxt built-in components should not produce diagnostics: {diagnostics:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_nuxt_import_meta_augmentations_do_not_conflict() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "nuxt-import-meta",
        &[
            ("nuxt.config.ts", "export default {}\n"),
            (
                "app.vue",
                r#"<script setup lang="ts">
const side = import.meta.client ? 'client' : 'server'
</script>

<template>
  <div>{{ side }}</div>
</template>
"#,
            ),
        ],
    );
    std::fs::create_dir_all(project_root.join(".nuxt/types")).unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/import-meta.d.ts"),
        r#"export {};

declare global {
  interface ImportMeta {
    client: boolean;
    server: boolean;
    dev: boolean;
    prod: boolean;
    ssr: boolean;
  }
}
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["app.vue", ".nuxt/types/**/*.d.ts"]
}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "app.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    let diagnostics = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap().iter())
        .filter_map(|diagnostic| diagnostic.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.contains("[TS2687]")),
        "ImportMeta augmentations should not conflict: {diagnostics:#?}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_explicit_nuxt_file_loads_hidden_declarations_and_project_shims() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "nuxt-hidden-declarations-and-shims",
        &[
            ("nuxt.config.ts", "export default {}\n"),
            (
                "app.vue",
                r#"<script setup lang="ts">
import IconX from "~icons/icons/ic_x"
import "~/assets/styles/main.css"

const ticketSalesEnabled: boolean = import.meta.vfFeatures.ticketSales
</script>

<template>
  <IconX />
  <div>{{ ticketSalesEnabled }}</div>
</template>
"#,
            ),
            ("assets/styles/main.css", "body { color: black; }\n"),
        ],
    );
    std::fs::create_dir_all(project_root.join(".nuxt/types")).unwrap();
    std::fs::write(
        project_root.join(".nuxt/tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "paths": {
      "~/*": ["../*"]
    }
  },
  "include": ["./nuxt.d.ts", "../app.vue", "../shim.d.ts"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/nuxt.d.ts"),
        r#"/// <reference path="types/feature-flags.d.ts" />
export {};
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/feature-flags.d.ts"),
        r#"export {};

declare global {
  interface ImportMetaFeatureFlags {
    readonly ticketSales: boolean;
  }

  interface ImportMeta {
    readonly vfFeatures: ImportMetaFeatureFlags;
  }
}
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("shim.d.ts"),
        r#"declare module "*.css";
declare module "~icons/icons/ic_x";
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "extends": "./.nuxt/tsconfig.json"
}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "app.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_declaration_emit_uses_tsconfig_options() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "emit-declarations-tsconfig",
        &[
            (
                "tsconfig.json",
                r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "declarationDir": "types-from-tsconfig",
    "declarationMap": true
  },
  "include": ["src/**/*"]
}"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
export interface PublicProps {
  label: string
}

defineProps<PublicProps>()
</script>

<template>
  <button>{{ label }}</button>
</template>
"#,
            ),
            (
                "src/index.ts",
                r#"export { default as App } from './App.vue'
"#,
            ),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json", "--declaration"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let declarations = json["declarations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        declarations,
        vec![
            "types-from-tsconfig/App.vue.d.ts",
            "types-from-tsconfig/index.d.ts"
        ]
    );
    assert!(
        project_root
            .join("types-from-tsconfig/App.vue.d.ts")
            .is_file()
    );
    assert!(
        project_root
            .join("types-from-tsconfig/App.vue.d.ts.map")
            .is_file()
    );
    assert!(
        project_root
            .join("types-from-tsconfig/index.d.ts")
            .is_file()
    );
    assert!(
        project_root
            .join("types-from-tsconfig/index.d.ts.map")
            .is_file()
    );
    assert!(!project_root.join("dist/types").exists());

    let app_declaration =
        std::fs::read_to_string(project_root.join("types-from-tsconfig/App.vue.d.ts")).unwrap();
    assert!(app_declaration.contains("export interface PublicProps"));

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_json_handles_monorepo_tsconfig_extends_paths_and_excludes() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "monorepo-tsconfig",
        &[
            (
                "tsconfig.base.json",
                r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "baseUrl": ".",
    "paths": {
      "@shared/*": ["packages/shared/src/*"]
    },
    "noEmit": true
  }
}"#,
            ),
            (
                "tsconfig.json",
                r#"{
  "extends": "./tsconfig.base.json",
  "include": [
    "packages/*/src/**/*.ts",
    "packages/*/src/**/*.vue"
  ],
  "exclude": [
    "packages/*/src/generated/**"
  ]
}"#,
            ),
            (
                "packages/app/tsconfig.json",
                r#"{
  "extends": "../../tsconfig.base.json",
  "include": ["src/**/*.ts", "src/**/*.vue"]
}"#,
            ),
            (
                "packages/ui/tsconfig.json",
                r#"{
  "extends": "../../tsconfig.base.json",
  "include": ["src/**/*.ts", "src/**/*.vue"]
}"#,
            ),
            (
                "packages/shared/tsconfig.json",
                r#"{
  "extends": "../../tsconfig.base.json",
  "include": ["src/**/*.ts"]
}"#,
            ),
            (
                "packages/shared/src/contracts.ts",
                r#"export type Label = string
"#,
            ),
            (
                "packages/ui/src/UiButton.vue",
                r#"<script setup lang="ts">
import type { Label } from '@shared/contracts'

defineProps<{
  label: Label
  count: number
}>()
</script>

<template>
  <button>{{ label }} {{ count }}</button>
</template>
"#,
            ),
            (
                "packages/app/src/App.vue",
                r#"<script setup lang="ts">
import UiButton from '../../ui/src/UiButton.vue'
import type { Label } from '@shared/contracts'

const label: Label = 'Save'
const count = 'not a number'
</script>

<template>
  <UiButton :label="label" :count="count" />
</template>
"#,
            ),
            (
                "packages/app/src/generated/Bad.vue",
                r#"<script setup lang="ts">
const shouldNotBeChecked: string = 0
</script>
"#,
            ),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    let diagnostics = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap().iter())
        .filter_map(|diagnostic| diagnostic.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["fileCount"], 3);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("[TS2322]") && diagnostic.contains("number")),
        "expected imported monorepo component prop type error, got: {diagnostics:#?}"
    );
    assert!(
        diagnostics.iter().all(|diagnostic| {
            !diagnostic.contains("Cannot find module")
                && !diagnostic.contains("shouldNotBeChecked")
                && !diagnostic.contains("generated/Bad.vue")
        }),
        "monorepo tsconfig paths/excludes should be respected: {diagnostics:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn collect_declaration_snapshot(
    declaration_dir: &Path,
) -> Vec<(std::string::String, std::string::String)> {
    let mut files = Vec::new();
    collect_declaration_snapshot_recursive(declaration_dir, declaration_dir, &mut files);

    files.sort();
    files
}

fn collect_declaration_snapshot_recursive(
    root: &Path,
    current: &Path,
    files: &mut Vec<(std::string::String, std::string::String)>,
) {
    let Ok(entries) = std::fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_declaration_snapshot_recursive(root, &path, files);
            continue;
        }
        if !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".d.ts"))
        {
            continue;
        }
        files.push((
            relative_path(root, &path),
            std::fs::read_to_string(path).unwrap(),
        ));
    }
}

fn relative_path(root: &Path, file: &Path) -> std::string::String {
    file.strip_prefix(root)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| file.display().to_string())
}

fn workspace_root() -> &'static std::path::Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}

fn create_cli_project(name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    link_workspace_node_modules(&project_root).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();

    for (path, source) in files {
        let file_path = project_root.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(file_path, source).unwrap();
    }

    project_root
}

fn write_lsp_message(stdin: &mut std::process::ChildStdin, message: &serde_json::Value) {
    let body = message.to_string();
    write!(stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
    stdin.flush().unwrap();
}

fn read_lsp_message(
    reader: &mut std::io::BufReader<std::process::ChildStdout>,
) -> std::io::Result<serde_json::Value> {
    let mut content_length = None;
    loop {
        let mut line = std::string::String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "lsp stdout closed",
            ));
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length =
                Some(value.trim().parse::<usize>().map_err(|error| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
                })?);
        }
    }

    let Some(content_length) = content_length else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing Content-Length header",
        ));
    };
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

fn recv_lsp_response(
    receiver: &std::sync::mpsc::Receiver<serde_json::Value>,
    id: i64,
) -> serde_json::Value {
    recv_lsp_matching(receiver, |message| message["id"].as_i64() == Some(id))
}

fn recv_lsp_notification(
    receiver: &std::sync::mpsc::Receiver<serde_json::Value>,
    method: &str,
) -> serde_json::Value {
    recv_lsp_matching(receiver, |message| {
        message["method"].as_str() == Some(method)
    })
}

fn recv_lsp_matching(
    receiver: &std::sync::mpsc::Receiver<serde_json::Value>,
    mut matches: impl FnMut(&serde_json::Value) -> bool,
) -> serde_json::Value {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let mut seen = Vec::new();
    loop {
        let now = std::time::Instant::now();
        assert!(
            now < deadline,
            "timed out waiting for LSP message; seen: {seen:#?}"
        );
        let remaining = deadline.saturating_duration_since(now);
        let message = receiver.recv_timeout(remaining).unwrap();
        if matches(&message) {
            return message;
        }
        seen.push(message);
    }
}

fn file_uri(path: &Path) -> std::string::String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let path = path.to_string_lossy().replace('\\', "/");
    let prefix = if path.starts_with('/') {
        "file://"
    } else {
        "file:///"
    };
    format!("{prefix}{}", percent_encode_file_uri_path(&path))
}

fn percent_encode_file_uri_path(path: &str) -> std::string::String {
    let mut encoded = std::string::String::with_capacity(path.len());
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                encoded.push(byte as char)
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                encoded.push('%');
                encoded.push(HEX[(byte >> 4) as usize] as char);
                encoded.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    encoded
}

fn resolve_test_corsa_path() -> Option<String> {
    let workspace_root = workspace_root();
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache.display().to_string());
    }

    if let Some(native_tsgo) = resolve_workspace_native_tsgo(workspace_root) {
        return Some(native_tsgo.display().to_string());
    }

    for candidate in [
        workspace_root.join("node_modules/.bin/tsgo"),
        workspace_root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ] {
        if candidate.exists() {
            return Some(candidate.display().to_string());
        }
    }

    None
}

fn resolve_workspace_native_tsgo(workspace_root: &Path) -> Option<std::path::PathBuf> {
    let platform_suffix = native_preview_platform_suffix();
    let package_name = cstr!("@typescript/native-preview-{platform_suffix}");

    let pnpm_root = workspace_root.join("node_modules/.pnpm");
    if let Ok(entries) = std::fs::read_dir(&pnpm_root) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with(cstr!("@typescript+native-preview-{platform_suffix}@").as_str()) {
                continue;
            }

            if let Some(path) = first_existing_tsgo_binary(
                entry
                    .path()
                    .join("node_modules")
                    .join("@typescript")
                    .join(package_name.as_str())
                    .join("lib"),
            ) {
                return Some(path);
            }
        }
    }

    first_existing_tsgo_binary(
        workspace_root
            .join("node_modules")
            .join("@typescript")
            .join(package_name.as_str())
            .join("lib"),
    )
}

fn first_existing_tsgo_binary(lib_dir: std::path::PathBuf) -> Option<std::path::PathBuf> {
    for executable in test_corsa_executable_names() {
        let candidate = lib_dir.join(executable);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn test_corsa_executable_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["tsgo.exe", "tsgo", "corsa.exe", "corsa"]
    } else {
        &["tsgo", "corsa"]
    }
}

fn native_preview_platform_suffix() -> &'static str {
    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "darwin-arm64"
        } else {
            "darwin-x64"
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "linux-arm64"
        } else {
            "linux-x64"
        }
    } else if cfg!(target_os = "windows") {
        "win32-x64"
    } else {
        ""
    }
}

fn link_workspace_node_modules(project_root: &Path) -> std::io::Result<()> {
    let workspace_node_modules = resolve_workspace_node_modules();

    let target = project_root.join("node_modules");
    remove_path_if_exists(&target)?;
    std::fs::create_dir_all(&target)?;

    if let Some(ref workspace_node_modules) = workspace_node_modules {
        link_or_stub_package(workspace_node_modules, &target, "vue", write_test_vue_stub)?;
        link_or_stub_package(
            workspace_node_modules,
            &target,
            "vite",
            write_test_vite_stub,
        )?;

        let vue_namespace = workspace_node_modules.join("@vue");
        if vue_namespace.exists() {
            symlink_path(&vue_namespace, &target.join("@vue"))?;
        }
    } else {
        write_test_vue_stub(&target)?;
        write_test_vite_stub(&target)?;
    }

    if let Some(corsa_path) = resolve_test_corsa_path() {
        let source = std::path::PathBuf::from(corsa_path);
        if source.exists() {
            let file_name = source.file_name().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid corsa binary path",
                )
            })?;
            if !is_node_modules_bin_wrapper(&source) {
                symlink_path(
                    &source,
                    &target
                        .join("@typescript")
                        .join("native-preview")
                        .join("lib")
                        .join(file_name),
                )?;
            }
            symlink_path(&source, &target.join(".bin").join(file_name))?;
        }
    }

    Ok(())
}

fn remove_path_if_exists(path: &Path) -> std::io::Result<()> {
    if path.is_symlink() || path.is_file() {
        std::fs::remove_file(path)?;
    } else if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn is_node_modules_bin_wrapper(path: &Path) -> bool {
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        == Some(".bin")
}

fn link_or_stub_package(
    workspace_node_modules: &Path,
    target: &Path,
    package: &str,
    stub_writer: fn(&Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let source = workspace_node_modules.join(package);
    if source.exists() {
        symlink_path(&source, &target.join(package))
    } else {
        stub_writer(target)
    }
}

fn resolve_workspace_node_modules() -> Option<std::path::PathBuf> {
    let override_path = std::env::var_os("VIZE_TEST_WORKSPACE_NODE_MODULES");
    if let Some(override_path) = override_path {
        let override_path = std::path::PathBuf::from(override_path);
        if override_path.as_os_str() == "__none__" {
            return None;
        }
        return override_path.exists().then_some(override_path);
    }

    let workspace_node_modules = workspace_root().join("node_modules");
    workspace_node_modules
        .exists()
        .then_some(workspace_node_modules)
}

fn write_test_vue_stub(target: &Path) -> std::io::Result<()> {
    let vue_dir = target.join("vue");
    std::fs::create_dir_all(&vue_dir)?;
    std::fs::write(
        vue_dir.join("package.json"),
        r#"{
  "name": "vue",
  "types": "index.d.ts"
}"#,
    )?;
    std::fs::write(
        vue_dir.join("index.d.ts"),
        r#"export interface Ref<T = any, S = T> {
  value: T;
}

export interface ShallowRef<T = any, S = T> extends Ref<T, S> {}

export interface ComponentPublicInstance {
  $attrs: any;
  $slots: any;
  $refs: any;
  $emit: (...args: any[]) => void;
}

export type DefineComponent<
  Props = any,
  _RawBindings = any,
  _Data = any,
  _Computed = any,
  _Methods = any,
  _Mixin = any,
  _Extends = any,
  Emits = any,
> = new (...args: any[]) => ComponentPublicInstance & {
  $props: Props;
  $emit: Emits extends (...args: any[]) => any ? Emits : (...args: any[]) => void;
};

export declare function ref<T>(value: T): Ref<T>;
export declare function useTemplateRef<T = any>(key: string): ShallowRef<T | null>;
export declare function defineComponent<Props = any>(options: any): DefineComponent<Props>;
"#,
    )?;
    Ok(())
}

fn write_test_vite_stub(target: &Path) -> std::io::Result<()> {
    let vite_dir = target.join("vite");
    std::fs::create_dir_all(&vite_dir)?;
    std::fs::write(
        vite_dir.join("package.json"),
        r#"{
  "name": "vite",
  "types": "client.d.ts"
}"#,
    )?;
    std::fs::write(vite_dir.join("client.d.ts"), "")?;
    Ok(())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    remove_path_if_exists(target)?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        let metadata = std::fs::metadata(source)?;
        if metadata.is_dir() {
            std::os::windows::fs::symlink_dir(source, target)
        } else {
            std::os::windows::fs::symlink_file(source, target)
        }
    }
}
