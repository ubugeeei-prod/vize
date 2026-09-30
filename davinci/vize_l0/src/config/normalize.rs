//! Public JavaScript-facing config normalization.
//!
//! The Rust CLI deserializes config into strongly typed structs, but the npm
//! `vize/config` API intentionally behaves like a lightweight loader: unknown
//! keys are preserved, `entries` can model monorepos, and legacy aliases are
//! normalized without schema validation. This module keeps that public shape in
//! Rust so JS callers do not need to reimplement the merge and alias rules.

use serde_json::{Map, Value};

/// Normalize a raw user config value into the public `ResolvedVizeConfig` shape.
///
/// The operation mirrors the historical TypeScript API:
///
/// - `null` values are removed recursively.
/// - A top-level array becomes `entries`, with global entries merged into the
///   resolved root object.
/// - A top-level object with `entries` becomes one root entry plus explicit
///   entries, unless the root entry is empty.
/// - The legacy `lsp` key is rewritten to `languageServer`, while an explicit
///   `languageServer` value wins when both are present.
///
/// The function deliberately does not validate against the generated schema.
/// Config consumers may carry package-local keys through this API, so unknown
/// object members are retained.
pub fn normalize_public_config_value(value: Value) -> Result<Value, String> {
    match strip_nullish(value) {
        Some(Value::Array(entries)) => normalize_config_entries(entries),
        Some(Value::Object(config)) => normalize_config_object(config),
        Some(_) | None => normalize_config_object(Map::new()),
    }
}

fn normalize_config_object(mut config: Map<String, Value>) -> Result<Value, String> {
    normalize_config_aliases(&mut config);

    let raw_entries = config.remove("entries");
    let root_entry = config;
    let mut entries = Vec::new();

    if !root_entry.is_empty() {
        entries.push(Value::Object(root_entry.clone()));
    }

    match raw_entries {
        Some(Value::Array(raw_entries)) => {
            for entry in raw_entries {
                entries.push(normalize_entry(entry)?);
            }
        }
        Some(Value::Null) | None => {}
        Some(_) => return Err("config.entries must be an array when provided".into()),
    }

    let mut resolved = root_entry;
    resolved.insert("entries".into(), Value::Array(entries));
    Ok(Value::Object(resolved))
}

fn normalize_config_entries(raw_entries: Vec<Value>) -> Result<Value, String> {
    let mut entries = Vec::with_capacity(raw_entries.len());
    for entry in raw_entries {
        entries.push(normalize_entry(entry)?);
    }

    let mut global_config = Map::new();
    for entry in &entries {
        if let Value::Object(entry) = entry
            && is_global_config_entry(entry)
        {
            deep_merge(&mut global_config, strip_entry_metadata(entry));
        }
    }

    global_config.insert("entries".into(), Value::Array(entries));
    Ok(Value::Object(global_config))
}

fn normalize_entry(entry: Value) -> Result<Value, String> {
    match entry {
        Value::Object(mut entry) => {
            normalize_config_aliases(&mut entry);
            Ok(Value::Object(entry))
        }
        Value::Null => Ok(Value::Object(Map::new())),
        _ => Err("config entries must be objects".into()),
    }
}

fn normalize_config_aliases(config: &mut Map<String, Value>) {
    let Some(lsp) = config.remove("lsp") else {
        return;
    };

    if !config.contains_key("languageServer") {
        config.insert("languageServer".into(), lsp);
    }
}

fn strip_nullish(value: Value) -> Option<Value> {
    match value {
        Value::Null => None,
        Value::Array(values) => Some(Value::Array(
            values.into_iter().filter_map(strip_nullish).collect(),
        )),
        Value::Object(values) => {
            let values = values
                .into_iter()
                .filter_map(|(key, value)| strip_nullish(value).map(|value| (key, value)))
                .collect();
            Some(Value::Object(values))
        }
        value => Some(value),
    }
}

fn is_global_config_entry(entry: &Map<String, Value>) -> bool {
    !entry.contains_key("basePath")
        && !entry.contains_key("files")
        && !entry.contains_key("ignores")
}

fn strip_entry_metadata(entry: &Map<String, Value>) -> Map<String, Value> {
    entry
        .iter()
        .filter(|(key, _)| {
            !matches!(
                key.as_str(),
                "name" | "basePath" | "files" | "ignores" | "extends"
            )
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn deep_merge(target: &mut Map<String, Value>, source: Map<String, Value>) {
    for (key, value) in source {
        match (target.get_mut(&key), value) {
            (Some(Value::Object(target_object)), Value::Object(source_object)) => {
                deep_merge(target_object, source_object);
            }
            (_, value) => {
                target.insert(key, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::normalize_public_config_value;

    #[test]
    fn empty_object_produces_empty_entries() {
        assert_eq!(
            normalize_public_config_value(json!({})).unwrap(),
            json!({ "entries": [] })
        );
    }

    #[test]
    fn nullish_values_are_removed_recursively() {
        assert_eq!(
            normalize_public_config_value(json!({
                "formatter": { "printWidth": 5, "useTabs": null },
                "linter": null
            }))
            .unwrap(),
            json!({
                "formatter": { "printWidth": 5 },
                "entries": [{ "formatter": { "printWidth": 5 } }]
            })
        );
    }

    #[test]
    fn lsp_alias_is_normalized() {
        assert_eq!(
            normalize_public_config_value(json!({ "lsp": { "enabled": true } })).unwrap(),
            json!({
                "languageServer": { "enabled": true },
                "entries": [{ "languageServer": { "enabled": true } }]
            })
        );
    }

    #[test]
    fn array_entries_merge_global_config_into_root() {
        assert_eq!(
            normalize_public_config_value(json!([
                { "formatter": { "printWidth": 50 }, "linter": { "enabled": true } },
                { "name": "scoped", "files": ["src/**"], "formatter": { "printWidth": 80 } }
            ]))
            .unwrap(),
            json!({
                "formatter": { "printWidth": 50 },
                "linter": { "enabled": true },
                "entries": [
                    { "formatter": { "printWidth": 50 }, "linter": { "enabled": true } },
                    { "name": "scoped", "files": ["src/**"], "formatter": { "printWidth": 80 } }
                ]
            })
        );
    }
}
