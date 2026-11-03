//! A real framed SDK peer records the normal provenance read's complete route.
#![expect(clippy::disallowed_macros, reason = "framed protocol fixtures")]

use crate::{corsa_bridge::DiagnosticPhase, lsp_client::CorsaProjectClient};
use corsa::{
    api::{ApiClient, ProjectSession},
    jsonrpc::{RawMessage, read_frame, write_frame},
    runtime::block_on,
};
use serde_json::{Value, json};
use std::{
    io::{BufReader, BufWriter},
    os::unix::net::UnixListener,
    thread,
};

const URI: &str = "file:///workspace/a%20%23.ts";
const FILE: &str = "/workspace/a #.ts";
const SNAPSHOT: &str = "n0000000000000001";
const PROJECT: &str = "p./workspace/tsconfig.json";

struct Peer {
    _root: tempfile::TempDir,
    api: ApiClient,
    thread: thread::JoinHandle<Vec<Value>>,
}

impl Peer {
    fn start() -> Self {
        Self::with_unsupported(None)
    }

    fn with_unsupported(unsupported: Option<&'static str>) -> Self {
        let root = tempfile::tempdir().unwrap();
        let pipe = root.path().join("normal-read.sock");
        let listener = UnixListener::bind(&pipe).unwrap();
        let fixture: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/native-diagnostic-phase-11626/framed-input.json"
        )))
        .unwrap();
        let worker = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(10)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut observed = Vec::new();
            while let Ok(bytes) = read_frame(&mut reader) {
                let message: RawMessage = serde_json::from_slice(&bytes).unwrap();
                let method = message.method.as_deref().unwrap();
                let params = message.params.clone().unwrap_or(Value::Null);
                observed.push(serde_json::to_value(&message).unwrap());
                if unsupported == Some(method) {
                    let response = json!({"jsonrpc":"2.0","id":message.id,
                        "error":{"code":-32601,"message":"method not found"}});
                    write_frame(&mut writer, &serde_json::to_vec(&response).unwrap()).unwrap();
                    continue;
                }
                let response = match method {
                    "initialize" => {
                        json!({"useCaseSensitiveFileNames":true,"currentDirectory":"/workspace"})
                    }
                    "updateSnapshot" => json!({"snapshot":SNAPSHOT,"projects":[{
                        "id":PROJECT,"configFileName":"/workspace/tsconfig.json",
                        "compilerOptions":{},"rootFiles":[FILE]
                    }],"changes":{"changedProjects":{},"removedProjects":[]}}),
                    "getDiagnosticsForFile" | "getDiagnosticsForProject" => {
                        let response = json!({"jsonrpc":"2.0","id":message.id,
                            "error":{"code":-32601,"message":"method not found"}});
                        write_frame(&mut writer, &serde_json::to_vec(&response).unwrap()).unwrap();
                        continue;
                    }
                    "getSyntacticDiagnostics" | "getSuggestionDiagnostics" => json!([]),
                    "getSemanticDiagnostics" => fixture["warmSemantic"].clone(),
                    "getSourceFile" => json!({"data":fixture["encodedSnapshotSource"]}),
                    "release" => Value::Null,
                    _ => panic!("unexpected normal provenance RPC {method}"),
                };
                if method.starts_with("get") {
                    assert_eq!(params["snapshot"], SNAPSHOT);
                    assert_eq!(params["project"], PROJECT);
                    if method != "getDiagnosticsForProject" {
                        assert_eq!(params["file"], FILE);
                    }
                }
                write_frame(
                    &mut writer,
                    &serde_json::to_vec(&RawMessage::response(message.id.unwrap(), response))
                        .unwrap(),
                )
                .unwrap();
                if method == "release" {
                    break;
                }
            }
            observed
        });
        Self {
            _root: root,
            api: block_on(ApiClient::connect_pipe(&pipe)).unwrap(),
            thread: worker,
        }
    }

    fn client(&self) -> CorsaProjectClient {
        let session = block_on(ProjectSession::open(
            self.api.clone(),
            "/workspace/tsconfig.json",
            None,
        ))
        .unwrap();
        let mut client = CorsaProjectClient::empty_for_test("/workspace".into());
        *client.project_session_for_test() = Some(session);
        client.session_document_uris.insert(URI.into(), URI.into());
        client
    }

    fn finish(self, client: &mut CorsaProjectClient) -> Vec<Value> {
        let session = client.project_session_for_test().take().unwrap();
        block_on(session.snapshot().release()).unwrap();
        drop(session);
        block_on(self.api.close()).unwrap();
        self.thread.join().unwrap()
    }
}

