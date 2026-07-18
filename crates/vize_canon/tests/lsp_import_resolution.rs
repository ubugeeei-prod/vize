use std::path::{Path, PathBuf};

use vize_canon::{CorsaBridge, CorsaBridgeConfig};
use vize_carton::ToCompactString;

#[test]
fn bridge_materialized_overlay_preserves_workspace_project_options() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().unwrap();
    let project_root = project.path();
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir(project_root.join("node_modules")).unwrap();

    let tsconfig = r#"{
  "compilerOptions": {
    "strict": false,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "noEmit": true,
    "paths": { "@/*": ["./src/*"] }
  },
  "include": ["src/**/*"]
}"#;
    std::fs::write(project_root.join("tsconfig.json"), tsconfig).unwrap();
    std::fs::write(src_dir.join("App.vue"), "<template><div /></template>\n").unwrap();
    std::fs::write(src_dir.join("Child.vue"), "<template><span /></template>\n").unwrap();
    std::fs::write(src_dir.join("util.ts"), "export const label = 'ok';\n").unwrap();

    let app_virtual_path = src_dir.join("App.vue.ts");
    let child_virtual_path = src_dir.join("Child.vue.ts");
    let app_virtual = "import Child from './Child.vue.ts';\nimport { label } from '@/util';\nfunction identity(value) { return value; }\nvoid Child;\nvoid label;\nvoid identity;\n";
    let child_virtual = "export default {};\n";

    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(project_root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let diagnostics = corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let child_uri = child_virtual_path.display().to_compact_string();
        bridge
            .open_or_update_virtual_document(child_uri.as_str(), child_virtual)
            .await
            .unwrap();
        let app_uri = app_virtual_path.display().to_compact_string();
        let app_uri = bridge
            .open_or_update_virtual_document(app_uri.as_str(), app_virtual)
            .await
            .unwrap();
        let diagnostics = bridge.get_diagnostics(app_uri.as_str()).await.unwrap();
        bridge.shutdown().await.unwrap();
        diagnostics
    });

    assert!(
        project_root
            .join("node_modules/.vize/corsa-overlay/tsconfig.json")
            .is_file(),
        "virtual overlays must activate the materialized project"
    );
    assert_eq!(
        diagnostics.len(),
        1,
        "unexpected diagnostics: {diagnostics:#?}"
    );
    assert_eq!(diagnostics[0].code, Some(serde_json::json!(7044)));
    assert_eq!(diagnostics[0].severity, Some(4));
    assert_eq!(
        diagnostics[0].message,
        "Parameter 'value' implicitly has an 'any' type, but a better type may be inferred from usage."
    );
    assert_eq!(
        std::fs::read_to_string(project_root.join("tsconfig.json")).unwrap(),
        tsconfig
    );
    let overlay_src = project_root.join("node_modules/.vize/corsa-overlay/src");
    assert_eq!(
        std::fs::read_to_string(overlay_src.join("App.vue.ts")).unwrap(),
        app_virtual
    );
    assert_eq!(
        std::fs::read_to_string(overlay_src.join("Child.vue.ts")).unwrap(),
        child_virtual
    );
    assert!(!app_virtual_path.exists());
    assert!(!child_virtual_path.exists());
}

fn resolve_test_tsgo_binary() -> Option<PathBuf> {
    let root = workspace_root();
    if let Some(resolved) = vize_carton::corsa_resolver::discover_corsa_in_ancestors(&root) {
        return Some(resolved);
    }
    [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.parent()?
            .join("corsa-bind/ref/corsa-upstream/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("vize_canon should live under crates/")
        .to_path_buf()
}
