//! TypeScript's own LSP over the Vize content mapper: answers an editor reads
//! from the mapper's spans alone, with no Vize editor in between.

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use corsa_lsp::{LspClient, LspSpawnConfig, VirtualDocument, jsonrpc::InboundEvent};
use lsp_types::Uri;
use serde_json::{Value, json};

use crate::content_mapper_lsp_support;
use content_mapper_lsp_support::raw_requests::{RawInitialize, RawInitialized};
use content_mapper_lsp_support::{
    EditorResponder, StopOnDrop, anchored_position, assert_location_range, definition,
    editor_capabilities, file_uri, hover, install_packages, pull_diagnostics, workspace_root,
};

mod component_tags;
mod template_bindings;

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";

/// One `tsc --lsp` session over a pinned differential fixture, with each
/// `.vue` file opened as the editor would.
struct Project {
    root: tempfile::TempDir,
    files: Vec<(String, String)>,
}

impl Project {
    /// Copy `tests/_fixtures/differential/typechecker/{name}`, dropping each
    /// file's `.txt` suffix.
    fn fixture(name: &str) -> Self {
        let fixture = workspace_root()
            .join("tests/_fixtures/differential/typechecker")
            .join(name);
        let cases = workspace_root().join("target/vize-tests/tests");
        std::fs::create_dir_all(&cases).unwrap();
        let root = tempfile::Builder::new()
            .prefix("content-mapper-editor-accuracy-")
            .tempdir_in(cases)
            .unwrap();
        let mut files = Vec::new();
        let mut pending = vec![fixture.clone()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    pending.push(path);
                    continue;
                }
                let relative = path.strip_prefix(&fixture).unwrap().to_string_lossy();
                let name = relative.strip_suffix(".txt").unwrap().replace('\\', "/");
                let text = std::fs::read_to_string(&path).unwrap();
                let target = root.path().join(&name);
                std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                std::fs::write(target, &text).unwrap();
                files.push((name, text));
            }
        }
        install_packages(root.path());
        Self { root, files }
    }

    fn uri(&self, name: &str) -> String {
        file_uri(&self.root.path().join(name)).to_string()
    }

    fn source(&self, name: &str) -> &str {
        self.files
            .iter()
            .find(|(file, _)| file == name)
            .map(|(_, text)| text.as_str())
            .unwrap_or_else(|| panic!("{name} is not part of the project"))
    }

    fn run(&self, tsgo: &Path, check: impl AsyncFnOnce(&LspClient)) {
        let root_uri = file_uri(self.root.path());
        let stop = AtomicBool::new(false);
        let editor = EditorResponder::default();
        std::thread::scope(|scope| {
            let _stop_on_drop = StopOnDrop(&stop);
            corsa::runtime::block_on(async {
                let client = LspClient::spawn(
                    LspSpawnConfig::new(tsgo)
                        .with_cwd(self.root.path())
                        .with_request_timeout(Some(Duration::from_secs(60))),
                )
                .await
                .unwrap();
                let responder_client = client.clone();
                let events = responder_client.subscribe();
                let (stop_ref, editor_ref) = (&stop, &editor);
                let responder = scope.spawn(move || {
                    while !stop_ref.load(Ordering::Relaxed) {
                        if let Ok(InboundEvent::Request { id, method, params }) =
                            events.recv_timeout(Duration::from_millis(50))
                        {
                            let result = editor_ref.respond_to(method.as_str(), &params);
                            let _ = responder_client.respond(id, result);
                        }
                    }
                });
                let initialize = client
                    .request::<RawInitialize>(json!({
                        "processId": std::process::id(),
                        "rootUri": root_uri,
                        "workspaceFolders": [{ "uri": root_uri, "name": "editor-accuracy" }],
                        "capabilities": editor_capabilities(),
                        "initializationOptions": { "runExternalCode": true }
                    }))
                    .await
                    .unwrap();
                assert!(initialize["capabilities"].is_object(), "{initialize:#}");
                client.notify::<RawInitialized>(json!({})).unwrap();
                let overlay = client.overlay();
                for (name, text) in self.files.iter().filter(|(name, _)| name.ends_with(".vue")) {
                    let uri = Uri::from_str(&self.uri(name)).unwrap();
                    overlay
                        .open(VirtualDocument::new(uri, "vue", text.as_str()))
                        .unwrap();
                }
                check(&client).await;
                stop.store(true, Ordering::Relaxed);
                client.graceful_close().await.unwrap();
                responder.join().unwrap();
            });
        });
    }
}

fn tsgo() -> Option<PathBuf> {
    let tsgo = std::env::var_os(TSGO_ENV).map(PathBuf::from);
    if tsgo.is_none() {
        eprintln!("skipping TypeScript LSP editor accuracy: {TSGO_ENV} is not set");
    }
    tsgo
}

fn hover_text(hover: &Value) -> &str {
    hover["contents"]["value"].as_str().unwrap_or_default()
}

/// Error codes of a pulled diagnostic report.
async fn error_codes(client: &LspClient, uri: &str) -> Vec<Value> {
    let report = pull_diagnostics(client, uri).await;
    report["items"]
        .as_array()
        .unwrap_or_else(|| panic!("{report:#}"))
        .iter()
        .map(|item| item["code"].clone())
        .collect()
}

async fn assert_definition_at(
    client: &LspClient,
    (uri, source): (&str, &str),
    anchor: &str,
    target_uri: &str,
    (start, end): (Value, Value),
) {
    let at = anchored_position(source, anchor, anchor);
    let response = definition(client, uri, &at).await;
    assert_location_range(&response, target_uri, &start, &end, anchor);
}
