//! Facts from the actual snapshot's complete native source inventory.
use super::{hash, write};
use corsa::{api::ProjectSession, runtime::block_on};
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_carton::String;

pub(super) fn sources(session: &ProjectSession, output: &Path) -> Vec<String> {
    let value = block_on(session.client().raw_json_request(
        "getSourceFileNames",
        json!({"snapshot":session.snapshot().handle,"project":session.project().id}),
    ))
    .unwrap();
    write(&output.join("source-file-names.json"), &value);
    let names: Vec<String> = serde_json::from_value(value).unwrap();
    let rows = names
        .iter()
        .map(|name| {
            let actual = Path::new(name.as_str()).canonicalize().unwrap();
            let bytes = fs::read(&actual).unwrap();
            json!({"nativeName":name,"actualPath":actual,"bytes":bytes.len(),"sha256":hash(&bytes)})
        })
        .collect::<Vec<_>>();
    write(&output.join("source-input-manifest.json"), &json!(rows));
    names
}
pub(super) fn diagnostics(
    session: &ProjectSession,
    version: &str,
    method: &str,
    files: &[String],
    output: &Path,
) -> Vec<Value> {
    let mut packets = Vec::new();
    let mut diagnostics = Vec::new();
    if version == "7.1.0-beta" {
        let params = json!({"snapshot":session.snapshot().handle,"project":session.project().id,"files":files});
        let response = block_on(session.client().raw_json_request(method, params.clone())).unwrap();
        if !response.is_null() {
            diagnostics.extend(serde_json::from_value::<Vec<Value>>(response.clone()).unwrap());
        }
        packets.push(json!({"request":params,"response":response}));
    } else {
        assert_eq!(version, "7.0.2");
        for file in files {
            let params = json!({"snapshot":session.snapshot().handle,"project":session.project().id,"file":file});
            let response =
                block_on(session.client().raw_json_request(method, params.clone())).unwrap();
            if !response.is_null() {
                diagnostics.extend(serde_json::from_value::<Vec<Value>>(response.clone()).unwrap());
            }
            packets.push(json!({"request":params,"response":response}));
        }
    }
    write(&output.join(format!("{method}.json")), &json!(packets));
    diagnostics
}
pub(super) fn declaration_path(symbol: &corsa::api::SymbolResponse) -> std::path::PathBuf {
    let declaration = symbol
        .declarations
        .first()
        .unwrap()
        .declaring_path()
        .unwrap();
    Path::new(declaration.as_str()).canonicalize().unwrap()
}
pub(super) fn package(symbol: &corsa::api::SymbolResponse, provider: &Path) -> Value {
    let declaration = declaration_path(symbol);
    let relative = declaration
        .strip_prefix(provider)
        .expect("the actual declaration must belong to this provider");
    assert_ne!(relative, Path::new(""));
    for parent in declaration.ancestors().skip(1) {
        let manifest = parent.join("package.json");
        if !manifest.is_file() {
            continue;
        }
        let bytes = fs::read(&manifest).unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        if value.get("name").and_then(Value::as_str) == Some("@vue/runtime-core") {
            return json!({"root":parent,"manifest":value,"sha256":hash(&bytes),"declaration":declaration});
        }
    }
    panic!("actual selected provider declaration has no runtime-core package owner")
}
