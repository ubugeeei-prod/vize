use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use corsa::jsonrpc::InboundEvent;
use corsa::lsp::{LspClient, LspSpawnConfig, VirtualDocument};
use lsp_types::Uri;
use serde_json::{Value, json};

mod content_mapper_lsp_support;
use content_mapper_lsp_support::{
    contains_location, copy_fixture, editor_capabilities, file_uri, install_packages, position,
    workspace_root,
};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";

struct StopOnDrop<'a>(&'a AtomicBool);

impl Drop for StopOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

struct RawInitialize;
struct RawDiscoverContentMappers;
struct RawCompletion;
struct RawSignatureHelp;
struct RawDocumentDiagnostic;
struct RawHover;
struct RawDefinition;
struct RawReferences;
struct RawRename;

macro_rules! raw_request {
    ($request:ty, $method:literal) => {
        impl lsp_types::request::Request for $request {
            type Params = Value;
            type Result = Value;
            const METHOD: &'static str = $method;
        }
    };
}

raw_request!(RawInitialize, "initialize");
raw_request!(RawDiscoverContentMappers, "custom/discoverContentMappers");
raw_request!(RawCompletion, "textDocument/completion");
raw_request!(RawSignatureHelp, "textDocument/signatureHelp");
raw_request!(RawDocumentDiagnostic, "textDocument/diagnostic");
raw_request!(RawHover, "textDocument/hover");
raw_request!(RawDefinition, "textDocument/definition");
raw_request!(RawReferences, "textDocument/references");
raw_request!(RawRename, "textDocument/rename");

struct RawInitialized;

impl lsp_types::notification::Notification for RawInitialized {
    type Params = Value;
    const METHOD: &'static str = "initialized";
}

