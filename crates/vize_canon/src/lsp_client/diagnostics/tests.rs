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

    let generated_config = project_root.join("node_modules/.vize/corsa-overlay/tsconfig.json");
    let config = std::fs::read_to_string(generated_config).unwrap();
    assert!(config.contains("\"allowImportingTsExtensions\":true"));
}
