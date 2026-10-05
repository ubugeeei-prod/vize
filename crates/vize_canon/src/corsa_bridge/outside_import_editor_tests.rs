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

pub(super) fn assert_complete_alias_response(
    diagnostics: &[super::LspDiagnostic],
    root: &Path,
    document: &super::CorsaVueVirtualDocument,
) {
    let complete = serde_json::to_value(diagnostics).unwrap();
    if let Some(capture) = std::env::var_os("VIZE_TYPECHECK_REGRESSION_CAPTURE") {
        let capture = PathBuf::from(capture).join("editor-monorepo-alias");
        std::fs::create_dir_all(&capture).unwrap();
        std::fs::write(
            capture.join("diagnostics.json"),
            serde_json::to_vec_pretty(&complete).unwrap(),
        )
        .unwrap();
        std::fs::write(capture.join("generated.vue.ts"), document.code.as_bytes()).unwrap();
        std::fs::write(capture.join("runtime.json"), serde_json::to_vec_pretty(&json!({"nativeBinary": std::env::var_os("CORSA_PATH").unwrap(), "workingDirectory": root, "timeoutMs": 30_000})).unwrap()).unwrap();
        let config = document
            .session_project_root
            .as_ref()
            .unwrap()
            .join("packages/web/tsconfig.json");
        std::fs::copy(config, capture.join("generated-tsconfig.json")).unwrap();
        copy_inputs(root, &capture.join("inputs"));
    }
    // The raw native protocol retains four unused generated type suggestions.
    // Assert every field; package-resolution errors or new suggestions must fail.
    assert_eq!(
        complete,
        json!([
            {"range":{"start":{"line":41,"character":5},"end":{"line":41,"character":10}},"severity":4,"code":6196,"source":"ts","message":"'__Ref' is declared but never used.","relatedInformation":null},
            {"range":{"start":{"line":49,"character":5},"end":{"line":49,"character":19}},"severity":4,"code":6196,"source":"ts","message":"'__VizePrettify' is declared but never used.","relatedInformation":null},
            {"range":{"start":{"line":51,"character":5},"end":{"line":51,"character":36}},"severity":4,"code":6196,"source":"ts","message":"'__VizeComponentFallthroughProps' is declared but never used.","relatedInformation":null},
            {"range":{"start":{"line":58,"character":5},"end":{"line":58,"character":37}},"severity":4,"code":6196,"source":"ts","message":"'__VizeComponentMissingInputGuard' is declared but never used.","relatedInformation":null}
        ])
    );
}

fn copy_inputs(source: &Path, target: &Path) {
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == ".vize" {
            continue;
        }
        let output = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_inputs(&entry.path(), &output);
        } else {
            std::fs::create_dir_all(output.parent().unwrap()).unwrap();
            std::fs::copy(entry.path(), output).unwrap();
        }
    }
}