#[test]
fn standard_tsgo_lsp_maps_core_symbol_features_to_authored_vue() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper LSP conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-lsp-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_packages(project.path());

    let child_path = project.path().join("src/Child.vue");
    let source = std::fs::read_to_string(&child_path).unwrap();
    let child_uri = file_uri(&child_path);
    let root_uri = file_uri(project.path());
    let symbol_offset = source.rfind("count.toFixed").unwrap();
    let symbol_position = position(&source, symbol_offset + 1);
    let completion_position = position(&source, symbol_offset + "count.".len());
    let signature_position = position(&source, symbol_offset + "count.toFixed(".len());
    let declaration_offset = source.find("count: number").unwrap();
    let declaration_position = position(&source, declaration_offset);

    let stop = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let _stop_on_drop = StopOnDrop(&stop);
        corsa::runtime::block_on(async {
            let client = LspClient::spawn(
                LspSpawnConfig::new(&tsgo)
                    .with_cwd(project.path())
                    .with_request_timeout(Some(Duration::from_secs(30))),
            )
            .await
            .unwrap();
            let responder_client = client.clone();
            let events = responder_client.subscribe();
            let stop_ref = &stop;
            let responder = scope.spawn(move || {
                while !stop_ref.load(Ordering::Relaxed) {
                    if let Ok(InboundEvent::Request { id, method, params }) =
                        events.recv_timeout(Duration::from_millis(50))
                    {
                        let result = if method.as_str() == "workspace/configuration" {
                            let count = params
                                .get("items")
                                .and_then(Value::as_array)
                                .map_or(0, Vec::len);
                            Value::Array(vec![Value::Null; count])
                        } else {
                            Value::Null
                        };
                        let _ = responder_client.respond(id, result);
                    }
                }
            });
            let initialize = client
                .request::<RawInitialize>(json!({
                    "processId": std::process::id(),
                    "rootUri": root_uri,
                    "workspaceFolders": [{ "uri": root_uri, "name": "content-mapper-lsp" }],
                    "capabilities": editor_capabilities(),
                    "initializationOptions": { "loadExternalPlugins": true }
                }))
                .await
                .unwrap();
            assert!(initialize["capabilities"].is_object(), "{initialize:#}");
            client.notify::<RawInitialized>(json!({})).unwrap();

            let discovered = client
                .request::<RawDiscoverContentMappers>(json!({
                    "textDocuments": [{ "uri": child_uri }],
                    "extensions": [".vue"]
                }))
                .await
                .unwrap();
            assert_eq!(discovered["extensions"], json!([".vue"]), "{discovered:#}");

            let uri = Uri::from_str(&child_uri).unwrap();
            let overlay = client.overlay();
            overlay
                .open(VirtualDocument::new(uri.clone(), "vue", source.as_str()))
                .unwrap();
            let params =
                json!({ "textDocument": { "uri": child_uri }, "position": symbol_position });

            let completion = client
                .request::<RawCompletion>(json!({
                    "textDocument": { "uri": child_uri },
                    "position": completion_position
                }))
                .await
                .unwrap();
            assert!(
                serde_json::to_string(&completion)
                    .unwrap()
                    .contains("toFixed"),
                "{completion:#}"
            );

            let signature = client
                .request::<RawSignatureHelp>(json!({
                    "textDocument": { "uri": child_uri },
                    "position": signature_position
                }))
                .await
                .unwrap();
            let signature_text = serde_json::to_string(&signature).unwrap();
            assert!(
                signature_text.contains("fractionDigits") && signature_text.contains("number"),
                "{signature:#}"
            );

            let hover = client.request::<RawHover>(params.clone()).await.unwrap();
            let hover_text = serde_json::to_string(&hover).unwrap();
            assert!(
                hover_text.contains("count") && hover_text.contains("number"),
                "{hover:#}"
            );

            let definition = client
                .request::<RawDefinition>(params.clone())
                .await
                .unwrap();
            let definition_text = serde_json::to_string(&definition).unwrap();
            assert!(
                definition_text.contains(child_uri.as_str()),
                "{definition:#}"
            );
            assert!(
                contains_location(&definition, &child_uri, &declaration_position),
                "{definition:#}"
            );
            assert!(!definition_text.contains(".vue.ts"), "{definition:#}");

            let references = client
                .request::<RawReferences>(json!({
                    "textDocument": { "uri": child_uri },
                    "position": symbol_position,
                    "context": { "includeDeclaration": true }
                }))
                .await
                .unwrap();
            let references_text = serde_json::to_string(&references).unwrap();
            assert!(
                references_text.matches(child_uri.as_str()).count() >= 2,
                "{references:#}"
            );
            assert!(!references_text.contains(".vue.ts"), "{references:#}");

            let rename = client
                .request::<RawRename>(json!({
                    "textDocument": { "uri": child_uri },
                    "position": symbol_position,
                    "newName": "renamedCount"
                }))
                .await
                .unwrap();
            let rename_text = serde_json::to_string(&rename).unwrap();
            assert!(rename_text.contains(child_uri.as_str()), "{rename:#}");
            assert!(
                rename_text.matches("renamedCount").count() >= 2,
                "{rename:#}"
            );
            assert!(!rename_text.contains(".vue.ts"), "{rename:#}");

            let diagnostic_params = json!({ "textDocument": { "uri": child_uri } });
            let clean = client
                .request::<RawDocumentDiagnostic>(diagnostic_params.clone())
                .await
                .unwrap();
            assert_eq!(clean["items"], json!([]), "{clean:#}");

            let broken_source = source.replace("count.toFixed(0)", "count.missing()");
            overlay.replace(&uri, broken_source.as_str()).unwrap();
            let broken = client
                .request::<RawDocumentDiagnostic>(diagnostic_params.clone())
                .await
                .unwrap();
            let broken_text = serde_json::to_string(&broken).unwrap();
            assert!(
                broken_text.contains("2339") && broken_text.contains("missing"),
                "{broken:#}"
            );
            let missing = position(&broken_source, broken_source.find("missing").unwrap());
            assert_eq!(broken["items"][0]["range"]["start"], missing, "{broken:#}");

            overlay.replace(&uri, source.as_str()).unwrap();
            let repaired = client
                .request::<RawDocumentDiagnostic>(diagnostic_params)
                .await
                .unwrap();
            assert_eq!(repaired["items"], json!([]), "{repaired:#}");

            overlay.replace(&uri, broken_source.as_str()).unwrap();
            let dirty = client
                .request::<RawDocumentDiagnostic>(json!({
                    "textDocument": { "uri": child_uri }
                }))
                .await
                .unwrap();
            assert!(!dirty["items"].as_array().unwrap().is_empty(), "{dirty:#}");

            assert!(overlay.close(&uri).unwrap().is_some());
            let closed = client
                .request::<RawDocumentDiagnostic>(json!({
                    "textDocument": { "uri": child_uri }
                }))
                .await
                .unwrap();
            assert_eq!(closed["items"], json!([]), "{closed:#}");
            stop.store(true, Ordering::Relaxed);
            client.close().await.unwrap();
            responder.join().unwrap();
        });
    });
}
