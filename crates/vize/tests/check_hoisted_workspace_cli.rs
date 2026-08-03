#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{path::Path, process::Command};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn unique_case_dir() -> std::path::PathBuf {
    workspace_root()
        .join("target/vize-tests/tests")
        .join(format!("hoisted-workspace-{}", std::process::id()))
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
}

fn symlink_dir(source: &Path, target: &Path) {
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

fn resolve_test_corsa_path() -> Option<std::path::PathBuf> {
    let sibling_cache = workspace_root().parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache);
    }
    let workspace_bin = workspace_root().join("node_modules/.bin/tsgo");
    workspace_bin.exists().then_some(workspace_bin)
}

#[test]
fn check_resolves_hoisted_workspace_package_with_explicit_types() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };

    let workspace = unique_case_dir();
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace).unwrap();

    write_file(
        &workspace,
        "packages/lib/package.json",
        r#"{
  "name": "@repro/lib",
  "private": true,
  "type": "module",
  "exports": { ".": "./src/index.ts" }
}"#,
    );
    write_file(
        &workspace,
        "packages/lib/src/index.ts",
        "export const answer: number = 42;\n",
    );
    write_file(
        &workspace,
        "node_modules/@types/node/package.json",
        r#"{ "name": "@types/node", "version": "1.0.0", "types": "index.d.ts" }"#,
    );
    write_file(
        &workspace,
        "node_modules/@types/node/index.d.ts",
        "export {}; declare global { const process: { pid: number }; }\n",
    );

    let app = workspace.join("app");
    write_file(
        &app,
        "package.json",
        r#"{ "name": "@repro/app", "private": true, "type": "module" }"#,
    );
    write_file(
        &app,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "skipLibCheck": true,
    "noEmit": true,
    "types": ["node"]
  },
  "include": ["src/**/*.ts"]
}"#,
    );
    write_file(
        &app,
        "src/main.ts",
        concat!(
            "import { answer } from \"@repro/lib\";\n\n",
            "export const doubled: number = answer * 2;\n",
            "export const pid: number = process.pid;\n",
        ),
    );

    symlink_dir(
        &workspace.join("packages/lib"),
        &app.join("node_modules/@repro/lib"),
    );
    symlink_dir(
        &workspace.join("node_modules/@types/node"),
        &app.join("node_modules/@types/node"),
    );
    assert!(
        std::fs::symlink_metadata(app.join("node_modules/@repro/lib"))
            .unwrap()
            .file_type()
            .is_symlink(),
        "the regression requires a hoisted workspace symlink"
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&app)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--no-config", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "hoisted workspace check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["warningCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["fileCount"], 1, "{stdout}\n{stderr}");
    assert!(
        !stdout.contains("Cannot find module '@repro/lib'"),
        "workspace package should resolve through app/node_modules:\n{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&workspace);
}
