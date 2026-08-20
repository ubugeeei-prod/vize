use std::path::{Path, PathBuf};
use std::process::Command;

use vize_carton::cstr;

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
        .join(cstr!("check-canon-fallthrough-{name}-{}", std::process::id()).as_str())
}

pub(super) fn resolve_test_corsa_path() -> Option<PathBuf> {
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

pub(super) fn create_case(name: &str, child: &str, app: &str) -> PathBuf {
    create_case_with_files(name, child, app, &[])
}

pub(super) fn create_case_with_files(
    name: &str,
    child: &str,
    app: &str,
    extra_files: &[(&str, &str)],
) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    std::fs::write(project_root.join("src/Child.vue"), child).unwrap();
    std::fs::write(project_root.join("src/App.vue"), app).unwrap();
    for (path, source) in extra_files {
        let file_path = project_root.join("src").join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(file_path, source).unwrap();
    }
    project_root
}

pub(super) fn run_check_json(project_root: &Path, corsa_path: &Path) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success() || (output.status.code() == Some(1) && !stdout.trim().is_empty()),
        "check crashed\nstdout:\n{}\nstderr:\n{}",
        stdout,
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
    );
    serde_json::from_str(stdout).unwrap()
}

fn diagnostics(report: &serde_json::Value) -> Vec<&str> {
    report["files"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|file| file["diagnostics"].as_array().into_iter().flatten())
        .filter_map(serde_json::Value::as_str)
        .collect()
}

pub(super) fn assert_clean(case_id: &str, report: &serde_json::Value) {
    let diagnostics = diagnostics(report);
    assert_eq!(
        report["errorCount"],
        serde_json::json!(0),
        "{case_id} should stay clean: {diagnostics:#?}"
    );
}

pub(super) fn assert_error_mentions(case_id: &str, report: &serde_json::Value, expected: &[&str]) {
    let diagnostics = diagnostics(report);
    assert!(
        report["errorCount"].as_u64().is_some_and(|count| count > 0),
        "{case_id} should report an error, got none: {report:#}"
    );
    for fragment in expected {
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains(fragment)),
            "{case_id} should mention {fragment:?}: {diagnostics:#?}"
        );
    }
}
