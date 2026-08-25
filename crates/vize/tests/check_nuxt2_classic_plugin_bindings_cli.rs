#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};
use vize_s0::cstr;

#[test]
fn check_nuxt2_use_context_sees_classic_plugin_binding_injections() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project();

    write_file(&project_root, "nuxt.config.ts", "export default {}\n");
    write_file(
        &project_root,
        "tsconfig.json",
        r##"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["pages/**/*.vue", "plugins/**/*.ts", "types/**/*.d.ts"]
}"##,
    );
    write_file(
        &project_root,
        "types/nuxt.d.ts",
        r##"declare module "@nuxt/types" {
  export interface Context {
    app: NuxtAppOptions;
  }
  export interface NuxtAppOptions {}
  export type Plugin = (
    context: Context,
    provide: (key: string, value: unknown) => void
  ) => void;
}

declare module "@nuxtjs/composition-api" {
  export interface UseContextReturn
    extends Omit<import("@nuxt/types").Context, "route" | "query" | "from" | "params"> {}
  export function useContext(): UseContextReturn;
}

declare module "#app" {
  export interface NuxtApp {}
}
"##,
    );
    write_file(
        &project_root,
        "plugins/auth.ts",
        r#"import type { Plugin } from "@nuxt/types";

const plugin: Plugin = (_context, provide) => {
  provide("auth", {
    loggedIn: true,
  });
};

export default plugin;
"#,
    );
    write_file(
        &project_root,
        "pages/index.vue",
        r#"<script setup lang="ts">
import { useContext } from "@nuxtjs/composition-api";

const context = useContext();
context.$auth.loggedIn;
context.app.$auth.loggedIn;
</script>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "pages",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        output.status.success(),
        "classic Nuxt2 plugin bindings should type-check\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}");

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_project() -> PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join(cstr!("nuxt2-classic-plugin-bindings-{}", std::process::id()).as_str());
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    let node_modules = workspace_root().join("node_modules");
    if node_modules.exists() {
        symlink_path(&node_modules, &project_root.join("node_modules")).unwrap();
    }
    project_root
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }
    let candidate = workspace_root().join("node_modules/.bin/tsgo");
    candidate.exists().then_some(candidate)
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}
