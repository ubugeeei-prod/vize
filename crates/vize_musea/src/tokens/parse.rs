use std::{fs, path::Path};

use serde_json::{Map, Value};
use vize_carton::{FxHashMap, String, ToCompactString};

use super::types::{DesignToken, TokenCategory, TokenError, TokenResult};

#[allow(clippy::disallowed_types)]
type JsonObject = Map<std::string::String, Value>;

pub fn parse_tokens_from_path(path: impl AsRef<Path>) -> TokenResult<Vec<TokenCategory>> {
    let path = path.as_ref();
    let metadata = fs::metadata(path).map_err(io_error)?;
    if metadata.is_dir() {
        let mut root = Value::Object(Map::new());
        merge_token_directory(&mut root, path)?;
        return Ok(parse_tokens_from_value(&root));
    }

    let bytes = fs::read(path).map_err(io_error)?;
    let value = serde_json::from_slice::<Value>(&bytes).map_err(json_error)?;
    Ok(parse_tokens_from_value(&value))
}

pub fn parse_tokens_from_json(source: &str) -> TokenResult<Vec<TokenCategory>> {
    let value = serde_json::from_str::<Value>(source).map_err(json_error)?;
    Ok(parse_tokens_from_value(&value))
}

pub fn parse_tokens_from_value(value: &Value) -> Vec<TokenCategory> {
    let Some(object) = value.as_object() else {
        return Vec::new();
    };
    flatten_token_tree(object)
}

fn merge_token_directory(target: &mut Value, dir: &Path) -> TokenResult<()> {
    let mut entries = fs::read_dir(dir)
        .map_err(io_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(io_error)?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let metadata = entry.metadata().map_err(io_error)?;
        if metadata.is_dir() {
            merge_token_directory(target, &path)?;
            continue;
        }
        if !metadata.is_file() || !is_token_file(&path) {
            continue;
        }

        let bytes = fs::read(&path).map_err(io_error)?;
        let value = serde_json::from_slice::<Value>(&bytes).map_err(json_error)?;
        deep_merge_token_trees(target, value);
    }

    Ok(())
}

fn deep_merge_token_trees(target: &mut Value, source: Value) {
    if is_token_leaf(target) || is_token_leaf(&source) {
        *target = source;
        return;
    }

    let Value::Object(source_obj) = source else {
        *target = source;
        return;
    };
    let Some(target_obj) = target.as_object_mut() else {
        *target = Value::Object(source_obj);
        return;
    };

    for (key, value) in source_obj {
        if let Some(existing) = target_obj.get_mut(&key) {
            deep_merge_token_trees(existing, value);
        } else {
            target_obj.insert(key, value);
        }
    }
}

fn flatten_token_tree(object: &JsonObject) -> Vec<TokenCategory> {
    let mut categories = Vec::new();
    for (key, value) in object {
        if is_token_leaf(value) {
            continue;
        }
        let Some(child) = value.as_object() else {
            continue;
        };

        let tokens = extract_tokens(child);
        let subcategories = flatten_token_tree(child);
        if !tokens.is_empty() || !subcategories.is_empty() {
            categories.push(TokenCategory {
                name: format_category_name(key),
                tokens,
                subcategories,
            });
        }
    }
    categories
}

fn extract_tokens(object: &JsonObject) -> FxHashMap<String, DesignToken> {
    let mut tokens = FxHashMap::default();
    for (key, value) in object {
        if let Some(token) = normalize_token(value) {
            tokens.insert(key.as_str().into(), token);
        }
    }
    tokens
}

fn normalize_token(value: &Value) -> Option<DesignToken> {
    let object = value.as_object()?;
    let raw_value = object.get("value").or_else(|| object.get("$value"))?;
    if !raw_value.is_string() && !raw_value.is_number() {
        return None;
    }

    let tier = object
        .get("$tier")
        .and_then(Value::as_str)
        .filter(|tier| *tier == "primitive" || *tier == "semantic")
        .map(Into::into);

    Some(DesignToken {
        value: raw_value.clone(),
        token_type: object
            .get("type")
            .or_else(|| object.get("$type"))
            .and_then(Value::as_str)
            .map(Into::into),
        description: object
            .get("description")
            .and_then(Value::as_str)
            .map(Into::into),
        attributes: object.get("attributes").cloned(),
        tier,
        reference: object
            .get("$reference")
            .and_then(Value::as_str)
            .map(Into::into),
        resolved_value: None,
    })
}

pub(super) fn is_token_leaf(value: &Value) -> bool {
    value.as_object().is_some_and(|object| {
        object
            .get("value")
            .or_else(|| object.get("$value"))
            .is_some_and(|value| value.is_string() || value.is_number())
    })
}

fn is_token_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".json") || name.ends_with(".tokens.json"))
}

fn format_category_name(name: &str) -> String {
    let mut result = String::new("");
    let mut word_started = false;
    let mut prev_lowercase = false;

    for ch in name.chars() {
        if ch == '-' || ch == '_' || ch.is_whitespace() {
            word_started = false;
            prev_lowercase = false;
            continue;
        }
        if prev_lowercase && ch.is_ascii_uppercase() {
            word_started = false;
        }
        if !word_started {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push(ch.to_ascii_uppercase());
            word_started = true;
        } else {
            result.push(ch.to_ascii_lowercase());
        }
        prev_lowercase = ch.is_ascii_lowercase();
    }

    if result.is_empty() {
        name.to_compact_string()
    } else {
        result
    }
}

fn io_error(error: std::io::Error) -> TokenError {
    TokenError::Io {
        message: error.to_compact_string(),
    }
}

fn json_error(error: serde_json::Error) -> TokenError {
    TokenError::Json {
        message: error.to_compact_string(),
    }
}
