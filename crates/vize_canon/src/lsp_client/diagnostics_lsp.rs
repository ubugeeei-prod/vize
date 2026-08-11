use crate::file_uri::path_to_file_uri;
use corsa::runtime::block_on;
use corsa_lsp::LspClient;
use lsp_types::{DocumentDiagnosticReportResult, Uri};
use serde_json::Value;
use std::path::Path;
use vize_carton::{String, cstr};

pub(super) fn initialize_lsp_client(client: &LspClient, project_root: &Path) -> Result<(), String> {
    struct InitializeRequest;

    impl lsp_types::request::Request for InitializeRequest {
        type Params = serde_json::Value;
        type Result = serde_json::Value;
        const METHOD: &'static str = "initialize";
    }

    struct InitializedNotification;

    impl lsp_types::notification::Notification for InitializedNotification {
        type Params = serde_json::Value;
        const METHOD: &'static str = "initialized";
    }

    let root_uri = path_to_file_uri(project_root);
    let workspace_name = project_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace");
    block_on(client.request::<InitializeRequest>(serde_json::json!({
        "processId": std::process::id(),
        "rootPath": project_root,
        "rootUri": root_uri,
        "workspaceFolders": [{
            "uri": root_uri,
            "name": workspace_name,
        }],
        "capabilities": {
            "textDocument": {
                "publishDiagnostics": {},
                "diagnostic": {
                    "dynamicRegistration": false,
                    "relatedDocumentSupport": true,
                }
            },
            "workspace": {
                "diagnostic": {
                    "refreshSupport": true,
                }
            }
        }
    })))
    .map_err(|error| cstr!("Failed to initialize Corsa LSP session: {error}"))?;
    client
        .notify::<InitializedNotification>(serde_json::json!({}))
        .map_err(|error| cstr!("Failed to send LSP initialized notification: {error}"))?;
    Ok(())
}

pub(super) fn request_lsp_document_diagnostics(
    client: &LspClient,
    uri: &Uri,
) -> Result<DocumentDiagnosticReportResult, String> {
    struct RawDocumentDiagnosticRequest;

    impl lsp_types::request::Request for RawDocumentDiagnosticRequest {
        type Params = serde_json::Value;
        type Result = DocumentDiagnosticReportResult;
        const METHOD: &'static str = "textDocument/diagnostic";
    }

    block_on(
        client.request::<RawDocumentDiagnosticRequest>(serde_json::json!({
            "textDocument": {
                "uri": uri,
            }
        })),
    )
    .map_err(|error| cstr!("{error}"))
}

/// Wait for a document diagnostic response without interpreting its payload.
///
/// Editor readiness only needs the response-backed transport ordering. The
/// native server's diagnostic payload can contain protocol extensions that are
/// irrelevant to that ordering and must not make a semantic query fail.
pub(super) fn request_lsp_document_diagnostic_ack(
    client: &LspClient,
    uri: &Uri,
) -> Result<(), String> {
    struct RawDocumentDiagnosticAckRequest;

    impl lsp_types::request::Request for RawDocumentDiagnosticAckRequest {
        type Params = serde_json::Value;
        type Result = Value;
        const METHOD: &'static str = "textDocument/diagnostic";
    }

    block_on(
        client.request::<RawDocumentDiagnosticAckRequest>(serde_json::json!({
            "textDocument": {
                "uri": uri,
            }
        })),
    )
    .map(|_| ())
    .map_err(|error| cstr!("{error}"))
}
