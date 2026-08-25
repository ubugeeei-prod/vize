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
        .join(cstr!("check-ambient-export-{name}-{}", std::process::id()).as_str())
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        return Some(PathBuf::from(path));
    }
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
fn check_allows_export_assignment_inside_ambient_module_declaration() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("cjs-module");
    let _ = std::fs::remove_dir_all(&project_root);

    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true
  },
  "include": ["src/**/*.d.ts"]
}"#,
    );
    write(
        &project_root,
        "src/ambients/buffer-from.d.ts",
        r#"declare interface Buffer {}

declare module "buffer-from" {
  function bufferFrom(arrayBuffer: ArrayBuffer): Buffer;
  export = bufferFrom;
}
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
            "src/ambients/buffer-from.d.ts",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "ambient CommonJS declaration should type-check\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    assert!(
        !stdout.contains("TS1203"),
        "ambient export assignment must not be rejected as an ES module export:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
