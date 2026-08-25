#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_s0::cstr;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("check-canon-generic-props-{name}-{}", std::process::id()).as_str())
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn link_workspace_vue(project_root: &Path) -> std::io::Result<()> {
    let Some(vue_package) = workspace_vue_package() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package missing",
        ));
    };
    let workspace_node_modules = vue_package.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package has no node_modules parent",
        )
    })?;
    let target = project_root.join("node_modules");
    std::fs::create_dir_all(&target)?;
    symlink_path(&vue_package, &target.join("vue"))?;
    let vue_namespace = workspace_node_modules.join("@vue");
    if vue_namespace.exists() {
        symlink_path(&vue_namespace, &target.join("@vue"))?;
    }
    Ok(())
}

fn workspace_vue_package() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.join("node_modules/vue"),
        root.join("tests/node_modules/vue"),
        root.join("playground/node_modules/vue"),
        root.join("examples/vite-musea/node_modules/vue"),
        root.join("examples/jsx-tsx/node_modules/vue"),
        root.join("npm/framework/nuxt/node_modules/vue"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(target)?;
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}

fn skip_case(name: &str, reason: impl std::fmt::Display) {
    eprintln!("skipping {name}: {reason}");
}

fn create_case_with_files(name: &str, files: &[(&str, &str)]) -> Option<PathBuf> {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    if let Err(error) = link_workspace_vue(&project_root) {
        skip_case(name, error);
        let _ = std::fs::remove_dir_all(&project_root);
        return None;
    }
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","jsx":"preserve","jsxImportSource":"vue","noEmit":true},"include":["src/**/*"]}"#,
    )
    .unwrap();
    for (path, source) in files {
        let target = project_root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(target, source).unwrap();
    }
    Some(project_root)
}

fn run_check_json(project_root: &Path, corsa_path: &Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success(),
        "check failed\nstdout:\n{}\nstderr:\n{}",
        stdout,
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
}

fn run_generic_props_case(name: &str, files: &[(&str, &str)]) {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        skip_case(name, "tsgo not found");
        return;
    };

    let Some(project_root) = create_case_with_files(name, files) else {
        return;
    };

    run_check_json(&project_root, &corsa_path);

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn generic_model_array_normalization_accepts_empty_arrays() {
    run_generic_props_case(
        "model-array-normalization",
        &[(
            "src/Foo.vue",
            r#"<script setup lang="ts" generic="T extends string | number = string">
import { type Ref, ref } from "vue";

interface Props<T> {
  defaultValue?: T | T[] | null;
  multiple?: boolean;
}

const props = withDefaults(defineProps<Props<T>>(), { defaultValue: undefined });
const modelValue = ref(props.defaultValue) as Ref<typeof props.defaultValue>;

if (props.multiple && !Array.isArray(modelValue.value)) {
  modelValue.value = modelValue.value ? [modelValue.value] : [];
}
</script>
"#,
        )],
    );
}

#[test]
fn generic_model_sentinel_comparison_keeps_literal_member() {
    run_generic_props_case(
        "model-sentinel-comparison",
        &[(
            "src/Foo.vue",
            r#"<script setup lang="ts" generic="T = boolean">
import { type Ref, computed, ref } from "vue";

type ValueState<T> = T | "special";
interface Props<T> { defaultValue?: ValueState<T>; }

const props = withDefaults(defineProps<Props<T>>(), { defaultValue: undefined });
const modelValue = ref(props.defaultValue) as Ref<typeof props.defaultValue>;
const isSpecial = computed(() => modelValue.value === "special");
</script>

<template>{{ isSpecial }}</template>
"#,
        )],
    );
}

#[test]
fn imported_generic_props_do_not_pollute_computed_values_with_boolean() {
    run_generic_props_case(
        "imported-props-computed-boolean-pollution",
        &[
            (
                "src/Base.vue",
                r#"<script lang="ts">
export interface BaseProps<T> {
  value: T;
  checked?: boolean;
}
</script>
<script setup lang="ts" generic="T">defineProps<BaseProps<T>>();</script>
<template />
"#,
            ),
            (
                "src/Foo.vue",
                r#"<script setup lang="ts" generic="T">
import { computed } from "vue";
import type { BaseProps } from "./Base.vue";

const props = withDefaults(defineProps<BaseProps<T>>(), { checked: undefined });
const entries = computed(() => {
  const value = props.value;
  if (Array.isArray(value)) return value;
  return [];
});
</script>

<template>{{ entries.length }}</template>
"#,
            ),
        ],
    );
}

#[test]
fn generic_function_prop_is_callable_after_typeof_guard() {
    run_generic_props_case(
        "function-prop-typeof-guard",
        &[(
            "src/Foo.vue",
            r#"<script lang="ts">
export type Item = { value: string };

export interface FooProps<TItems extends Item[] = Item[]> {
  items?: TItems;
  getChildren?: (item: TItems[number]) => TItems[number][] | undefined;
}
</script>

<script setup lang="ts" generic="TItems extends Item[] = Item[]">
const props = withDefaults(defineProps<FooProps<TItems>>(), {
  items: () => [] as unknown as TItems
});
const item = props.items[0]!;

if (typeof props.getChildren === "function") {
  props.getChildren(item);
}
</script>
"#,
        )],
    );
}
