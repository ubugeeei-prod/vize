use super::{CorsaError, corsa_diagnostics_error_is_unsupported, diagnostics_api_is_unsupported};
use crate::{
    file_uri::{file_uri_to_path, path_to_file_uri},
    lsp_client::CorsaProjectClient,
    lsp_client::lsp_transport_error_is_transient,
};

// Regression: a missing diagnostics scope on the corsa `ProjectSession`
// must be detected through the typed `CorsaError::Unsupported` variant
// (corsa-bind raises it directly and normalizes "unknown method" RPC
// errors into it), so the project/file-diagnostics fallback gates on the
// variant rather than sniffing the rendered message. A genuine failure on
// a different variant must propagate instead of being silently swallowed.
#[test]
fn corsa_diagnostics_unsupported_uses_typed_capability_error() {
    assert!(corsa_diagnostics_error_is_unsupported(
        &CorsaError::Unsupported("file diagnostics are not supported")
    ));
    assert!(!corsa_diagnostics_error_is_unsupported(
        &CorsaError::Protocol("diagnostics request failed: process exited".into())
    ));
}

#[test]
fn recognizes_unsupported_diagnostics_api_errors() {
    assert!(diagnostics_api_is_unsupported("unknown API method"));
    assert!(diagnostics_api_is_unsupported(
        "unsupported: project diagnostics are not supported by this runtime"
    ));
    assert!(diagnostics_api_is_unsupported(
        "project diagnostics are not supported by this runtime"
    ));
    assert!(!diagnostics_api_is_unsupported(
        "Failed to request Corsa project diagnostics: process exited"
    ));
}

#[test]
fn recognizes_transient_lsp_transport_errors() {
    assert!(lsp_transport_error_is_transient(
        "protocol error: EOF while parsing a string at line 1 column 150"
    ));
    assert!(lsp_transport_error_is_transient(
        "Failed to request LSP diagnostics for file:///src/App.vue.ts: process is closed: jsonrpc reader"
    ));
    assert!(lsp_transport_error_is_transient("Broken pipe"));
    assert!(!lsp_transport_error_is_transient(
        "TypeScript semantic diagnostics are unavailable"
    ));
}

#[test]
fn virtual_vue_overlay_diagnostics_force_materialized_project_config() {
    let project = tempfile::tempdir().unwrap();
    let project_root = project.path().join("workspace");
    let src = project_root.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(project_root.join("tsconfig.json"), "{}").unwrap();
    std::fs::write(src.join("App.vue"), "<template><div /></template>").unwrap();

    let virtual_uri = path_to_file_uri(&src.join("App.vue.ts"));
    let mut client = CorsaProjectClient::empty_for_test(project_root);
    client
        .document_texts
        .insert(virtual_uri.clone(), "export {};".into());

    assert!(
        super::virtual_overlay_diagnostics::needs_materialized_project_config(
            &client,
            &virtual_uri
        )
    );

    let authored_uri = path_to_file_uri(&src.join("App.vue"));
    client
        .document_texts
        .insert(authored_uri.clone(), "<template><div /></template>".into());
    assert!(
        !super::virtual_overlay_diagnostics::needs_materialized_project_config(
            &client,
            &authored_uri
        )
    );

    let missing_backing_uri = path_to_file_uri(&src.join("Missing.vue.ts"));
    client
        .document_texts
        .insert(missing_backing_uri.clone(), "export {};".into());
    assert!(
        !super::virtual_overlay_diagnostics::needs_materialized_project_config(
            &client,
            &missing_backing_uri
        )
    );
}

