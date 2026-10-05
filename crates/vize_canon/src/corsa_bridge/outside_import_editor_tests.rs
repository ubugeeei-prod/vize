//! Complete official editor vector for the original outside-import issue.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use serde_json::json;
use vize_carton::corsa_resolver::{CorsaResolveRequest, resolve_corsa_executable};

const ORIGINAL: &str = include_str!(
    "../../../../tests/_fixtures/differential/typechecker/outside-import-types/input.json"
);

#[test]
fn original_outside_import_preserves_types_in_native_editor() {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return;
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let explicit = std::env::var_os("VIZE_TEST_TSGO_PATH").map(PathBuf::from);
    let corsa = resolve_corsa_executable(CorsaResolveRequest {
        explicit_path: explicit.as_deref(),
        project_root: Some(workspace),
    });
    assert!(corsa.is_ok() || std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none());
    let Ok(corsa) = corsa else { return };
    let fixture: serde_json::Value = serde_json::from_str(ORIGINAL).unwrap();
    let root = tempfile::tempdir().unwrap();
    for (name, source) in fixture["files"].as_object().unwrap() {
        let file = root.path().join(name);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, source.as_str().unwrap()).unwrap();
    }
    let source = fixture["files"]["app/src/b.ts"].as_str().unwrap();
    let host = root.path().join("app/src/b.ts");
    let bridge = super::CorsaBridge::with_config(super::CorsaBridgeConfig {
        corsa_path: Some(corsa.clone()),
        working_dir: Some(root.path().to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });
    corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let options = crate::virtual_ts::VirtualTsOptions::default();
        let project = bridge
            .open_script_virtual_project(super::CorsaScriptVirtualDocumentRequest {
                source_path: &host,
                request_path: host.to_str().unwrap(),
                code: source,
                source_type: SourceType::ts(),
                options: Default::default(),
                overlays: &[],
                virtual_ts_options: &options,
            })
            .await
            .unwrap();
        let diagnostics = bridge
            .get_diagnostics(&project.document.request_uri)
            .await
            .unwrap();
        bridge.shutdown().await.unwrap();
        let complete = serde_json::to_value(diagnostics).unwrap();
        if let Some(capture) = std::env::var_os("VIZE_TYPECHECK_REGRESSION_CAPTURE") {
            let capture = PathBuf::from(capture).join("editor-original-script");
            std::fs::create_dir_all(&capture).unwrap();
            std::fs::write(
                capture.join("diagnostics.json"),
                serde_json::to_vec_pretty(&complete).unwrap(),
            )
            .unwrap();
            std::fs::write(capture.join("runtime.json"), serde_json::to_vec_pretty(&json!({"nativeBinary": corsa, "workingDirectory": root.path(), "requestUri": project.document.request_uri, "timeoutMs": 30_000})).unwrap()).unwrap();
            for (name, source) in fixture["files"].as_object().unwrap() {
                let file = capture.join("inputs").join(name);
                std::fs::create_dir_all(file.parent().unwrap()).unwrap();
                std::fs::write(file, source.as_str().unwrap()).unwrap();
            }
        }
        assert_eq!(
            complete,
            json!([{
                "range": {"start": {"line": 2, "character": 13}, "end": {"line": 2, "character": 16}},
                "severity": 1, "code": 2322, "source": "ts",
                "message": "Type 'number' is not assignable to type 'string'.",
                "relatedInformation": null
            }])
        );
    });
}
