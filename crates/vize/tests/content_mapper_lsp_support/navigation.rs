use corsa_lsp::LspClient;
use serde_json::{Value, json};

use super::leak_assertions::assert_no_generated_uri;

pub struct RawReferences;
pub struct RawDocumentHighlight;
pub struct RawRename;

impl lsp_types::request::Request for RawReferences {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "textDocument/references";
}

impl lsp_types::request::Request for RawRename {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "textDocument/rename";
}

impl lsp_types::request::Request for RawDocumentHighlight {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "textDocument/documentHighlight";
}

pub async fn references(client: &LspClient, uri: &str, position: &Value) -> Value {
    client
        .request::<RawReferences>(json!({
            "textDocument": { "uri": uri },
            "position": position,
            "context": { "includeDeclaration": true }
        }))
        .await
        .unwrap()
}

pub async fn document_highlights(client: &LspClient, uri: &str, position: &Value) -> Value {
    client
        .request::<RawDocumentHighlight>(json!({
            "textDocument": { "uri": uri },
            "position": position
        }))
        .await
        .unwrap()
}

pub async fn rename(client: &LspClient, uri: &str, position: &Value, new_name: &str) -> Value {
    client
        .request::<RawRename>(json!({
            "textDocument": { "uri": uri },
            "position": position,
            "newName": new_name
        }))
        .await
        .unwrap()
}

pub fn contains_location_range(value: &Value, uri: &str, start: &Value, end: &Value) -> bool {
    match value {
        Value::Array(values) => values
            .iter()
            .any(|value| contains_location_range(value, uri, start, end)),
        Value::Object(object) => {
            object.get("uri") == Some(&Value::String(uri.to_owned()))
                && object.get("range").and_then(|range| range.get("start")) == Some(start)
                && object.get("range").and_then(|range| range.get("end")) == Some(end)
        }
        _ => false,
    }
}

pub fn contains_range(value: &Value, start: &Value, end: &Value) -> bool {
    match value {
        Value::Array(values) => values.iter().any(|value| contains_range(value, start, end)),
        Value::Object(object) => {
            object.get("range").and_then(|range| range.get("start")) == Some(start)
                && object.get("range").and_then(|range| range.get("end")) == Some(end)
        }
        _ => false,
    }
}

pub fn contains_text_edit(
    value: &Value,
    uri: &str,
    start: &Value,
    end: &Value,
    new_name: &str,
) -> bool {
    let matches = |edit: &Value| {
        edit["range"]["start"] == *start
            && edit["range"]["end"] == *end
            && edit["newText"].as_str() == Some(new_name)
    };
    value["changes"][uri]
        .as_array()
        .is_some_and(|edits| edits.iter().any(matches))
        || value["documentChanges"].as_array().is_some_and(|changes| {
            changes.iter().any(|change| {
                change["textDocument"]["uri"] == uri
                    && change["edits"]
                        .as_array()
                        .is_some_and(|edits| edits.iter().any(matches))
            })
        })
}

pub fn assert_location_range(
    response: &Value,
    uri: &str,
    target_start: &Value,
    target_end: &Value,
    label: &str,
) {
    assert!(
        contains_location_range(response, uri, target_start, target_end),
        "{label} should map to the authored source range:\n{response:#}"
    );
    assert_no_generated_uri(response);
}

pub fn assert_hover(
    response: &Value,
    expected_start: &Value,
    expected_end: &Value,
    expected_contents: Value,
    label: &str,
) {
    assert_no_generated_uri(response);
    assert_eq!(
        response["contents"], expected_contents,
        "{label} should expose the expected hover contents:\n{response:#}"
    );
    assert_eq!(
        response["range"],
        json!({
            "start": expected_start,
            "end": expected_end,
        }),
        "{label} range should cover the consumer symbol:\n{response:#}"
    );
}
