use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use corsa::jsonrpc::InboundEvent;
use corsa::lsp::{LspClient, LspSpawnConfig, VirtualDocument};
use lsp_types::Uri;
use serde_json::{Value, json};

#[allow(dead_code)]
mod content_mapper_lsp_support;
use content_mapper_lsp_support::{
    assert_completion, assert_prop_navigation, copy_fixture, editor_capabilities, file_uri,
    install_packages, position, pull_diagnostics, workspace_root,
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
struct RawInitialized;

impl lsp_types::request::Request for RawInitialize {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "initialize";
}

impl lsp_types::request::Request for RawDiscoverContentMappers {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "custom/discoverContentMappers";
}

impl lsp_types::notification::Notification for RawInitialized {
    type Params = Value;
    const METHOD: &'static str = "initialized";
}

#[test]
fn standard_tsgo_lsp_maps_call_signature_and_runtime_events() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper LSP conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-lsp-event-forms-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_packages(project.path());

    let app_path = project.path().join("src/App.vue");
    let app_source = std::fs::read_to_string(&app_path).unwrap();
    let app_uri = file_uri(&app_path);
    let cases = [
        (
            "CallSignatureChild.vue",
            "@submit",
            "\"submit\"",
            "submit",
            "boolean",
        ),
        ("RuntimeChild.vue", "@cancel", "cancel:", "cancel", "string"),
    ]
    .map(|(file, usage, declaration, name, ty)| {
        let path = project.path().join("src").join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        let uri = file_uri(&path);
        let usage_position = position(&app_source, app_source.find(usage).unwrap() + 1);
        let declaration_position = position(&source, source.find(declaration).unwrap());
        (uri, source, usage_position, declaration_position, name, ty)
    });
    let root_uri = file_uri(project.path());

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
                    "workspaceFolders": [{ "uri": root_uri, "name": "event-forms" }],
                    "capabilities": editor_capabilities(),
                    "initializationOptions": { "loadExternalPlugins": true }
                }))
                .await
                .unwrap();
            assert!(initialize["capabilities"].is_object(), "{initialize:#}");
            client.notify::<RawInitialized>(json!({})).unwrap();

            let discovered = client
                .request::<RawDiscoverContentMappers>(json!({
                    "textDocuments": cases.iter().map(|case| json!({ "uri": case.0 })).collect::<Vec<_>>(),
                    "extensions": [".vue"]
                }))
                .await
                .unwrap();
            assert_eq!(discovered["extensions"], json!([".vue"]), "{discovered:#}");

            let overlay = client.overlay();
            let app_document_uri = Uri::from_str(&app_uri).unwrap();
            overlay
                .open(VirtualDocument::new(
                    app_document_uri.clone(),
                    "vue",
                    app_source.as_str(),
                ))
                .unwrap();
            for (uri, source, usage, declaration, name, ty) in &cases {
                overlay
                    .open(VirtualDocument::new(
                        Uri::from_str(uri).unwrap(),
                        "vue",
                        source.as_str(),
                    ))
                    .unwrap();
                assert_prop_navigation(&client, &app_uri, usage, name, ty, uri, declaration).await;
                let clean = pull_diagnostics(&client, uri).await;
                assert_eq!(clean["items"], json!([]), "{clean:#}");
            }

            let partial = app_source
                .replace("@submit", "@sub")
                .replace("@cancel", "@can");
            overlay
                .replace(&app_document_uri, partial.as_str())
                .unwrap();
            for (usage, label, payload) in
                [("@sub", "submit", "boolean"), ("@can", "cancel", "string")]
            {
                let completion_position =
                    position(&partial, partial.find(usage).unwrap() + usage.len());
                assert_completion(&client, &app_uri, &completion_position, label, &[payload]).await;
            }

            stop.store(true, Ordering::Relaxed);
            client.close().await.unwrap();
            responder.join().unwrap();
        });
    });
}
