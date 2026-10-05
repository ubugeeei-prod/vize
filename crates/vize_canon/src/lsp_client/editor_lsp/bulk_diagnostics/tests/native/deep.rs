//! Actual legal native chain refusal, owner retirement and whole LSP recovery.

use crate::{file_uri::path_to_file_uri, lsp_client::CorsaProjectClient};
use serde_json::json;
use std::path::Path;

#[test]
fn native_deep_chain_reader_refusal_retires_owner_and_preserves_whole_fallback() {
    run_profile(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/native-bulk-diagnostics/deep-return-chain.ts"
        )),
        true,
        "return-chain",
    );
}

#[test]
fn native_collapsed_property_chain_preserves_complete_bulk_and_same_owner() {
    run_profile(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/native-bulk-diagnostics/deep-chain.ts"
        )),
        false,
        "property-chain",
    );
}

fn run_profile(source: &str, reader_refusal: bool, witness: &str) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let executable = vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(repo),
        },
    )
    .unwrap();
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("tsconfig.json");
    let config_bytes = serde_json::to_vec(&json!({
        "compilerOptions":{"strict":true,"noEmit":true,"module":"ESNext","moduleResolution":"Bundler","target":"ESNext","types":[]},
        "files":["deep.ts","empty.ts"],
    })).unwrap();
    std::fs::write(&config, &config_bytes).unwrap();
    let path = root.path().join("deep.ts");
    let empty_path = root.path().join("empty.ts");
    std::fs::write(&path, source).unwrap();
    std::fs::write(&empty_path, "export {};\n").unwrap();
    let uri = path_to_file_uri(&path);
    let empty = path_to_file_uri(&empty_path);
    let uris = [uri.clone(), empty.clone(), uri.clone()];
    let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
    client.executable = executable.to_str().unwrap().into();
    client.explicit_project_config = Some(config.clone());
    client.document_texts.insert(uri.clone(), source.into());
    client
        .document_texts
        .insert(empty.clone(), "export {};\n".into());
    let editor = client.editor_lsp_session().unwrap();
    let original = uris
        .iter()
        .map(|uri| serde_json::to_value(editor.diagnostics(uri).unwrap()).unwrap())
        .collect::<Vec<_>>();
    let old_attachment =
        serde_json::to_value(&editor.configured_api.as_ref().unwrap().session).unwrap();
    client.begin_batch_test_receipt(true);
    let batch = client.request_diagnostics_batch(&uris);
    let observed = client.batch_test_receipt(&uri);
    let complete_cache = uris
        .iter()
        .map(|uri| serde_json::to_value(client.diagnostics.get(uri.as_str())).unwrap())
        .collect::<Vec<_>>();
    capture(
        witness,
        "original-generation",
        &json!({
            "nativeBinary":executable,"configuration":config_bytes,"originalSource":source,
            "originalWholeLsp":original,"wholeBatch":batch.as_ref().ok(),
            "wholeBatchError":batch.as_ref().err(),"completeCache":complete_cache,
            "oldAttachment":old_attachment,"observed":observed,
        }),
    );
    let batch = batch.unwrap();
    assert_eq!(batch.len(), uris.len());
    if reader_refusal {
        assert_eq!(observed["mode"], "editor-lsp");
        assert_eq!(
            observed["bulkFallback"]["outcome"],
            "whole-original-fallback"
        );
        assert!(
            observed["bulkFallback"]["cause"]
                .as_str()
                .unwrap()
                .contains("recursion limit exceeded")
        );
        assert_ne!(observed["attachment"], old_attachment);
        assert_eq!(
            observed["bulkFallback"]["wholeFallbackError"],
            serde_json::Value::Null
        );
    } else {
        assert_eq!(observed["mode"], "native-bulk");
        assert_eq!(observed["attachment"], old_attachment);
        assert_eq!(observed["bulk"]["release"], "acknowledged");
        assert!(observed.get("bulkFallback").is_none());
    }
    for (actual, expected) in complete_cache.iter().zip(&original) {
        assert_eq!(expected["kind"], "full");
        assert_eq!(actual, &expected["items"]);
    }
    assert!(!original[0]["items"].as_array().unwrap().is_empty());

    // The new owner must also support its retained API on a later generation.
    client
        .document_texts
        .insert(uri.clone(), "export const repaired: number = 1;\n".into());
    client.editor_lsp_documents_dirty = true;
    client.begin_batch_test_receipt(true);
    let repaired = client.request_diagnostics_batch(&uris);
    let healthy = client.batch_test_receipt(&uri);
    capture(
        witness,
        "repaired-generation",
        &json!({
            "wholeBatch":repaired.as_ref().ok(),"wholeBatchError":repaired.as_ref().err(),
            "observed":healthy,"previousFallback":observed,
        }),
    );
    let repaired = repaired.unwrap();
    assert!(
        repaired
            .iter()
            .all(|(_, diagnostics)| diagnostics.is_empty())
    );
    assert_eq!(healthy["mode"], "native-bulk");
    assert_eq!(healthy["bulk"]["outcome"], "complete");
    assert_eq!(healthy["bulk"]["release"], "acknowledged");
    assert_eq!(healthy["attachment"], observed["attachment"]);
    client.retire_editor_lsp().unwrap();
    assert_eq!(std::fs::read(&config).unwrap(), config_bytes);
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    assert_eq!(std::fs::read(&empty_path).unwrap(), b"export {};\n");
}

fn capture(witness: &str, phase: &str, value: &serde_json::Value) {
    if let Some(dir) = std::env::var_os("VIZE_NATIVE_BULK_CAPTURE_DIR") {
        let dir = Path::new(&dir).join("deep-chain").join(witness).join(phase);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("whole-diagnostics.json"),
            serde_json::to_vec_pretty(value).unwrap(),
        )
        .unwrap();
    }
}