#[test]
fn warm_read_view_borrows_retained_sources_and_project_metadata() {
    let peer = Peer::start();
    let mut client = peer.client();
    client.document_texts.insert(
        "file:///workspace/other.ts".into(),
        "export const large = 'retained';\n".repeat(2048).into(),
    );
    client.external_document_uris.insert(
        "file:///workspace/large-mirror.ts".into(),
        "file:///workspace/large-source.vue".into(),
    );
    client.session_document_uris.insert(URI.into(), URI.into());
    let read = client.borrow_diagnostics_read_view(URI, URI).unwrap();
    // Currentness requires pointer identity for both maps and the project.
    assert!(client.borrowed_diagnostics_read_is_current(&read));
    drop(read);
    let observed = peer.finish(&mut client);
    assert_eq!(
        observed
            .iter()
            .map(|row| row["method"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["initialize", "updateSnapshot", "release"],
    );
}

#[test]
fn warm_provenance_read_retains_phase_from_one_current_snapshot_and_exact_rpc_order() {
    let peer = Peer::start();
    let mut client = peer.client();
    let result = client.request_diagnostics_full_with_provenance(URI);
    let observed = peer.finish(&mut client);
    let fetched =
        result.expect("compiler-kind endpoints must own the warm result before LSP fallback");
    assert!(!fetched.used_cache);
    assert_eq!(fetched.diagnostics.len(), 1);
    assert_eq!(fetched.diagnostics[0].phase, DiagnosticPhase::Semantic);
    assert_eq!(client.diagnostics[URI][0].phase, DiagnosticPhase::Semantic);
    let methods = observed
        .iter()
        .map(|row| row["method"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        methods,
        [
            "initialize",
            "updateSnapshot",
            "getDiagnosticsForFile",
            "getDiagnosticsForProject",
            "getSourceFile",
            "getSyntacticDiagnostics",
            "getSemanticDiagnostics",
            "getSuggestionDiagnostics",
            "release"
        ]
    );
}

#[test]
fn a_rejected_current_api_owner_never_probes_compiler_kinds() {
    let peer = Peer::start();
    let mut client = peer.client();
    // This open logical world has no admitted API overlay. A phase read must
    // leave the retained snapshot unused just like every other semantic query.
    client.logical_workspace_overlays = true;
    client.materialized_project_session = true;
    client
        .document_texts
        .insert(URI.into(), "export {};".into());
    assert!(!client.can_use_api_for_uri(URI));
    let result = client.request_diagnostics_full_with_provenance(URI);
    let observed = peer.finish(&mut client);
    assert!(
        result.is_err(),
        "the mock deliberately provides no fallback LSP runtime"
    );
    assert!(client.diagnostics.is_empty());
    assert_eq!(
        observed
            .iter()
            .map(|row| row["method"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["initialize", "updateSnapshot", "release"]
    );
}

#[test]
fn an_unsupported_kind_cannot_publish_a_partial_warm_result_or_reprobe_the_api() {
    let peer = Peer::with_unsupported(Some("getSemanticDiagnostics"));
    let mut client = peer.client();
    let result = client.request_diagnostics_full_with_provenance(URI);
    let observed = peer.finish(&mut client);
    assert!(
        result.is_err(),
        "only the deliberately unavailable fallback may answer now"
    );
    assert!(client.diagnostics.is_empty());
    assert_eq!(
        observed
            .iter()
            .map(|row| row["method"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "initialize",
            "updateSnapshot",
            "getDiagnosticsForFile",
            "getDiagnosticsForProject",
            "getSourceFile",
            "getSyntacticDiagnostics",
            "getSemanticDiagnostics",
            "release",
        ]
    );
}
