use std::{
    path::{Path, PathBuf},
    process::Command,
};

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
        .join(format!(
            "check-nuxt-bridge-shims-{name}-{}",
            std::process::id()
        ))
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

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[test]
fn check_suppresses_legacy_vue_shim_and_nuxt_bridge_global_duplicates() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("gtag");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src/types/thirdParty")).unwrap();
    std::fs::create_dir_all(project_root.join("plugins")).unwrap();
    write(
        &project_root,
        "node_modules/vue/package.json",
        r#"{ "name": "vue", "types": "types/index.d.ts" }"#,
    );
    write(
        &project_root,
        "node_modules/vue/types/index.d.ts",
        r#"export interface Vue { $attrs: Record<string, unknown>; }
declare const VueConstructor: { version: string };
export default VueConstructor;
"#,
    );
    write(
        &project_root,
        "node_modules/@nuxt/types/package.json",
        r#"{ "name": "@nuxt/types", "types": "index.d.ts" }"#,
    );
    write(
        &project_root,
        "node_modules/@nuxt/types/index.d.ts",
        r#"declare module "@nuxt/types" {
  export interface Context {}
  export interface NuxtAppOptions {}
}
"#,
    );
    write(
        &project_root,
        "node_modules/@nuxt/bridge-schema/package.json",
        r#"{ "name": "@nuxt/bridge-schema", "types": "index.d.ts" }"#,
    );
    write(
        &project_root,
        "node_modules/@nuxt/bridge-schema/index.d.ts",
        r#"declare module "@nuxt/bridge-schema" {
  export interface Context { $gtag: any; }
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
    "types": ["@nuxt/types", "@nuxt/bridge-schema"]
  },
  "include": ["ts-shim.d.ts", "plugins/**/*", "src/**/*"]
}"#,
    );
    write(
        &project_root,
        "ts-shim.d.ts",
        r#"declare module "*.vue" {
  import Vue from "vue";
  export default Vue;
}
"#,
    );
    write(
        &project_root,
        "plugins/gtag.ts",
        r#"export default (_ctx: unknown, inject: (key: string, value: unknown) => void) => {
  inject("gtag", () => {});
};
"#,
    );
    write(
        &project_root,
        "src/types/thirdParty/gtag.d.ts",
        r#"declare namespace Gtag { type Gtag = (event: string) => void; }
declare module "@nuxt/bridge-schema" {
  export interface Context { $gtag: Gtag.Gtag; }
}
"#,
    );
    write(
        &project_root,
        "src/App.vue",
        r#"<script lang="ts">
export default { mounted() { const ready = true; void ready; } };
</script>
<template><div /></template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    for unexpected in ["TS2300", "TS2717", "Duplicate identifier", "$gtag"] {
        assert!(
            !stdout.contains(unexpected) && !stderr.contains(unexpected),
            "diagnostics should not mention {unexpected}:\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}
