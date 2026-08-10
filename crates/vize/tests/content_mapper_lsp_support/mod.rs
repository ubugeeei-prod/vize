use std::path::Path;
use std::str::FromStr;

use corsa::lsp::LspClient;
use lsp_types::{FileChangeType, FileEvent, Uri};
use serde_json::{Value, json};
use vize_carton::String as CompactString;

struct RawDocumentDiagnostic;

impl lsp_types::request::Request for RawDocumentDiagnostic {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "textDocument/diagnostic";
}

pub async fn pull_diagnostics(client: &LspClient, uri: &str) -> Value {
    try_pull_diagnostics(client, uri).await.unwrap()
}

pub async fn try_pull_diagnostics(client: &LspClient, uri: &str) -> corsa::Result<Value> {
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

pub fn install_packages(project_root: &Path) {
    let mapper_root = project_root.join("node_modules/vize");
    std::fs::create_dir_all(&mapper_root).unwrap();
    std::fs::write(
        mapper_root.join("package.json"),
        serde_json::to_vec_pretty(&json!({
            "name": "vize",
            "private": true,
            "tsContentMapper": {
                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                "compilerOptions": ["noUnusedLocals"],
            },
        }))
        .unwrap(),
    )
    .unwrap();

    let store = workspace_root().join("node_modules/.pnpm");
    let mut candidates = std::fs::read_dir(&store)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("vue@3."))
        .map(|entry| entry.path().join("node_modules/vue"))
        .filter(|path| path.join("package.json").is_file())
        .collect::<Vec<_>>();
    candidates.sort();
    let source = candidates.pop().expect("workspace Vue package");
    let target = project_root.join("node_modules/vue");
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
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
            "references": { "dynamicRegistration": true },
            "rename": { "dynamicRegistration": true }
        }
    })
}
