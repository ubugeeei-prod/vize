//! Native port of `projectViteConfigSource` (vite-runtime.mjs) for literal sources.
//!
//! It answers only when the embedded JS projection would return without
//! throwing. `None` sends the config back to Node, which also owns every
//! error message, so a refused setting surfaces exactly as before.

use std::sync::OnceLock;

use serde_json::{Map, Value};

type Object = Map<std::string::String, Value>;

/// Top-level Vite keys the JS projection reads besides `vue`. Node keeps them.
const NODE_OWNED: [&str; 5] = ["compiler", "typecheck", "lint", "fmt", "root"];
/// Retired native sections an explicitly selected config may not carry.
const EXPLICIT_RETIRED: [&str; 7] = [
    "linter",
    "formatter",
    "languageServer",
    "globalTypes",
    "files",
    "ignores",
    "entries",
];
const ACTION_ALIASES: [(&str, &str); 3] = [
    ("compile", "compiler"),
    ("typeCheck", "typeChecker"),
    ("fmt", "formatter"),
];
const TYPE_CHECKER_PATHS: [&str; 5] = [
    "tsconfig",
    "corsaPath",
    "tsgoPath",
    "tsrsPath",
    "globalsFile",
];

/// Project one evaluated Vite source the way `projectViteConfigSource` does.
pub(super) fn project(source: &Object, explicit: bool, root: &str) -> Option<Value> {
    if explicit && EXPLICIT_RETIRED.iter().any(|key| source.contains_key(*key)) {
        return None;
    }
    if rejects_legacy(source) || NODE_OWNED.iter().any(|key| source.contains_key(*key)) {
        return None;
    }
    let mut config = Object::new();
    let mut type_checker = Object::new();
    type_checker.insert("jsxTypecheck".into(), Value::Bool(true));
    config.insert("typeChecker".into(), Value::Object(type_checker));
    if let Some(vue) = source.get("vue") {
        let schema = schema()?;
        if check(vue, schema, schema)? {
            overlay(&mut config, flat(normalize(vue.as_object()?)?)?);
        } else {
            return None;
        }
    }
    let mut config = project_paths(config, root, false)?;
    config.insert("__vizeProjectRoot".into(), Value::String(root.into()));
    Some(Value::Object(config))
}

/// `rejectLegacyConfig`: retired inheritance and `vize:` sections throw.
fn rejects_legacy(source: &Object) -> bool {
    let nested_vize = |value: &Value| {
        value
            .as_object()
            .is_some_and(|object| object.contains_key("vize"))
    };
    source
        .get("extends")
        .is_some_and(|value| !value.is_boolean())
        || source.contains_key("vize")
        || ["lint", "fmt", "pack"]
            .iter()
            .filter_map(|key| source.get(*key))
            .any(|value| match value {
                Value::Array(values) => values.iter().any(nested_vize),
                value => nested_vize(value),
            })
}

/// `normalizeVueActionAliases`, declining alias conflicts and Vite+ `lint`.
fn normalize(value: &Object) -> Option<Object> {
    let conflict = ACTION_ALIASES
        .iter()
        .chain(&[("lint", "linter")])
        .any(|(action, native)| value.contains_key(*action) && value.contains_key(*native));
    // Vite+ lint projection stays in JS; Node also reports its conflicts.
    if conflict || value.contains_key("lint") {
        return None;
    }
    let mut result = value.clone();
    for (action, native) in ACTION_ALIASES {
        if let Some(section) = result.remove(action) {
            result.insert(native.into(), section);
        }
    }
    if let Some(entries) = result.get_mut("entries") {
        let Value::Array(entries) = entries else {
            return None;
        };
        for entry in entries {
            *entry = Value::Object(normalize(entry.as_object()?)?);
        }
    }
    Some(result)
}

