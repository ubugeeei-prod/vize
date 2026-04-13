use std::{path::Path, process::Command};

use vize_carton::cstr;

#[test]
fn check_json_reports_type_errors_via_project_typechecker() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "json-type-errors",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
        )],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let snapshot = serde_json::json!({
        "status": output.status.code(),
        "errorCount": json["errorCount"],
        "fileCount": json["fileCount"],
        "diagnostics": json["files"][0]["diagnostics"],
    });

    insta::with_settings!({
        snapshot_path => "snapshots"
    }, {
        insta::assert_snapshot!(
            "check_json_reports_type_errors_via_project_typechecker",
            serde_json::to_string_pretty(&snapshot).unwrap()
        );
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_json_reports_ts_importing_vue_errors() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "json-ts-vue-import",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
defineProps<{
  count: number
}>()
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
            ),
            (
                "src/main.ts",
                r#"import App from './App.vue'

type AppProps = InstanceType<typeof App>['$props']

const props: AppProps = {
  count: 'oops',
}

void props
"#,
            ),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let snapshot = serde_json::json!({
        "status": output.status.code(),
        "errorCount": json["errorCount"],
        "files": json["files"].as_array().unwrap().iter().map(|file| {
            serde_json::json!({
                "file": file["file"],
                "diagnostics": file["diagnostics"],
            })
        }).collect::<Vec<_>>(),
    });

    insta::with_settings!({
        snapshot_path => "snapshots"
    }, {
        insta::assert_snapshot!(
            "check_json_reports_ts_importing_vue_errors",
            serde_json::to_string_pretty(&snapshot).unwrap()
        );
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_can_emit_declarations() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "emit-declarations",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
export interface PublicProps {
  count: number
}

const props = defineProps<PublicProps>()
</script>

<template>
  <div>{{ props.count }}</div>
</template>
"#,
            ),
            (
                "src/index.ts",
                r#"export { default as App } from './App.vue'
"#,
            ),
        ],
    );
    let declaration_dir = project_root.join("types");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
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

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let declarations = collect_declaration_snapshot(&declaration_dir);
    let snapshot = serde_json::json!({
        "status": output.status.code(),
        "errorCount": json["errorCount"],
        "declarations": json["declarations"],
        "files": declarations,
    });

    insta::with_settings!({
        snapshot_path => "snapshots"
    }, {
        insta::assert_snapshot!(
            "check_can_emit_declarations",
            serde_json::to_string_pretty(&snapshot).unwrap()
        );
    });

    let _ = std::fs::remove_dir_all(&project_root);
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

fn relative_path(root: &Path, file: &Path) -> std::string::String {
    file.strip_prefix(root)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| file.display().to_string())
}

fn workspace_root() -> &'static std::path::Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    workspace_root()
        .join("__agent_only")
        .join("tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}

fn create_cli_project(name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    link_workspace_node_modules(&project_root).unwrap();
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
        return Some(sibling_cache.display().to_string());
    }

    for candidate in [
        workspace_root.join("node_modules/.bin/tsgo"),
        workspace_root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ] {
        if candidate.exists() {
            return Some(candidate.display().to_string());
        }
    }

    None
}

fn link_workspace_node_modules(project_root: &Path) -> std::io::Result<()> {
    let workspace_node_modules = resolve_workspace_node_modules();

    let target = project_root.join("node_modules");
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(&target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(&target)?;
    }
    std::fs::create_dir_all(&target)?;

    if let Some(ref workspace_node_modules) = workspace_node_modules {
        for package in ["vue", "vite", "@vue"] {
            let source = workspace_node_modules.join(package);
            if source.exists() {
                symlink_path(&source, &target.join(package))?;
            }
        }
    } else {
        write_test_vue_stub(&target)?;
        write_test_vite_stub(&target)?;
    }

    if let Some(corsa_path) = resolve_test_corsa_path() {
        let source = std::path::PathBuf::from(corsa_path);
        if source.exists() {
            let file_name = source.file_name().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid corsa binary path",
                )
            })?;
            symlink_path(
                &source,
                &target
                    .join("@typescript")
                    .join("native-preview")
                    .join("lib")
                    .join(file_name),
            )?;
            symlink_path(&source, &target.join(".bin").join(file_name))?;
        }
    }

    Ok(())
}

fn resolve_workspace_node_modules() -> Option<std::path::PathBuf> {
    let override_path = std::env::var_os("VIZE_TEST_WORKSPACE_NODE_MODULES");
    if let Some(override_path) = override_path {
        let override_path = std::path::PathBuf::from(override_path);
        if override_path.as_os_str() == "__none__" {
            return None;
        }
        return override_path.exists().then_some(override_path);
    }

    let workspace_node_modules = workspace_root().join("node_modules");
    workspace_node_modules
        .exists()
        .then_some(workspace_node_modules)
}

fn write_test_vue_stub(target: &Path) -> std::io::Result<()> {
    let vue_dir = target.join("vue");
    std::fs::create_dir_all(&vue_dir)?;
    std::fs::write(
        vue_dir.join("package.json"),
        r#"{
  "name": "vue",
  "types": "index.d.ts"
}"#,
    )?;
    std::fs::write(
        vue_dir.join("index.d.ts"),
        r#"export interface Ref<T = any, S = T> {
  value: T;
}

export interface ShallowRef<T = any, S = T> extends Ref<T, S> {}

export interface ComponentPublicInstance {
  $attrs: any;
  $slots: any;
  $refs: any;
  $emit: (...args: any[]) => void;
}

export declare function ref<T>(value: T): Ref<T>;
export declare function useTemplateRef<T = any>(key: string): ShallowRef<T | null>;
"#,
    )?;
    Ok(())
}

fn write_test_vite_stub(target: &Path) -> std::io::Result<()> {
    let vite_dir = target.join("vite");
    std::fs::create_dir_all(&vite_dir)?;
    std::fs::write(
        vite_dir.join("package.json"),
        r#"{
  "name": "vite",
  "types": "client.d.ts"
}"#,
    )?;
    std::fs::write(vite_dir.join("client.d.ts"), "")?;
    Ok(())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(target)?;
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        let metadata = std::fs::metadata(source)?;
        if metadata.is_dir() {
            std::os::windows::fs::symlink_dir(source, target)
        } else {
            std::os::windows::fs::symlink_file(source, target)
        }
    }
}
