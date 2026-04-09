use std::{path::Path, process::Command};

use tempfile::TempDir;

#[test]
fn check_json_reports_type_errors_via_lsp_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    link_workspace_node_modules(temp_dir.path()).unwrap();
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

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(workspace_root)
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

fn link_workspace_node_modules(project_root: &Path) -> std::io::Result<()> {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| std::io::Error::other("workspace root not found"))?;
    let workspace_node_modules = workspace_root.join("node_modules");
    if !workspace_node_modules.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace node_modules not found",
        ));
    }

    let target = project_root.join("node_modules");
    std::fs::create_dir_all(&target)?;
    for package in ["vue", "vite", "@vue"] {
        let source = workspace_node_modules.join(package);
        if source.exists() {
            symlink_path(&source, &target.join(package))?;
        }
    }

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
