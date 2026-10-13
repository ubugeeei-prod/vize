use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

pub fn project(config_file_name: &str) -> Value {
    json!({
        "id": "p./workspace/tsconfig.json",
        "configFileName": config_file_name,
        "compilerOptions": { "strict": true, "module": "esnext" },
        "rootFiles": ["/workspace/src/index.ts"],
    })
}

pub fn snapshot_from_update_params(config_file_name: &str, params: &Value) -> Value {
    let changed_files = extract_changed_files(params);
    snapshot_with_changed_files(
        config_file_name,
        if changed_files.is_empty() {
            vec!["/workspace/src/index.ts".to_owned()]
        } else {
            changed_files
        },
    )
}

fn snapshot_with_changed_files(config_file_name: &str, changed_files: Vec<String>) -> Value {
    json!({
        "snapshot": "n0000000000000001",
        "projects": [project(config_file_name)],
        "changes": {
            "changedProjects": {
                "p./workspace/tsconfig.json": {
                    "changedFiles": changed_files,
                    "deletedFiles": []
                }
            },
            "removedProjects": []
        }
    })
}

pub fn symbol(name: &str) -> Value {
    json!({
        "id": "s0000000000000001",
        "name": name,
        "flags": 2,
        "checkFlags": 0,
        "declarations": ["1.3.80./workspace/src/index.ts"],
        "valueDeclaration": "1.3.80./workspace/src/index.ts",
    })
}

/// A symbol response with a caller-chosen handle and name.
pub fn named_symbol(id: &str, name: &str) -> Value {
    json!({
        "id": id,
        "name": name,
        "flags": 1,
        "checkFlags": 0,
        "declarations": ["1.3.80./workspace/src/index.ts"],
        "valueDeclaration": "1.3.80./workspace/src/index.ts",
    })
}

/// Every `TypeFlags` bit the client guards a type-relation request on, so one
/// probe type can drive the whole traversal surface.
const ALL_TRAVERSAL_TYPE_FLAGS: u32 = 0b1_1111_1111 << 20;
/// `classOrInterface | reference | mapped`.
const ALL_TRAVERSAL_OBJECT_FLAGS: u32 = 0b11 | (1 << 2) | (1 << 5);

pub fn type_response(id: &str) -> Value {
    let (flags, object_flags) = if all_traversal_flags_enabled() {
        (ALL_TRAVERSAL_TYPE_FLAGS, ALL_TRAVERSAL_OBJECT_FLAGS)
    } else {
        (262144, 16)
    };
    json!({
        "id": id,
        "flags": flags,
        "objectFlags": object_flags,
        "symbol": "s0000000000000001",
        "texts": ["type-text"],
    })
}

/// Whether the mock should report a type that satisfies every client-side
/// traversal guard.
///
/// Clients skip most type-relation requests unless the type carries the
/// matching `TypeFlags` bit, so protocol-contract tests would otherwise only
/// ever observe a handful of endpoints. `CORSA_MOCK_ALL_TYPE_FLAGS` lets such a
/// test drive the full surface from a single type.
fn all_traversal_flags_enabled() -> bool {
    std::env::var_os("CORSA_MOCK_ALL_TYPE_FLAGS").is_some()
}

pub fn signature() -> Value {
    json!({
        "id": "g0000000000000001",
        "flags": 1,
        "declaration": "1.3.80./workspace/src/index.ts",
        "typeParameters": ["t0000000000000002"],
        "parameters": ["s0000000000000001"],
        "thisParameter": "s0000000000000002",
    })
}

pub fn type_predicate() -> Value {
    json!({
        "kind": 1,
        "parameterIndex": 0,
        "parameterName": "value",
        "type": type_response("t0000000000000003"),
    })
}

pub fn index_info() -> Value {
    json!({
        "keyType": type_response("t0000000000000004"),
        "valueType": type_response("t0000000000000005"),
        "isReadonly": true,
    })
}

pub fn encoded(bytes: &[u8]) -> Value {
    json!({ "data": STANDARD.encode(bytes) })
}

pub fn capabilities() -> Value {
    json!({
        "overlay": {
            "updateSnapshotOverlayChanges": true
        },
        "diagnostics": {
            "snapshot": true,
            "project": true,
            "file": true
        },
        "editor": {
            "hover": true,
            "definition": true,
            "references": true,
            "rename": true,
            "completion": true
        }
    })
}

pub fn file_diagnostics(file: Value) -> Value {
    json!({
        "file": file,
        "syntactic": [diagnostic("TS1005", "expected ';'")],
        "semantic": [diagnostic("TS2322", "type mismatch")],
        "suggestion": [diagnostic("TS80006", "convert to shorthand")]
    })
}

