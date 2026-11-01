//! Prospective two-context experiment; no production pool or isolated editor world.

use super::{control, only_owner};
use crate::lsp_client::{editor_lsp::EditorLspSession, session::uri_document_identifier};
use corsa::{
    api::{ApiClient, ApiDialect, ManagedSnapshot, ProjectHandle, ProjectSession},
    runtime::block_on,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path, process::Command};
use vize_l0::{FxHashMap, String, cstr};

const A_BAD: &str = "export const answer: number = 'wrong';\nanswer.toFixed();\n";
const A_GOOD: &str = "export const answer: number = 42;\nanswer.toFixed();\n";
const A_NULL: &str = "export const answer: number = null;\nanswer.toFixed();\n";
const B_NULL: &str = "export const label: string = null;\nlabel.toUpperCase();\n";
const A_NEXT: &str = "export const answer: boolean = true;\nanswer.valueOf();\n";
const B_GOOD: &str = "export const label: string = 'clean';\nlabel.toUpperCase();\n";
const B_BAD: &str = "export const label: string = 42;\nlabel.toUpperCase();\n";

fn record(name: &str, value: &Value) {
    let directory = std::env::var_os("VIZE_PRIVATE_SHARED_CONTEXT_CAPTURE")
        .expect("the functional Actions run must retain whole packets");
    let directory = Path::new(&directory);
    fs::create_dir_all(directory).unwrap();
    let path = directory.join(cstr!("{name}.json").as_str());
    assert!(!path.exists(), "never replace earlier raw observations");
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn runtime(version: &str, image: &str) -> std::path::PathBuf {
    assert!(
        control::enabled(),
        "this ignored test requires actual native execution"
    );
    let native = control::runtime();
    let digest = Sha256::digest(fs::read(&native).unwrap())
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect::<String>();
    let output = Command::new(&native).arg("--version").output().unwrap();
    record(
        "provider",
        &json!({"path":native,"sha256":digest,"versionCommand":{
            "argv":["--version"],"status":output.status.code(),
            "stdout":output.stdout,"stderr":output.stderr
        }}),
    );
    assert_eq!(digest, image);
    assert!(output.status.success());
    assert_eq!(output.stdout, cstr!("Version {version}\n").as_bytes());
    assert!(output.stderr.is_empty());
    native
}

fn config(root: &Path, name: &str, strict: bool, source: &str) -> std::path::PathBuf {
    let directory = root.join(name);
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("source.ts"), source).unwrap();
    let config = directory.join("tsconfig.json");
    fs::write(
        &config,
        serde_json::to_vec(&json!({
            "compilerOptions":{"strict":strict,"noEmit":true,"types":[],
                "target":"ESNext","module":"ESNext","moduleResolution":"Bundler"},
            "files":["source.ts"]
        }))
        .unwrap(),
    )
    .unwrap();
    config
}

/// Only disjoint URI ownership is admitted by this prototype, even for equal text.
fn union(
    a: Option<(&str, &str)>,
    b: Option<(&str, &str)>,
) -> Result<FxHashMap<String, String>, ()> {
    let mut documents = FxHashMap::default();
    for (uri, text) in [a, b].into_iter().flatten() {
        if documents.insert(uri.into(), text.into()).is_some() {
            return Err(());
        }
    }
    Ok(documents)
}

fn acknowledge(editor: &mut EditorLspSession, documents: &FxHashMap<String, String>) {
    editor.synchronize(documents).unwrap();
    let mut uris = documents.keys().collect::<Vec<_>>();
    uris.sort_unstable();
    for uri in uris {
        editor.ready_document_uri(uri).unwrap();
    }
}

fn open(api: &ApiClient, config: &Path, uri: &str) -> ProjectSession {
    let session = block_on(ProjectSession::open(
        api.fork_snapshot_lineage().unwrap(),
        config.to_str().unwrap(),
        Some(uri_document_identifier(uri)),
    ))
    .unwrap();
    assert_eq!(
        Path::new(session.project().config_file_name.as_str()),
        config
    );
    let parsed = block_on(api.parse_config_file(uri_document_identifier(
        &crate::file_uri::path_to_file_uri(config),
    )))
    .unwrap();
    assert_eq!(session.project().compiler_options, parsed.options);
    assert!(session.project().root_files.iter().any(|file| {
        Path::new(file.as_str()) == crate::file_uri::file_uri_to_path(uri).unwrap()
    }));
    session
}

