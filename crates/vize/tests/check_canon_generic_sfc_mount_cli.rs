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
        .join(cstr!("check-canon-mount-{name}-{}", std::process::id()).as_str())
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

fn run_check_json(project_root: &Path, corsa_path: &Path, target: &str) {
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
}

#[test]
fn check_generic_sfc_props_infer_from_typescript_mount_options() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case_with_files(
        "generic-sfc-mount-props",
        &[
            (
                "src/Child.vue",
                r#"<script lang="ts">
type ItemKey<T> = Extract<keyof T, string> | (string & {});

export interface ChildProps<
  TItems extends object[] = object[],
  TValueKey extends ItemKey<TItems[number]> | undefined = undefined
> {
  items: TItems;
  valueKey?: TValueKey;
}
</script>

<script setup lang="ts" generic="
  TItems extends object[] = object[],
  TValueKey extends ItemKey<TItems[number]> | undefined = undefined
">
defineProps<ChildProps<TItems, TValueKey>>();
</script>

<template><div /></template>
"#,
            ),
            (
                "src/usage.ts",
                r#"import { mount } from "@vue/test-utils";
import Child from "./Child.vue";

const items = [{ label: "One", value: "one" }];

mount(Child, {
  props: {
    items,
    valueKey: "value",
  },
});
"#,
            ),
            (
                "node_modules/vue-component-type-helpers/index.d.ts",
                r#"export type ComponentProps<T> = T extends new (...args: any) => {
  $props: infer P;
} ? NonNullable<P> : T extends (props: infer P, ...args: any) => any ? P : {};
"#,
            ),
            (
                "node_modules/@vue/test-utils/index.d.ts",
                r#"import type { ComponentProps } from "vue-component-type-helpers";

type RawProps = Record<string, any>;
interface MountingOptions<Props> {
  props?: RawProps & Props;
}
export type ComponentMountingOptions<
  T,
  P extends ComponentProps<T> = ComponentProps<T>,
> = MountingOptions<P> & Record<string, unknown>;
export declare function mount<
  T,
  C = T extends ((...args: any) => any) | (new (...args: any) => any) ? T : any,
  P extends ComponentProps<C> = ComponentProps<C>,
>(originalComponent: T, options?: ComponentMountingOptions<C, P>): void;
"#,
            ),
        ],
    );

    run_check_json(&project_root, &corsa_path, "src");

    let _ = std::fs::remove_dir_all(&project_root);
}
