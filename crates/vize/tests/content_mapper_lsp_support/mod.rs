#![allow(dead_code)]

use std::{path::Path, str::FromStr};

use corsa_lsp::LspClient;
use lsp_types::{FileChangeType, FileEvent, Uri};
use serde_json::{Value, json};
use vize_s0::String as CompactString;

mod component_oracles;
mod leak_assertions;
mod navigation;
mod package_install;
mod process_output;
pub mod raw_requests;
mod responder;
mod stop;
#[allow(unused_imports)]
pub use component_oracles::{
    assert_component_completions, assert_component_members, assert_component_navigation,
    assert_prop_navigation,
};
pub use leak_assertions::{assert_no_generated_uri, assert_no_generated_uri_or_zero_range};
#[allow(unused_imports)]
pub use navigation::{
    contains_location_range, contains_range, contains_text_edit, document_highlights, references,
    rename,
};
pub use package_install::install_packages;
#[allow(unused_imports)]
pub use process_output::output_text;
pub use responder::EditorResponder;
#[allow(unused_imports)]
pub use stop::StopOnDrop;

struct RawDocumentDiagnostic;
struct RawHover;
struct RawDefinition;
struct RawTypeDefinition;
struct RawCompletion;
struct RawCompletionResolve;

impl lsp_types::request::Request for RawDocumentDiagnostic {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "textDocument/diagnostic";
}

impl lsp_types::request::Request for RawHover {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "textDocument/hover";
}

impl lsp_types::request::Request for RawDefinition {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "textDocument/definition";
}

impl lsp_types::request::Request for RawTypeDefinition {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "textDocument/typeDefinition";
}

impl lsp_types::request::Request for RawCompletion {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "textDocument/completion";
}

impl lsp_types::request::Request for RawCompletionResolve {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "completionItem/resolve";
}

pub async fn hover(client: &LspClient, uri: &str, position: &Value) -> Value {
    client
        .request::<RawHover>(json!({ "textDocument": { "uri": uri }, "position": position }))
        .await
        .unwrap()
}

pub async fn definition(client: &LspClient, uri: &str, position: &Value) -> Value {
    client
        .request::<RawDefinition>(json!({ "textDocument": { "uri": uri }, "position": position }))
        .await
        .unwrap()
}

pub async fn type_definition(client: &LspClient, uri: &str, position: &Value) -> Value {
    client
        .request::<RawTypeDefinition>(json!({
            "textDocument": { "uri": uri },
            "position": position
        }))
        .await
        .unwrap()
}

pub async fn completion(client: &LspClient, uri: &str, position: &Value) -> Value {
    client
        .request::<RawCompletion>(json!({ "textDocument": { "uri": uri }, "position": position }))
        .await
        .unwrap()
}

pub async fn assert_completion(
    client: &LspClient,
    uri: &str,
    position: &Value,
    label: &str,
    resolved_fragments: &[&str],
) {
    let response = completion(client, uri, position).await;
    let items = response
        .get("items")
        .and_then(Value::as_array)
        .or_else(|| response.as_array())
        .unwrap_or_else(|| panic!("{response:#}"));
    let item = items
        .iter()
        .find(|item| item["label"] == label)
        .cloned()
        .unwrap_or_else(|| panic!("{response:#}"));
    let resolved = client.request::<RawCompletionResolve>(item).await.unwrap();
    assert_eq!(resolved["label"], label, "{resolved:#}");
    assert_no_generated_uri(&resolved);
    let resolved_text = serde_json::to_string(&resolved).unwrap();
    assert!(
        resolved_fragments
            .iter()
            .all(|fragment| resolved_text.contains(fragment)),
        "{resolved:#}"
    );
    assert!(!resolved_text.contains(".vue.ts"), "{resolved:#}");
}

pub async fn pull_diagnostics(client: &LspClient, uri: &str) -> Value {
    try_pull_diagnostics(client, uri).await.unwrap()
}

pub async fn try_pull_diagnostics(client: &LspClient, uri: &str) -> corsa_lsp::Result<Value> {
    client
        .request::<RawDocumentDiagnostic>(json!({ "textDocument": { "uri": uri } }))
        .await
}

pub fn notify_file_changes(client: &LspClient, changes: &[(&str, FileChangeType)]) {
    let changes = changes
        .iter()
        .map(|(uri, typ)| FileEvent {
            uri: Uri::from_str(uri).unwrap(),
            typ: *typ,
        })
        .collect();
    client
        .notify::<lsp_types::notification::DidChangeWatchedFiles>(
            lsp_types::DidChangeWatchedFilesParams { changes },
        )
        .unwrap();
}

pub fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

pub fn copy_fixture(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "node_modules" {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_fixture(&source_path, &destination_path);
        } else {
            std::fs::copy(source_path, destination_path).unwrap();
        }
    }
}

pub fn file_uri(path: &Path) -> CompactString {
    let mut uri = CompactString::from("file://");
    for byte in path.to_string_lossy().bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                uri.push(byte as char)
            }
            _ => {
                use std::fmt::Write;
                write!(uri, "%{byte:02X}").unwrap();
            }
        }
    }
    uri
}

pub fn position(source: &str, offset: usize) -> Value {
    let before = &source[..offset];
    let line = before.matches('\n').count();
    let character = before.rsplit_once('\n').map_or(before, |(_, tail)| tail);
    json!({ "line": line, "character": character.encode_utf16().count() })
}

pub fn contains_location(value: &Value, uri: &str, start: &Value) -> bool {
    match value {
        Value::Array(values) => values
            .iter()
            .any(|value| contains_location(value, uri, start)),
        Value::Object(object) => {
            let location_uri = object.get("uri").or_else(|| object.get("targetUri"));
            let range = object
                .get("range")
                .or_else(|| object.get("targetSelectionRange"));
            location_uri == Some(&Value::String(uri.to_owned()))
                && range.and_then(|range| range.get("start")) == Some(start)
        }
        _ => false,
    }
}

pub fn editor_capabilities() -> Value {
    json!({
        "workspace": {
            "configuration": true,
            "didChangeWatchedFiles": { "dynamicRegistration": true }
        },
        "textDocument": {
            "synchronization": { "dynamicRegistration": true },
            "diagnostic": { "dynamicRegistration": true },
            "completion": { "dynamicRegistration": true },
            "signatureHelp": { "dynamicRegistration": true },
            "hover": { "dynamicRegistration": true },
            "definition": { "dynamicRegistration": true },
            "typeDefinition": { "dynamicRegistration": true },
            "documentHighlight": { "dynamicRegistration": true },
            "references": { "dynamicRegistration": true },
            "rename": { "dynamicRegistration": true }
        }
    })
}
