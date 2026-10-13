use super::*;
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

#[test]
fn invoke_callback_covers_read_file_modes() {
    assert_eq!(
        invoke_callback(&FullFs, "readFile", &json!("/found.ts")).unwrap(),
        json!({ "content": "content" })
    );
    assert_eq!(
        invoke_callback(&FullFs, "readFile", &json!("/missing.ts")).unwrap(),
        json!({ "content": Value::Null })
    );
    assert_eq!(
        invoke_callback(&FullFs, "readFile", &json!("/fallback.ts")).unwrap(),
        Value::Null
    );
}

#[test]
fn invoke_callback_serializes_directory_entries_and_realpath() {
    assert_eq!(
        invoke_callback(&FullFs, "getAccessibleEntries", &json!("/virtual")).unwrap(),
        json!({ "files": ["a.ts", "b.ts"], "directories": ["nested"] })
    );
    assert_eq!(
        invoke_callback(&FullFs, "realpath", &json!("/virtual/a.ts")).unwrap(),
        json!("/virtual/a.ts")
    );
}

#[test]
fn jsonrpc_handlers_only_expose_enabled_callbacks() {
    let handlers = jsonrpc_handlers(Arc::new(FullFs));
    assert_eq!(handlers.len(), 5);
    assert!(handlers.contains_key("readFile"));
    assert!(handlers.contains_key("realpath"));
}

#[test]
fn unknown_callback_returns_jsonrpc_error() {
    let error = invoke_callback(&FullFs, "missing", &Value::Null).unwrap_err();
    assert_eq!(error.code, -32601);
}

#[test]
fn invalid_callback_payload_returns_invalid_params_error() {
    let error = invoke_callback(&FullFs, "readFile", &json!({ "path": "/found.ts" })).unwrap_err();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("expected a string path"));
}

#[test]
fn jsonrpc_handler_propagates_invalid_callback_params() {
    let handlers = jsonrpc_handlers(Arc::new(FullFs));
    let handler = handlers.get("realpath").unwrap();
    let error = handler(json!(["/virtual/a.ts"])).unwrap_err();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("realpath"));
}
