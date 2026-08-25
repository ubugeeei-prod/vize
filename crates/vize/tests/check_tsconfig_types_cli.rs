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
        .join(cstr!("check-tsconfig-types-{name}-{}", std::process::id()).as_str())
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

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[test]
fn check_loads_compiler_options_types_from_tsconfig_ambient_declarations() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("compiler-options-types");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write(
        &project_root,
        "node_modules/@nuxtjs/composition-api/package.json",
        r#"{ "types": "index.d.ts" }"#,
    );
    write(
        &project_root,
        "node_modules/@nuxtjs/composition-api/index.d.ts",
        r#"export interface UseContextReturn {}
export function useContext(): UseContextReturn;
"#,
    );
    write(
        &project_root,
        "node_modules/nuxt-i18n/package.json",
        r#"{ "types": "index.d.ts" }"#,
    );
    write(
        &project_root,
        "node_modules/nuxt-i18n/index.d.ts",
        r#"import "@nuxtjs/composition-api";

declare module "@nuxtjs/composition-api" {
  interface UseContextReturn {
    app: {
      i18n: {
        t(key: string): string;
      };
    };
  }
}
"#,
    );
    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "types": ["nuxt-i18n"]
  },
  "include": ["src/**/*"]
}"#,
    );
    write(
        &project_root,
        "src/App.vue",
        r#"<script setup lang="ts">
import { useContext } from "@nuxtjs/composition-api";

const label: string = useContext().app.i18n.t("ready");
void label;
</script>

<template><div /></template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src/App.vue",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    assert!(
        !stdout.contains("TS2339"),
        "compilerOptions.types should provide module augmentations:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_loads_parent_type_packages_without_mirroring_them_as_sources() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let workspace = unique_case_dir("parent-types");
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace).unwrap();
    link_workspace_vue(&workspace).unwrap();
    write(
        &workspace,
        "node_modules/@types/vize-external/index.d.ts",
        "declare const externalFixtureValue: string;\n",
    );
    write(&workspace, "packages/router/package.json", "{}\n");
    write(
        &workspace,
        "packages/router/tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "types": ["vize-external"],
    "noEmit": true
  }
}"#,
    );
    write(
        &workspace,
        "packages/playground/src/App.vue",
        r#"<script setup lang="ts">
const message: string = externalFixtureValue;
</script>

<template>{{ message }}</template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&workspace)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--tsconfig",
            "packages/router/tsconfig.json",
            "packages/playground/src/App.vue",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "parent type package check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("Failed to strip prefix from path"),
        "{stdout}\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["fileCount"], 1, "{stdout}\n{stderr}");

    let _ = std::fs::remove_dir_all(&workspace);
}
