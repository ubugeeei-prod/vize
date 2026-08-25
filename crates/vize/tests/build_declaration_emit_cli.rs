#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{path::Path, process::Command};

use vize_s0::cstr;

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

fn resolve_test_corsa_path() -> Option<std::string::String> {
    corsa_path::resolve(workspace_root())
}

const BUTTON_VUE: &str = r#"<script setup lang="ts">
export interface ButtonProps {
  label: string
  count?: number
}

const props = defineProps<ButtonProps>()

const emit = defineEmits<{
  change: [value: number]
}>()

function onClick() {
  emit('change', props.count ?? 0)
}
</script>

<template>
  <button type="button" @click="onClick">{{ label }}</button>
</template>
"#;

fn list_files(root: &Path) -> Vec<std::string::String> {
    let mut files = Vec::new();
    list_files_recursive(root, root, &mut files);
    files.sort();
    files
}

fn list_files_recursive(root: &Path, current: &Path, files: &mut Vec<std::string::String>) {
    let Ok(entries) = std::fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            list_files_recursive(root, &path, files);
            continue;
        }
        files.push(relative_path(root, &path));
    }
}

fn relative_path(root: &Path, file: &Path) -> std::string::String {
    file.strip_prefix(root)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| file.display().to_string())
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

#[test]
fn build_dts_emits_declarations_alongside_compiled_js() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_cli_project("build-dts-emit", &[("src/Button.vue", BUTTON_VUE)]);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path.as_str())
        .args(["build", "src/Button.vue", "--output", "dist", "--dts"])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let dist = project_root.join("dist");
    let snapshot = serde_json::json!({
        "status": output.status.code(),
        "outputs": list_files(&dist),
        "declarations": collect_declaration_snapshot(&dist),
    });

    insta::with_settings!({
        snapshot_path => "snapshots"
    }, {
        insta::assert_snapshot!(
            "build_dts_emits_declarations_alongside_compiled_js",
            serde_json::to_string_pretty(&snapshot).unwrap()
        );
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn build_declaration_dir_redirects_declaration_outputs() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_cli_project(
        "build-dts-declaration-dir",
        &[("src/Button.vue", BUTTON_VUE)],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path.as_str())
        .args([
            "build",
            "src/Button.vue",
            "--output",
            "dist",
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

    assert_eq!(
        list_files(&project_root.join("dist")),
        vec![std::string::String::from("Button.js")]
    );
    assert_eq!(
        list_files(&project_root.join("types")),
        vec![
            std::string::String::from("Button.vue.d.ts"),
            std::string::String::from("__vize_helpers.d.ts"),
        ]
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn build_without_declaration_flag_emits_no_declarations() {
    let project_root =
        create_cli_project("build-no-declaration", &[("src/Button.vue", BUTTON_VUE)]);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["build", "src/Button.vue", "--output", "dist"])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    assert_eq!(
        list_files(&project_root.join("dist")),
        vec![std::string::String::from("Button.js")]
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn build_declaration_rejects_stats_format() {
    let project_root = create_cli_project(
        "build-dts-stats-conflict",
        &[("src/Button.vue", BUTTON_VUE)],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "build",
            "src/Button.vue",
            "--format",
            "stats",
            "--declaration",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(1), "stdout:\n{stdout}");
    assert_eq!(
        stderr,
        "\u{1b}[31mError:\u{1b}[0m --declaration cannot be combined with --format stats\n"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
