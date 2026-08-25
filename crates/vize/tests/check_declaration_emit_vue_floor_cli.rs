//! `vize check --declaration` against a `vue` whose typings predate
//! `NativeElements` and `Directive` (Vue 2.7, an older Vue 3 minor, a trimmed or
//! shimmed package).
//!
//! The shared helpers file names `import('vue').NativeElements` once so native
//! element `v-bind` values can be checked against the element's own prop type,
//! and `import('vue').Directive` once so a custom directive's value can be
//! checked against its declared value type. When the installed `vue` does not
//! export them the aliases must degrade to unchecked. The diagnostics path drops
//! the resulting `TS2694` because the helpers text carries no mapping back to
//! authored source, but declaration emit treats any non-zero Corsa exit as
//! fatal, so the same unmapped diagnostic aborted the whole emit:
//!
//! ```text
//! Error: corsa error (exit code 1): __vize_helpers.d.ts(44,43): error TS2694:
//!   Namespace '".../node_modules/vue/index"' has no exported member 'NativeElements'.
//! ```
//!
//! This project writes its own `node_modules` instead of linking the
//! workspace's, so the floor is pinned regardless of which `vue` the developer
//! happens to have installed.

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{path::Path, process::Command};

use vize_s0::{String, ToCompactString, cstr};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<String> {
    let workspace_root = workspace_root();
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache.to_string_lossy().to_compact_string());
    }

    let workspace_bin = workspace_root.join("node_modules/.bin/tsgo");
    workspace_bin
        .exists()
        .then(|| workspace_bin.to_string_lossy().to_compact_string())
}

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#;

const APP_VUE: &str = r#"<script setup lang="ts">
export interface PublicProps {
  count: number
}

const props = defineProps<PublicProps>()
const vFocus = (_el: HTMLElement, _binding: { value: number }) => {}
</script>

<template>
  <a :href="props.count" v-focus="'nope'">{{ props.count }}</a>
</template>
"#;

const INDEX_TS: &str = "export { default as App } from './App.vue'\n";

/// A `vue` that stops short of `NativeElements` and `Directive`, keeping only
/// the surface the generated virtual modules name (`ComponentPublicInstance`,
/// `Ref`, `ShallowRef`, `DefineComponent`).
const VUE_WITHOUT_NATIVE_ELEMENTS_OR_DIRECTIVE: &str = r#"export interface ComponentPublicInstance<Props = {}> {
  $props: Props;
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: any[]) => void;
}

export type DefineComponent<Props = {}> = {
  new (): ComponentPublicInstance<Props>;
};

export type ComponentOptions<Props = {}> = {
  props?: Props;
  setup?: any;
  render?: Function;
};

export interface FunctionalComponent<P = {}> {
  (props: P, ctx: any): any;
}

export type ConcreteComponent<Props = {}> =
  | ComponentOptions<Props>
  | FunctionalComponent<Props>;

export interface Ref<T = unknown, _Raw = T> {
  value: T;
}

export interface ShallowRef<T = unknown, _Raw = T> extends Ref<T, _Raw> {
  readonly __v_isShallow?: true;
}

export type PropType<T> = { new (...args: any[]): T & {} } | { (): T } | null;

export declare const Transition: DefineComponent;
export declare function defineComponent(options: any): DefineComponent;
"#;

const VITE_CLIENT: &str = r#"interface ImportMetaEnv {
  readonly [key: string]: string | boolean | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

export {};
"#;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

fn create_project(corsa_path: &str) -> std::path::PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("declaration-emit-vue-floor-{}", std::process::id()).as_str());
    let _ = std::fs::remove_dir_all(&project_root);

    write_file(&project_root.join("tsconfig.json"), TSCONFIG);
    write_file(&project_root.join("src/App.vue"), APP_VUE);
    write_file(&project_root.join("src/index.ts"), INDEX_TS);

    let node_modules = project_root.join("node_modules");
    write_file(
        &node_modules.join("vue/package.json"),
        "{\n  \"name\": \"vue\",\n  \"types\": \"index.d.ts\"\n}\n",
    );
    write_file(
        &node_modules.join("vue/index.d.ts"),
        VUE_WITHOUT_NATIVE_ELEMENTS_OR_DIRECTIVE,
    );
    write_file(
        &node_modules.join("vite/package.json"),
        "{\n  \"name\": \"vite\",\n  \"types\": \"client.d.ts\"\n}\n",
    );
    write_file(&node_modules.join("vite/client.d.ts"), VITE_CLIENT);

    let corsa = Path::new(corsa_path);
    if let Some(file_name) = corsa.file_name() {
        let link = node_modules.join(".bin").join(file_name);
        std::fs::create_dir_all(link.parent().unwrap()).unwrap();
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink(corsa, &link);
        #[cfg(windows)]
        let _ = std::os::windows::fs::symlink_file(corsa, &link);
    }

    project_root
}

#[test]
fn declaration_emit_survives_a_vue_without_native_elements_or_directive() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project(corsa_path.as_str());

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path.as_str())
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

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    // The `:href="props.count"` binding is a `number` against `string`, and
    // `v-focus="'nope'"` is a `string` against the directive's `number` value.
    // With a `vue` that declares `NativeElements` and `Directive` both are
    // `TS2322`; without them each alias is an error type, so the checks accept
    // any value. Absent means unchecked, never an error.
    assert_eq!(json["errorCount"], serde_json::json!(0));
    let diagnostics = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|entry| entry["diagnostics"].as_array().unwrap())
        .map(|diagnostic| diagnostic.as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(diagnostics, Vec::<&str>::new());
    assert_eq!(
        json["declarations"],
        serde_json::json!([
            "types/App.vue.d.ts",
            "types/__vize_helpers.d.ts",
            "types/index.d.ts"
        ])
    );

    // Emitted declarations must not push vize's Vue floor onto consumers: the
    // aliases that name `NativeElements` and `Directive` stay out of the shipped
    // helper file and out of every emitted `.d.ts`.
    let emitted_helpers =
        std::fs::read_to_string(project_root.join("types/__vize_helpers.d.ts")).unwrap();
    let app_declaration = std::fs::read_to_string(project_root.join("types/App.vue.d.ts")).unwrap();
    for (name, contents) in [
        ("types/__vize_helpers.d.ts", &emitted_helpers),
        ("types/App.vue.d.ts", &app_declaration),
    ] {
        assert_eq!(
            contents
                .lines()
                .filter(|line| line.contains("__VizeNativeElement")
                    || line.contains("__VizeDirectiveValue"))
                .collect::<Vec<_>>(),
            Vec::<&str>::new(),
            "{name} must not ship the helpers that name Vue's own types"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}
