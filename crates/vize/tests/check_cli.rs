use std::{path::Path, process::Command};

use tempfile::TempDir;

#[test]
fn check_json_reports_type_errors_via_lsp_fallback() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        temp_dir.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*.vue", "src/**/*.ts"]
}"#,
    )
    .unwrap();
    let app_vue = src_dir.join("App.vue");
    std::fs::write(
        &app_vue,
        r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
    )
    .unwrap();

    let workspace_root = workspace_root();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(workspace_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", app_vue.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
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
            "check_json_reports_type_errors_via_lsp_fallback",
            serde_json::to_string_pretty(&snapshot).unwrap()
        );
    });
}

fn workspace_root() -> &'static std::path::Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
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