pub fn project_diagnostics(file: Value) -> Value {
    json!({
        "project": "p./workspace/tsconfig.json",
        "files": [file_diagnostics(file)]
    })
}

pub fn snapshot_diagnostics(file: Value) -> Value {
    json!({
        "snapshot": "n0000000000000001",
        "projects": [project_diagnostics(file)]
    })
}

pub fn hover() -> Value {
    json!({
        "contents": {
            "kind": "markdown",
            "value": "`value`: string"
        },
        "range": range()
    })
}

pub fn definition() -> Value {
    json!([location()])
}

pub fn references() -> Value {
    json!([location(), secondary_location()])
}

pub fn rename(new_name: &str) -> Value {
    json!({
        "changes": {
            "file:///workspace/src/index.ts": [
                {
                    "range": range(),
                    "newText": new_name
                }
            ]
        }
    })
}

pub fn completion() -> Value {
    json!({
        "isIncomplete": false,
        "items": [
            {
                "label": "value",
                "kind": 6,
                "detail": "const value: string"
            }
        ]
    })
}

fn extract_changed_files(params: &Value) -> Vec<String> {
    let mut files = Vec::new();
    if let Some(file_changes) = params.get("fileChanges") {
        push_documents(
            &mut files,
            file_changes
                .get("changed")
                .and_then(Value::as_array)
                .into_iter()
                .flatten(),
        );
        push_documents(
            &mut files,
            file_changes
                .get("created")
                .and_then(Value::as_array)
                .into_iter()
                .flatten(),
        );
        push_documents(
            &mut files,
            file_changes
                .get("deleted")
                .and_then(Value::as_array)
                .into_iter()
                .flatten(),
        );
    }
    if let Some(overlay_changes) = params.get("overlayChanges") {
        push_documents(
            &mut files,
            overlay_changes
                .get("upsert")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.get("document")),
        );
        push_documents(
            &mut files,
            overlay_changes
                .get("delete")
                .and_then(Value::as_array)
                .into_iter()
                .flatten(),
        );
    }
    files
}

fn push_documents<'a>(files: &mut Vec<String>, documents: impl IntoIterator<Item = &'a Value>) {
    for document in documents {
        if let Some(path) = document.as_str() {
            files.push(path.to_owned());
        } else if let Some(uri) = document.get("uri").and_then(Value::as_str) {
            files.push(uri.to_owned());
        }
    }
}

fn diagnostic(code: &str, message: &str) -> Value {
    json!({
        "range": range(),
        "severity": 1,
        "code": code,
        "source": "mock-corsa",
        "message": message
    })
}

fn location() -> Value {
    json!({
        "uri": "file:///workspace/src/index.ts",
        "range": range()
    })
}

fn secondary_location() -> Value {
    json!({
        "uri": "file:///workspace/src/other.ts",
        "range": range()
    })
}

fn range() -> Value {
    json!({
        "start": { "line": 0, "character": 0 },
        "end": { "line": 0, "character": 5 }
    })
}

/// Handle-carrying request fields that upstream can only resolve inside a
/// project, in the order upstream validates them.
const PROJECT_SCOPED_HANDLE_FIELDS: [(&str, &str); 3] = [
    ("type", "type"),
    ("symbol", "symbol"),
    ("signature", "signature"),
];

/// Rejects requests that reference an object handle without naming the project
/// that issued it.
///
/// TypeScript 7 stable runtimes resolve type, symbol, and signature handles per
/// project and answer project-less lookups with
/// `empty project ID for <kind> handle <n>`. The mock enforces the same
/// contract so a binding that forgets to forward the project handle fails in
/// the normal test suite instead of only against a real runtime.
///
/// Every regression in this family — issues #384, #389, #390, #392, #393, #395,
/// #410, #413, #416, #418, #427, and #440 — reached a release because the mock
/// answered project-less requests that upstream rejects.
pub fn missing_project_message(params: &Value) -> Option<String> {
    let params = params.as_object()?;
    if params
        .get("project")
        .and_then(Value::as_str)
        .is_some_and(|project| !project.trim().is_empty())
    {
        return None;
    }
    PROJECT_SCOPED_HANDLE_FIELDS
        .iter()
        .find_map(|(field, kind)| {
            let handle = handle_text(params.get(*field)?)?;
            Some(format!("empty project ID for {kind} handle {handle}"))
        })
}

fn handle_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) if !text.trim().is_empty() => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}
