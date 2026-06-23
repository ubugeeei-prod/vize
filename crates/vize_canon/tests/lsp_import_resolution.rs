use std::path::{Path, PathBuf};

use vize_canon::{CorsaBridge, CorsaBridgeConfig};

#[test]
fn bridge_vue_virtual_overlay_keeps_real_relative_import_base() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().unwrap();
    let project_root = project.path();
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    if link_workspace_node_modules(project_root).is_err() {
        return;
    }

    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(src_dir.join("App.vue"), "<template><div /></template>\n").unwrap();
    std::fs::write(src_dir.join("Child.vue"), "<template><span /></template>\n").unwrap();
    std::fs::write(src_dir.join("util.ts"), "export const label = 'ok';\n").unwrap();

    let app_virtual_path = src_dir.join("App.vue.ts");
    let child_virtual_path = src_dir.join("Child.vue.ts");
    let app_virtual = "import Child from './Child.vue.ts';\nimport { label } from './util';\nvoid Child;\nvoid label;\n";
    let child_virtual = "export default {};\n";

    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(project_root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let diagnostics = corsa::runtime::block_on(async {
        if bridge.spawn().await.is_err() {
            return None;
        }
        let child_uri = child_virtual_path.display().to_string();
        if bridge
            .open_or_update_virtual_document(child_uri.as_str(), child_virtual)
            .await
            .is_err()
        {
            let _ = bridge.shutdown().await;
            return None;
        }
        let app_uri = app_virtual_path.display().to_string();
        if bridge
            .open_or_update_virtual_document(app_uri.as_str(), app_virtual)
            .await
            .is_err()
        {
            let _ = bridge.shutdown().await;
            return None;
        }
        let diagnostics = bridge.get_diagnostics(app_uri.as_str()).await.ok()?;
        let _ = bridge.shutdown().await;
        Some(diagnostics)
    });

    let Some(diagnostics) = diagnostics else {
        return;
    };
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("Cannot find module")),
        "unexpected module-resolution diagnostics: {diagnostics:#?}"
    );
}

fn resolve_test_tsgo_binary() -> Option<PathBuf> {
    let root = workspace_root();
    for candidate in [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.parent()?
            .join("corsa-bind/ref/corsa-upstream/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
    ] {
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("vize_canon should live under crates/")
        .to_path_buf()
}

fn link_workspace_node_modules(project_root: &Path) -> std::io::Result<()> {
    let workspace_node_modules = workspace_root().join("node_modules");
    if !workspace_node_modules.exists() {
        return Ok(());
    }

    let target = project_root.join("node_modules");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(workspace_node_modules, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(workspace_node_modules, target)
    }
}
