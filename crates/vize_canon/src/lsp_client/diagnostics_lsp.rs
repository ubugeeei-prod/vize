use crate::file_uri::path_to_file_uri;
use corsa::runtime::block_on;
use corsa_lsp::LspClient;
use lsp_types::{
    ClientCapabilities, DiagnosticClientCapabilities, DiagnosticWorkspaceClientCapabilities,
    DidChangeWatchedFilesClientCapabilities, DocumentDiagnosticReportResult, InitializeParams,
    InitializedParams, TextDocumentClientCapabilities, Uri, WorkDoneProgressParams,
    WorkspaceClientCapabilities, WorkspaceFolder,
};
use serde_json::Value;
use std::{path::Path, str::FromStr};
use vize_s0::{String, cstr};

pub(super) fn initialize_lsp_client(client: &LspClient, project_root: &Path) -> Result<(), String> {
    struct InitializeRequest;

    impl lsp_types::request::Request for InitializeRequest {
        type Params = InitializeParams;
        type Result = serde_json::Value;
        const METHOD: &'static str = "initialize";
    }

    struct InitializedNotification;

    impl lsp_types::notification::Notification for InitializedNotification {
        type Params = InitializedParams;
        const METHOD: &'static str = "initialized";
    }

    let root_uri = path_to_file_uri(project_root);
    let workspace_name = project_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace");
    block_on(client.request::<InitializeRequest>(initialize_lsp_params(
        project_root,
        root_uri,
        workspace_name,
    )?))
    .map_err(|error| cstr!("Failed to initialize Corsa LSP session: {error}"))?;
    client
        .notify::<InitializedNotification>(InitializedParams {})
        .map_err(|error| cstr!("Failed to send LSP initialized notification: {error}"))?;
    Ok(())
}

fn initialize_lsp_params(
    project_root: &Path,
    root_uri: String,
    workspace_name: &str,
) -> Result<InitializeParams, String> {
    let root_uri = Uri::from_str(root_uri.as_str())
        .map_err(|error| cstr!("Failed to build Corsa LSP root URI: {error}"))?;
    let capabilities = ClientCapabilities {
        text_document: Some(TextDocumentClientCapabilities {
            diagnostic: Some(DiagnosticClientCapabilities {
                dynamic_registration: Some(false),
                related_document_support: Some(true),
            }),
            ..Default::default()
        }),
        workspace: Some(WorkspaceClientCapabilities {
            did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
                dynamic_registration: Some(true),
                relative_pattern_support: Some(true),
            }),
            diagnostic: Some(DiagnosticWorkspaceClientCapabilities {
                refresh_support: Some(true),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    #[allow(deprecated)]
    Ok(InitializeParams {
        process_id: Some(std::process::id()),
        root_path: Some(project_root.to_string_lossy().into_owned()),
        root_uri: Some(root_uri.clone()),
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: root_uri,
            name: workspace_name.to_owned(),
        }]),
        capabilities,
        work_done_progress_params: WorkDoneProgressParams::default(),
        ..Default::default()
    })
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

#[cfg(all(test, feature = "native", unix))]
mod tests {
    use super::initialize_lsp_params;
    use std::{
        io::{BufReader, Write},
        os::unix::net::UnixStream,
        path::Path,
        thread,
        time::Duration,
    };

    use corsa::runtime::block_on;
    use corsa_lsp::jsonrpc::{
        JsonRpcConnection, JsonRpcConnectionOptions, RpcHandlerMap, read_frame,
    };
    use serde_json::json;

    #[test]
    fn initialize_advertises_client_side_file_watching() {
        let params = initialize_lsp_params(
            Path::new("/workspace"),
            "file:///workspace".into(),
            "workspace",
        )
        .unwrap();
        let params = serde_json::to_value(&params).unwrap();

        assert_eq!(
            params["capabilities"]["workspace"]["didChangeWatchedFiles"],
            json!({
                "dynamicRegistration": true,
                "relativePatternSupport": true,
            })
        );
    }

    #[test]
    fn malformed_readiness_response_fails_at_jsonrpc_frame_reader() {
        let (client_socket, mut server_socket) = UnixStream::pair().unwrap();
        let client = JsonRpcConnection::try_spawn_with_options(
            BufReader::new(client_socket.try_clone().unwrap()),
            client_socket,
            RpcHandlerMap::default(),
            JsonRpcConnectionOptions::new().with_request_timeout(Some(Duration::from_secs(5))),
        )
        .unwrap();
        let server_reader = server_socket.try_clone().unwrap();
        let server = thread::spawn(move || {
            let mut reader = BufReader::new(server_reader);
            let request_payload = read_frame(&mut reader).unwrap();
            let request: serde_json::Value = serde_json::from_slice(&request_payload).unwrap();
            assert_eq!(request["method"], json!("textDocument/diagnostic"));
            assert_eq!(
                request["params"]["textDocument"]["uri"],
                json!("file:///workspace/App.vue.ts")
            );

            let id = &request["id"];
            let body = format!(r#"{{"jsonrpc":"2.0","id":{id},"result":null}}{{}}"#);
            write!(
                server_socket,
                "Content-Length: {}\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
            server_socket.flush().unwrap();
        });

        let error = block_on(client.request_value(
            "textDocument/diagnostic",
            json!({
                "textDocument": {
                    "uri": "file:///workspace/App.vue.ts",
                },
            }),
        ))
        .unwrap_err();
        server.join().unwrap();
        let error = error.to_string();
        assert!(
            error.contains("trailing characters"),
            "malformed readiness response escaped the frame reader: {error}"
        );
    }
}
