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
        .join(cstr!("check-canon-recent-{name}-{}", std::process::id()).as_str())
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

fn write_tsconfig(project_root: &Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
}

fn create_case(name: &str, vue_file: &str, source: &str) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    std::fs::write(project_root.join("src").join(vue_file), source).unwrap();
    project_root
}

fn run_check_json(project_root: &Path, corsa_path: &Path, target: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            target,
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
    stdout.to_string()
}

#[test]
fn check_with_defaults_preserves_loose_required_props() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case(
        "with-defaults-loose-required",
        "Foo.vue",
        r#"<script lang="ts">
export interface FooProps {
  modelValue?: string
  as?: string
}
</script>

<script setup lang="ts">
type ReadModelValueOptions<TValue> = {
  props: {
    modelValue: TValue | undefined
  }
}

function readModelValue<TValue>(options: ReadModelValueOptions<TValue>) {
  return options.props.modelValue
}

const props = withDefaults(defineProps<FooProps>(), {
  as: 'input',
})

readModelValue({ props })
</script>

<template>{{ props.modelValue }}</template>
"#,
    );

    run_check_json(&project_root, &corsa_path, "src/Foo.vue");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_with_defaults_undefined_default_preserves_optional_prop_value() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case(
        "with-defaults-undefined-default",
        "Foo.vue",
        r#"<script setup lang="ts">
interface FooProps {
  modelValue?: string
}

type UseModelOptions<P, K extends keyof P> = {
  props: P
  key: K
  clone?: boolean | ((value: P[K]) => P[K])
}

function useModel<P, K extends keyof P>(options: UseModelOptions<P, K>) {
  void options
}

const props = withDefaults(defineProps<FooProps>(), {
  modelValue: undefined,
})

useModel({
  props,
  key: 'modelValue',
  clone: value => value === undefined ? undefined : value.trim(),
})
</script>

<template>{{ props.modelValue }}</template>
"#,
    );

    run_check_json(&project_root, &corsa_path, "src/Foo.vue");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_v_for_infers_items_from_union_of_arrays() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case(
        "vfor-array-union",
        "Foo.vue",
        r#"<script setup lang="ts">
import { computed, ref } from 'vue'

type ItemA = { key: string; valueA: number }
type ItemB = { key: string; valueB: string }

const showA = ref(true)
const items = computed<ItemA[] | ItemB[]>(() => (
  showA.value
    ? [{ key: 'a', valueA: 1 }]
    : [{ key: 'b', valueB: 'b' }]
))
</script>

<template>
  <div v-for="item in items" :key="item.key">
    {{ item.key }}
  </div>
</template>
"#,
    );

    run_check_json(&project_root, &corsa_path, "src/Foo.vue");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_define_props_return_assignable_to_record_helpers() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case(
        "define-props-record",
        "Foo.vue",
        r#"<script setup lang="ts">
type MaybeRefOrGetter<T> = T | (() => T)

interface FooProps {
  forceMount?: boolean
}

function forwardProps<T extends Record<string, unknown>>(value: MaybeRefOrGetter<T>) {
  return value
}

const props = defineProps<FooProps>()
forwardProps(props)
</script>

<template><div /></template>
"#,
    );

    run_check_json(&project_root, &corsa_path, "src/Foo.vue");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_extended_interface_props_are_available_in_template_scope() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case(
        "extended-interface-template-props",
        "Foo.vue",
        r#"<script setup lang="ts">
interface BaseProps {
  required?: boolean
}

interface FooProps extends BaseProps {
  label: string
}

defineProps<FooProps>()
</script>

<template>
  <label>
    <input :required="required" />
    {{ label }} {{ required }}
  </label>
</template>
"#,
    );

    run_check_json(&project_root, &corsa_path, "src/Foo.vue");

    let _ = std::fs::remove_dir_all(&project_root);
}