fn held_type(
    api: &ApiClient,
    snapshot: &ManagedSnapshot,
    project: &ProjectHandle,
    uri: &str,
) -> String {
    let ty = block_on(api.get_type_at_position(
        snapshot.handle.clone(),
        project.clone(),
        uri_document_identifier(uri),
        14,
    ))
    .unwrap()
    .unwrap();
    block_on(api.type_to_string(snapshot.handle.clone(), project.clone(), ty.id, None, None))
        .unwrap()
        .into()
}

fn packet(
    editor: &mut EditorLspSession,
    name: &str,
    uri: &str,
    config: &Path,
    completion_column: u32,
) -> Value {
    let observed = editor.diagnosing_configuration(uri).map_err(|error| match error {
        crate::lsp_client::editor_lsp::project_identity::ProjectIdentityError::Communication(error) => error,
        crate::lsp_client::editor_lsp::project_identity::ProjectIdentityError::Unconfigured => "unconfigured".into(),
    });
    record(
        cstr!("{name}-editor-project").as_str(),
        &json!({"uri":uri,"expectedConfig":config,"observed":observed}),
    );
    assert_eq!(observed.unwrap(), config);
    let diagnostics = editor.diagnostics(uri);
    record(cstr!("{name}-diagnostics").as_str(), &json!(diagnostics));
    let diagnostics = diagnostics.unwrap();
    let hover = editor.hover(uri, 1, 2);
    record(cstr!("{name}-hover").as_str(), &json!(hover));
    let hover = hover.unwrap();
    let completion = editor.completion(uri, 1, completion_column);
    record(cstr!("{name}-completion").as_str(), &json!(completion));
    let completion = completion.unwrap();
    let value = json!({"uri":uri,"diagnostics":diagnostics,"hover":hover,"completion":completion});
    record(name, &value);
    value
}

fn expected_error(end: u32, message: &str) -> Value {
    json!({"kind":"full","items":[{
        "range":{"start":{"line":0,"character":13},"end":{"line":0,"character":end}},
        "severity":1,"code":2322,"source":"ts","message":message
    }]})
}

fn assert_diagnostics(packet: &Value, expected: Value) {
    assert_eq!(packet["diagnostics"], expected);
    assert!(!packet["hover"].is_null());
    assert!(!packet["completion"].is_null());
}

// This is the exact two-key defaults materialization owned by issued.rs::record.
// The original native response remains byte-for-byte retained in packet().
fn completion_item(packet: &Value, label: &str) -> Value {
    let response = &packet["completion"];
    let (items, defaults) = match response {
        Value::Array(items) => (items, None),
        Value::Object(list) => (
            list["items"].as_array().unwrap(),
            list.get("itemDefaults").and_then(Value::as_object),
        ),
        _ => panic!("native completion response must contain actual items"),
    };
    let raw = items.iter().find(|item| item["label"] == label).unwrap();
    let mut item = raw.as_object().unwrap().clone();
    if let Some(defaults) = defaults {
        for key in ["data", "insertTextFormat"] {
            if let Some(value) = defaults.get(key) {
                item.entry(key).or_insert_with(|| value.clone());
            }
        }
    }
    assert!(item.get("data").is_some_and(|value| !value.is_null()));
    Value::Object(item)
}

fn assert_unchanged_semantic_packet(current: &Value, previous: &Value) {
    // Native opaque completion data can change with native program generation.
    // Preserve it whole and use the actual issuance/resolve gate instead of equality.
    assert_eq!(current["uri"], previous["uri"]);
    assert_eq!(current["diagnostics"], previous["diagnostics"]);
    assert_eq!(current["hover"], previous["hover"]);
}

fn clean() -> Value {
    json!({"kind":"full","items":[]})
}

mod tests;
