use super::*;
use crate::api::ApiDialect;
use serde_json::json;

#[derive(Default)]
struct FullFs;

impl ApiFileSystem for FullFs {
    fn capabilities(&self) -> FileSystemCapabilities {
        FileSystemCapabilities {
            read_file: true,
            file_exists: true,
            directory_exists: true,
            get_accessible_entries: true,
            realpath: true,
        }
    }

    fn read_file(&self, path: &str) -> ReadFileResult {
        match path {
            "/found.ts" => ReadFileResult::Content("content".into()),
            "/missing.ts" => ReadFileResult::NotFound,
            _ => ReadFileResult::Fallback,
        }
    }

    fn file_exists(&self, path: &str) -> Option<bool> {
        Some(path == "/found.ts")
    }

    fn directory_exists(&self, path: &str) -> Option<bool> {
        Some(path == "/virtual")
    }

    fn get_accessible_entries(&self, path: &str) -> Option<DirectoryEntries> {
        (path == "/virtual").then(|| DirectoryEntries {
            files: ["a.ts", "b.ts"].into_iter().map(Into::into).collect(),
            directories: ["nested"].into_iter().map(Into::into).collect(),
        })
    }

    fn realpath(&self, path: &str) -> Option<CompactString> {
        Some(path.into())
    }
}

#[derive(Default)]
struct EmptyFs;

impl ApiFileSystem for EmptyFs {
    fn capabilities(&self) -> FileSystemCapabilities {
        FileSystemCapabilities::default()
    }
}

#[test]
fn callback_flag_is_rendered_once() {
    let flag = callback_flag(&FullFs).unwrap();
    assert_eq!(
        flag,
        "--callbacks=readFile,fileExists,directoryExists,getAccessibleEntries,realpath"
    );
}

#[test]
fn callback_flag_is_absent_without_capabilities() {
    assert_eq!(callback_flag(&EmptyFs), None);
    assert!(callback_names(&EmptyFs).is_empty());
}

/// Builds a host whose connection has completed the handshake as `dialect`,
/// or has not completed it at all for `None`.
fn host(fs: impl ApiFileSystem, dialect: Option<ApiDialect>) -> CallbackHost {
    let cell = DialectCell::default();
    if let Some(dialect) = dialect {
        cell.set(dialect);
    }
    CallbackHost::new(Arc::new(fs), Arc::new(cell))
}

fn session_host() -> CallbackHost {
    host(FullFs, Some(ApiDialect::SessionSnapshots))
}

fn derived_host() -> CallbackHost {
    host(FullFs, Some(ApiDialect::DerivedSnapshots))
}

#[test]
fn invoke_callback_covers_read_file_modes() {
    let host = session_host();
    assert_eq!(
        invoke_callback(&host, "readFile", &json!("/found.ts")).unwrap(),
        json!({ "content": "content" })
    );
    assert_eq!(
        invoke_callback(&host, "readFile", &json!("/missing.ts")).unwrap(),
        json!({ "content": Value::Null })
    );
    assert_eq!(
        invoke_callback(&host, "readFile", &json!("/fallback.ts")).unwrap(),
        Value::Null
    );
}

#[test]
fn invoke_callback_serializes_directory_entries_and_realpath() {
    let host = session_host();
    assert_eq!(
        invoke_callback(&host, "getAccessibleEntries", &json!("/virtual")).unwrap(),
        json!({ "files": ["a.ts", "b.ts"], "directories": ["nested"] })
    );
    assert_eq!(
        invoke_callback(&host, "realpath", &json!("/virtual/a.ts")).unwrap(),
        json!("/virtual/a.ts")
    );
}

#[test]
fn callbacks_answer_untagged_until_the_handshake_names_a_dialect() {
    let host = host(FullFs, None);
    assert_eq!(
        invoke_callback(&host, "fileExists", &json!("/found.ts")).unwrap(),
        json!(true)
    );
}

#[test]
fn derived_snapshots_dialect_tags_every_read_file_answer() {
    let host = derived_host();
    assert_eq!(
        invoke_callback(&host, "readFile", &json!("/found.ts")).unwrap(),
        json!({ "kind": "value", "value": "content" })
    );
    assert_eq!(
        invoke_callback(&host, "readFile", &json!("/missing.ts")).unwrap(),
        json!({ "kind": "missing" })
    );
    assert_eq!(
        invoke_callback(&host, "readFile", &json!("/fallback.ts")).unwrap(),
        json!({ "kind": "useOS" })
    );
}

#[test]
fn derived_snapshots_dialect_tags_existence_listing_and_realpath_answers() {
    let host = derived_host();
    assert_eq!(
        invoke_callback(&host, "fileExists", &json!("/other.ts")).unwrap(),
        json!({ "kind": "value", "value": false })
    );
    assert_eq!(
        invoke_callback(&host, "directoryExists", &json!("/virtual")).unwrap(),
        json!({ "kind": "value", "value": true })
    );
    assert_eq!(
        invoke_callback(&host, "getAccessibleEntries", &json!("/virtual")).unwrap(),
        json!({
            "kind": "value",
            "value": { "files": ["a.ts", "b.ts"], "directories": ["nested"] },
        })
    );
    assert_eq!(
        invoke_callback(&host, "getAccessibleEntries", &json!("/elsewhere")).unwrap(),
        json!({ "kind": "useOS" })
    );
    assert_eq!(
        invoke_callback(&host, "realpath", &json!("/virtual/a.ts")).unwrap(),
        json!({ "kind": "value", "value": "/virtual/a.ts" })
    );
}

#[test]
fn jsonrpc_handlers_only_expose_enabled_callbacks() {
    let handlers = jsonrpc_handlers(session_host());
    assert_eq!(handlers.len(), 5);
    assert!(handlers.contains_key("readFile"));
    assert!(handlers.contains_key("realpath"));
}

#[test]
fn unknown_callback_returns_jsonrpc_error() {
    let error = invoke_callback(&session_host(), "missing", &Value::Null).unwrap_err();
    assert_eq!(error.code, -32601);
}

#[test]
fn invalid_callback_payload_returns_invalid_params_error() {
    let error =
        invoke_callback(&session_host(), "readFile", &json!({ "path": "/found.ts" })).unwrap_err();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("expected a string path"));
}

#[test]
fn jsonrpc_handler_propagates_invalid_callback_params() {
    let handlers = jsonrpc_handlers(session_host());
    let handler = handlers.get("realpath").unwrap();
    let error = handler(json!(["/virtual/a.ts"])).unwrap_err();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("realpath"));
}
