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
        .join(cstr!("check-canon-type-{name}-{}", std::process::id()).as_str())
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

fn create_case_with_files(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    for (path, source) in files {
        let target = project_root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(target, source).unwrap();
    }
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
fn check_imported_base_interface_props_are_available_in_template_scope() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case_with_files(
        "imported-base-interface-template-props",
        &[
            (
                "src/base.ts",
                "export interface BaseProps { as?: string; decorative?: boolean; }\n",
            ),
            (
                "src/Foo.vue",
                r#"<script setup lang="ts">
import type { BaseProps } from "./base";
interface FooProps extends BaseProps { orientation?: "horizontal" | "vertical"; }
const props = withDefaults(defineProps<FooProps>(), { orientation: "horizontal" });
</script>
<template><div :as :data-decorative="decorative" :data-orientation="props.orientation" /></template>
"#,
            ),
        ],
    );

    run_check_json(&project_root, &corsa_path, "src");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_contextmenu_accepts_pointer_event_handler() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case_with_files(
        "contextmenu-pointer-event",
        &[(
            "src/Foo.vue",
            r#"<script setup lang="ts">
async function onPointer(event: PointerEvent) { event.preventDefault(); }
</script>
<template><button @contextmenu="onPointer" /></template>
"#,
        )],
    );

    run_check_json(&project_root, &corsa_path, "src");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_generic_sfc_value_can_be_specialized_from_typescript() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case_with_files(
        "generic-sfc-value-specialization",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T = string">
defineProps<{ modelValue?: T }>();
</script>
<template><div /></template>
"#,
            ),
            (
                "src/main.ts",
                "import Child from \"./Child.vue\";\nexport const NumberChild = Child<number>;\n",
            ),
        ],
    );

    run_check_json(&project_root, &corsa_path, "src");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_exposed_template_ref_is_unwrapped_on_component_instance() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case_with_files(
        "exposed-template-ref-instance-unwrap",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
import { useTemplateRef } from "vue";

const element = useTemplateRef<HTMLElement>("element");

defineExpose({ element });
</script>

<template>
  <div ref="element" />
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import { useTemplateRef } from "vue";

import Child from "./Child.vue";

const child = useTemplateRef<InstanceType<typeof Child>>("child");
</script>

<template>
  <Child ref="child" />
  {{ child?.element?.id }}
</template>
"#,
            ),
        ],
    );

    run_check_json(&project_root, &corsa_path, "src");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_with_defaults_generic_props_do_not_gain_boolean_intersections() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case_with_files(
        "with-defaults-generic-boolean-keys",
        &[(
            "src/Foo.vue",
            r#"<script lang="ts">
interface Props<T> {
  as?: string;
  value: T;
}
</script>

<script setup lang="ts" generic="T extends Record<string, unknown>">
const props = withDefaults(defineProps<Props<T>>(), {
  as: "div"
});

function getKey(value: Record<PropertyKey, unknown>) {
  return String(value.id);
}

getKey(props.value);
</script>
"#,
        )],
    );

    run_check_json(&project_root, &corsa_path, "src");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_generic_define_props_preserves_local_boolean_keys() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case_with_files(
        "generic-define-props-local-boolean-keys",
        &[(
            "src/Foo.vue",
            r#"<script setup lang="ts" generic="T extends Record<string, unknown>">
interface Props<T> {
  enabled?: boolean;
  value: T;
}

const props = defineProps<Props<T>>();
const enabled: boolean = props.enabled;

function getKey(value: Record<PropertyKey, unknown>) {
  return String(value.id);
}

getKey(props.value);
</script>
"#,
        )],
    );

    run_check_json(&project_root, &corsa_path, "src");

    let _ = std::fs::remove_dir_all(&project_root);
}
