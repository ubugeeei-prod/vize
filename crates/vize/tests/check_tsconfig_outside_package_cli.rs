#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
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
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    workspace_root().join("target/vize-tests/tests").join(
        cstr!(
            "check-tsconfig-outside-package-{name}-{}-{case_id}",
            std::process::id()
        )
        .as_str(),
    )
}

fn resolve_test_corsa_path() -> Option<String> {
    corsa_path::resolve(workspace_root().as_path())
}

fn write_file(root: &Path, relative_path: &str, content: &str) {
    let file_path = root.join(relative_path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
}

fn assert_external_declaration_is_checked(case_name: &str, tsconfig: &str) {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let workspace = unique_case_dir(case_name);
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace).unwrap();
    write_file(&workspace, "package.json", r#"{ "private": true }"#);
    write_file(&workspace, "app/package.json", r#"{ "private": true }"#);
    write_file(&workspace, "app/tsconfig.json", tsconfig);
    write_file(
        &workspace,
        "shared/globals.d.ts",
        r#"interface AmbientPayload {
  label: string;
}

declare const ambientPayload: AmbientPayload;
"#,
    );
    write_file(
        &workspace,
        "app/src/a.ts",
        r#"const label: string = ambientPayload.label;
export {};
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(workspace.join("app"))
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--tsconfig", "tsconfig.json", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["fileCount"], 2, "{stdout}\n{stderr}");

    let _ = std::fs::remove_dir_all(&workspace);
}

#[test]
fn check_keeps_tsconfig_files_entry_outside_nearest_package_root() {
    assert_external_declaration_is_checked(
        "files",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "files": ["../shared/globals.d.ts", "src/a.ts"]
}"#,
    );
}

#[test]
fn check_keeps_tsconfig_include_entry_outside_nearest_package_root() {
    assert_external_declaration_is_checked(
        "include",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["../shared/**/*.d.ts", "src/**/*"]
}"#,
    );
}
