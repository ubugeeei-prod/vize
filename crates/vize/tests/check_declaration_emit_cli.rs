#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "check_declaration_emit_cli/packed_consumer.rs"]
mod packed_consumer;
#[path = "check_declaration_emit_cli/project_reference_packed_consumer.rs"]
mod project_reference_packed_consumer;
#[path = "check_declaration_emit_cli/workspace_package_vue_exports.rs"]
mod workspace_package_vue_exports;

use std::{path::Path, process::Command};

use vize_carton::{String, ToCompactString, cstr};

#[path = "support/vue_stub.rs"]
mod vue_stub;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
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

fn link_workspace_node_modules(project_root: &Path) {
    let source = workspace_root().join("node_modules");
    let target = project_root.join("node_modules");
    if target.exists() {
        return;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

fn create_cli_project(name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    link_workspace_node_modules(&project_root);
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

#[test]
fn check_declaration_emit_reports_module_declaration_outputs() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_cli_project(
        "module-declaration-emit",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
export interface PublicProps {
  label: string
}

defineProps<PublicProps>()
</script>

<template>
  <p>{{ label }}</p>
</template>
"#,
            ),
            (
                "src/TsxWidget.vue",
                r#"<script setup lang="tsx">
export interface TsxProps {
  active: boolean
}

defineProps<TsxProps>()
const vnode = null as unknown
</script>

<template>
  <div />
</template>
"#,
            ),
            (
                "src/modern.mts",
                r#"export { default as App } from "./App.vue";
export type AppModule = typeof import("./App.vue");
export { default as TsxWidget } from "./TsxWidget.vue";
export type TsxWidgetModule = typeof import("./TsxWidget.vue");
export const answer = 42;
"#,
            ),
            (
                "src/legacy.cts",
                r#"export const legacy = "ok";
"#,
            ),
        ],
    );
    vue_stub::install_vue_jsx_type_stub(&project_root);

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
    let declarations = json["declarations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect::<Vec<_>>();

    assert!(
        declarations.contains(&"types/modern.d.mts"),
        "missing .d.mts declaration output: {declarations:#?}"
    );
    assert!(
        declarations.contains(&"types/legacy.d.cts"),
        "missing .d.cts declaration output: {declarations:#?}"
    );

    let modern_declaration = std::fs::read_to_string(project_root.join("types/modern.d.mts"))
        .expect("modern declaration should be emitted");
    assert!(
        modern_declaration.contains("./App.vue"),
        "declaration import should point at .vue:\n{modern_declaration}"
    );
    assert!(
        modern_declaration.contains(r#"typeof import("./App.vue")"#),
        "declaration import types should point at .vue:\n{modern_declaration}"
    );
    assert!(
        modern_declaration.contains("./TsxWidget.vue"),
        "TSX SFC declaration import should point at .vue:\n{modern_declaration}"
    );
    assert!(
        modern_declaration.contains(r#"typeof import("./TsxWidget.vue")"#),
        "TSX SFC declaration import types should point at .vue:\n{modern_declaration}"
    );
    assert!(
        !modern_declaration.contains(".vue.ts"),
        "declaration import should not leak virtual .vue.ts/.vue.tsx paths:\n{modern_declaration}"
    );
    let tsx_widget_declaration =
        std::fs::read_to_string(project_root.join("types/TsxWidget.vue.d.ts"))
            .expect("TSX widget declaration should be emitted");
    assert!(
        tsx_widget_declaration.contains("export interface TsxProps"),
        "TSX SFC declaration should preserve exported props:\n{tsx_widget_declaration}"
    );
    assert!(project_root.join("types/legacy.d.cts").is_file());

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_declaration_emit_rewrites_declaration_map_sources() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let app_source = r#"<script setup lang="ts">
export interface PublicProps {
  label: string
}

defineProps<PublicProps>()
</script>

<template>
  <p>{{ label }}</p>
</template>
"#;
    let index_source = r#"export { default as App } from "./App.vue";
"#;
    let project_root = create_cli_project(
        "declaration-map-sources",
        &[("src/App.vue", app_source), ("src/index.ts", index_source)],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "inlineSources": true,
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path.as_str())
        .args(["check", ".", "--format", "json", "--declaration"])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let app_map_path = project_root.join("dist/types/App.vue.d.ts.map");
    let index_map_path = project_root.join("dist/types/index.d.ts.map");
    let app_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&app_map_path).unwrap()).unwrap();
    let index_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&index_map_path).unwrap()).unwrap();

    assert_eq!(app_map["sources"], serde_json::json!(["../../src/App.vue"]));
    assert_eq!(
        index_map["sources"],
        serde_json::json!(["../../src/index.ts"])
    );
    if !app_map["sourcesContent"].is_null() {
        assert_eq!(app_map["sourcesContent"], serde_json::json!([app_source]));
    }
    if !index_map["sourcesContent"].is_null() {
        assert_eq!(
            index_map["sourcesContent"],
            serde_json::json!([index_source])
        );
    }

    let app_map_text = std::fs::read_to_string(app_map_path).unwrap();
    let index_map_text = std::fs::read_to_string(index_map_path).unwrap();
    assert!(!app_map_text.contains(".vize"), "{app_map_text}");
    assert!(!app_map_text.contains(".vue.ts"), "{app_map_text}");
    assert!(!index_map_text.contains(".vize"), "{index_map_text}");

    let _ = std::fs::remove_dir_all(&project_root);
}