#[test]
fn virtual_vue_overlay_diagnostics_materializes_for_lsp_fallback() {
    let project = tempfile::tempdir().unwrap();
    let project_root = project.path().join("workspace");
    let src = project_root.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(project_root.join("tsconfig.json"), "{}").unwrap();
    std::fs::write(src.join("App.vue"), "<template><div /></template>").unwrap();

    let virtual_uri = path_to_file_uri(&src.join("App.vue.ts"));
    let mut client = CorsaProjectClient::empty_for_test(project_root.clone());
    client
        .document_texts
        .insert(virtual_uri.clone(), "export {};".into());

    super::virtual_overlay_diagnostics::ensure_materialized_project(
        &mut client,
        std::iter::once(virtual_uri.as_str()),
    )
    .unwrap();

    assert!(client.materialized_project_session);
    let document_uri = client.session_document_uri(virtual_uri.as_str());
    assert_ne!(document_uri, virtual_uri);
    let document_path = file_uri_to_path(document_uri.as_str()).unwrap();
    assert_eq!(
        std::fs::read_to_string(document_path).unwrap(),
        "export {};"
    );

    let generated_config = client.overlay_storage.root().join("tsconfig.json");
    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(generated_config).unwrap()).unwrap();
    assert_eq!(
        config["compilerOptions"]["allowImportingTsExtensions"],
        serde_json::Value::Bool(true)
    );
}

// Diagnostics and position queries share one native document identity.
// Alternating them never opens a second program for the caller's spelling.
#[cfg(unix)]
#[test]
fn materialized_diagnostics_and_queries_mirror_one_document_set() {
    use serde_json::Value;
    use std::{fs, os::unix::fs::PermissionsExt};

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let script = root.join("peer.ts");
    let transcript = root.join("whole-packets.jsonl");
    fs::write(
        &script,
        include_str!("../editor_lsp/native_shutdown/peer.ts.txt")
            .replace("LOG_PATH", &serde_json::to_string(&transcript).unwrap())
            .replace("REPLY_MODE", "\"ok\"")
            .replace("request.method === \"textDocument/hover\"", "[\"textDocument/hover\",\"textDocument/definition\",\"textDocument/completion\",\"textDocument/codeAction\",\"textDocument/inlayHint\"].includes(request.method)"),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let mut client = CorsaProjectClient::batch_editor_peer_for_test(&script, &root).unwrap();
    client.materialized_project_session = true;
    let uris = ["App", "Child", "Leaf"]
        .map(|name| path_to_file_uri(&root.join(format!("src/{name}.vue.ts"))));
    for (index, uri) in uris.iter().enumerate() {
        client.document_texts.insert(
            uri.clone(),
            format!("export const value{index} = {index};").into(),
        );
    }
    client.editor_lsp_documents_dirty = true;

    for _ in 0..3 {
        client
            .request_diagnostics_batch_via_lsp_chunk_once(std::slice::from_ref(&uris[0]))
            .unwrap();
        client.hover_via_editor_lsp(&uris[0], 0, 0).unwrap();
        client.definition_via_editor_lsp(&uris[0], 0, 0).unwrap();
        client.completion_via_editor_lsp(&uris[0], 0, 0).unwrap();
        client
            .source_actions_via_editor_lsp(&uris[0], &["source.organizeImports".into()])
            .unwrap();
        client.inlay_hints_raw(&uris[0], (0, 0), (0, 1)).unwrap();
    }
    let native = uris
        .iter()
        .map(|uri| std::string::String::from(client.editor_document_uri(uri).as_str()))
        .collect::<Vec<_>>();
    assert_ne!(native[0], uris[0].as_str());
    client.shutdown().unwrap();

    let requests = fs::read_to_string(&transcript)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .filter(|row| row["direction"] == "request")
        .map(|row| row["wholePacket"].clone())
        .collect::<Vec<_>>();
    let documents = |method: &str| {
        requests
            .iter()
            .filter(|packet| packet["method"] == method)
            .map(|packet| {
                packet["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        documents("textDocument/didClose"),
        Vec::<std::string::String>::new()
    );
    let mut opened = documents("textDocument/didOpen");
    opened.sort();
    let mut expected = native.clone();
    expected.sort();
    assert_eq!(opened, expected, "each document opens under one identity");
    for method in [
        "textDocument/hover",
        "textDocument/definition",
        "textDocument/completion",
        "textDocument/codeAction",
        "textDocument/inlayHint",
    ] {
        assert_eq!(documents(method), vec![native[0].clone(); 3], "{method}");
    }
}
