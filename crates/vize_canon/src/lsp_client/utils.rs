use super::LspDiagnostic;
use lsp_types::Diagnostic;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use vize_carton::{cstr, FxHashMap, String};

pub(super) fn value_to_json<T>(value: T) -> Result<Value, String>
where
    T: Serialize,
{
    serde_json::to_value(value).map_err(|e| cstr!("Failed to encode JSON response: {e}"))
}

pub(super) fn convert_diagnostics(diagnostics: &[Diagnostic]) -> Vec<LspDiagnostic> {
    diagnostics
        .iter()
        .filter_map(|diagnostic| {
            serde_json::to_value(diagnostic)
                .ok()
                .and_then(|value| serde_json::from_value(value).ok())
        })
        .collect()
}

pub(super) fn remap_json_uris(value: &mut Value, mappings: &FxHashMap<String, String>) {
    match value {
        Value::Array(items) => {
            for item in items {
                remap_json_uris(item, mappings);
            }
        }
        Value::Object(object) => {
            let mut replacements = Vec::new();
            for (key, item) in object.iter_mut() {
                if let Some(mapped) = mappings.get(key.as_str()) {
                    replacements.push((key.clone(), mapped.clone()));
                }
                remap_json_uris(item, mappings);
            }
            for (old_key, new_key) in replacements {
                if let Some(value) = object.remove(old_key.as_str()) {
                    object.insert(new_key.as_str().into(), value);
                }
            }
        }
        Value::String(text) => {
            if let Some(mapped) = mappings.get(text.as_str()) {
                *text = mapped.as_str().into();
            }
        }
        _ => {}
    }
}

pub(super) fn remap_serialized_uris<T>(
    value: T,
    mappings: &FxHashMap<String, String>,
) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
{
    let mut value = serde_json::to_value(value)
        .map_err(|error| cstr!("Failed to encode Corsa result: {error}"))?;
    remap_json_uris(&mut value, mappings);
    serde_json::from_value(value).map_err(|error| cstr!("Failed to decode Corsa result: {error}"))
}