/// `flatVueOptions`: `version` moves under `vue`, entries flatten recursively.
fn flat(mut selected: Object) -> Option<Object> {
    let version = selected.remove("version");
    let entries = selected.remove("entries");
    if let Some(version) = version {
        let mut vue = Object::new();
        vue.insert("version".into(), version);
        selected.insert("vue".into(), Value::Object(vue));
    }
    if let Some(entries) = entries {
        let Value::Array(entries) = entries else {
            return None;
        };
        let entries = entries
            .into_iter()
            .map(|entry| match entry {
                Value::Object(entry) => flat(entry).map(Value::Object),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        selected.insert("entries".into(), Value::Array(entries));
    }
    Some(selected)
}

/// `overlayVueOptions`: objects merge, everything else replaces.
fn overlay(base: &mut Object, own: Object) {
    for (key, value) in own {
        match (base.get_mut(&key), value) {
            (Some(Value::Object(previous)), Value::Object(value)) => overlay(previous, value),
            (_, value) => {
                base.insert(key, value);
            }
        }
    }
}

/// `projectPaths`: resolve authored paths against the config directory.
fn project_paths(mut config: Object, root: &str, entry: bool) -> Option<Object> {
    let base_path = match config.get("basePath") {
        None | Some(Value::Null) => None,
        Some(Value::String(path)) => Some(path.as_str()),
        // `path.resolve` throws on any other type.
        Some(_) => return None,
    };
    if base_path.is_some()
        || config.contains_key("files")
        || (entry && config.contains_key("ignores"))
    {
        let resolved = resolve(root, base_path.unwrap_or("."));
        config.insert("basePath".into(), Value::String(resolved));
    }
    if let Some(Value::Array(entries)) = config.get_mut("entries") {
        for entry in entries {
            if let Value::Object(object) = entry {
                *object = project_paths(std::mem::take(object), root, true)?;
            }
        }
    }
    if let Some(Value::Object(type_checker)) = config.get_mut("typeChecker") {
        for key in TYPE_CHECKER_PATHS {
            if let Some(Value::String(path)) = type_checker.get_mut(key) {
                *path = resolve(root, path);
            }
        }
    }
    Some(config)
}

/// POSIX `path.resolve(root, path)` for an absolute, normalized `root`.
fn resolve(root: &str, path: &str) -> std::string::String {
    let mut segments = Vec::new();
    let start = if path.starts_with('/') { "" } else { root };
    for segment in start.split('/').chain(path.split('/')) {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            segment => segments.push(segment),
        }
    }
    let mut resolved = std::string::String::with_capacity(root.len() + path.len() + 1);
    for segment in &segments {
        resolved.push('/');
        resolved.push_str(segment);
    }
    if resolved.is_empty() {
        resolved.push('/');
    }
    resolved
}

/// The schema `vueConfigOptions` validates against, read from the same
/// generated literal the JS runtime parses. `None` disables the fast path.
fn schema() -> Option<&'static Value> {
    static SCHEMA: OnceLock<Option<Value>> = OnceLock::new();
    SCHEMA
        .get_or_init(|| {
            let (_, rest) = super::js::VITE_RUNTIME.split_once("JSON.parse(")?;
            let (literal, _) = rest.trim_start().strip_prefix('\'')?.split_once('\'')?;
            // A JS escape would need decoding first.
            if literal.contains('\\') {
                return None;
            }
            serde_json::from_str(literal).ok()
        })
        .as_ref()
}

/// `validateVueOption` as a predicate: `Some(false)` where JS throws, and
/// `None` wherever this port cannot be sure (the caller then declines).
fn check(value: &Value, schema: &Value, root: &Value) -> Option<bool> {
    let schema = schema.as_object()?;
    if let Some(reference) = schema.get("$ref") {
        let name = reference.as_str()?.rsplit('/').next()?;
        return match root
            .get("definitions")
            .and_then(|definitions| definitions.get(name))
        {
            Some(definition) => check(value, definition, root),
            None => Some(false),
        };
    }
    if let Some(candidates) = schema.get("oneOf") {
        let mut matches = 0;
        for candidate in candidates.as_array()? {
            matches += usize::from(check(value, candidate, root)?);
        }
        return Some(matches == 1);
    }
    if let Some(allowed) = schema.get("enum") {
        let found = allowed
            .as_array()?
            .iter()
            .any(|allowed| same_value(allowed, value));
        if !found {
            return Some(false);
        }
    }
    if let Some(expected) = schema.get("type") {
        let valid = match expected.as_str()? {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "boolean" => value.is_boolean(),
            "integer" => value.as_f64().is_some_and(|number| number.fract() == 0.0),
            "number" => value.is_number(),
            "null" => value.is_null(),
            // JS looks the name up on a plain object, so inherited names pass.
            _ => return None,
        };
        if !valid {
            return Some(false);
        }
    }
    for (bound, below) in [("minimum", true), ("maximum", false)] {
        if let Some(bound) = schema.get(bound) {
            let (bound, number) = (bound.as_f64()?, value.as_f64()?);
            if (below && number < bound) || (!below && number > bound) {
                return Some(false);
            }
        }
    }
    if let (Value::Array(items), Some(rule)) = (value, schema.get("items")) {
        for item in items {
            if !check(item, rule, root)? {
                return Some(false);
            }
        }
    }
    let Value::Object(object) = value else {
        return Some(true);
    };
    for required in schema
        .get("required")
        .map_or(Some(&[][..]), |value| value.as_array().map(Vec::as_slice))?
    {
        if !object.contains_key(required.as_str()?) {
            return Some(false);
        }
    }
    // Property name patterns need a regex engine; Node keeps those values.
    if schema.contains_key("propertyNames") && !object.is_empty() {
        return None;
    }
    let properties = schema.get("properties").and_then(Value::as_object);
    let additional = schema.get("additionalProperties");
    for (key, item) in object {
        let rule = match properties.and_then(|properties| properties.get(key)) {
            Some(rule) => Some(rule),
            None if additional == Some(&Value::Bool(false)) => return Some(false),
            None => additional,
        };
        if let Some(rule) = rule.filter(|rule| rule.is_object())
            && !check(item, rule, root)?
        {
            return Some(false);
        }
    }
    Some(true)
}

/// `Array.prototype.includes` on JSON primitives (objects are never included).
fn same_value(allowed: &Value, value: &Value) -> bool {
    match (allowed, value) {
        (Value::Number(allowed), Value::Number(value)) => allowed.as_f64() == value.as_f64(),
        (Value::Array(_) | Value::Object(_), _) | (_, Value::Array(_) | Value::Object(_)) => false,
        (allowed, value) => allowed == value,
    }
}
