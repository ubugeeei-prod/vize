//! `extends` composition for dedicated config files.
//!
//! A config names its bases as paths relative to itself or as package
//! specifiers. Bases apply in declaration order and the extending config
//! overrides them: objects merge key by key, `entries` concatenate so that
//! scoped overrides keep their order, and every other value is replaced.
//! An entry may also extend a base, which then contributes settings to that
//! entry while the entry keeps its own scope.

use std::{
    io::{Error as IoError, ErrorKind},
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::{Map, Value};

use super::{discovery::CONFIG_FILE_NAMES, js, pkl};

type LoadError = Box<dyn std::error::Error>;

const EXTENDS: &str = "extends";
const ENTRIES: &str = "entries";
/// Keys that describe one file or one entry's scope rather than shared settings.
const DOCUMENT_ONLY_KEYS: [&str; 2] = ["$schema", "projectRoot"];
const ENTRY_SCOPE_KEYS: [&str; 4] = ["name", "basePath", "files", "ignores"];

/// Cheap pre-check that keeps configs without `extends` on the typed reader.
pub(super) fn mentions_extends(json: &[u8]) -> bool {
    const NEEDLE: &[u8] = b"\"extends\"";
    json.windows(NEEDLE.len()).any(|window| window == NEEDLE)
}

/// Compose every `extends` chain reachable from the document read at `path`.
pub(super) fn compose(value: Value, path: &Path) -> Result<Value, LoadError> {
    let origin = canonical(path);
    let mut chain = Vec::with_capacity(4);
    chain.push(origin.clone());
    compose_document(value, &origin, &mut chain)
}

fn compose_document(
    value: Value,
    path: &Path,
    chain: &mut Vec<PathBuf>,
) -> Result<Value, LoadError> {
    let mut own = match value {
        Value::Object(own) => own,
        // A flat entry array is a document that only declares entries.
        Value::Array(entries) => {
            let mut own = Map::new();
            own.insert(ENTRIES.into(), Value::Array(entries));
            own
        }
        other => return Ok(other),
    };

    if let Some(Value::Array(entries)) = own.get_mut(ENTRIES) {
        for entry in entries.iter_mut() {
            *entry = compose_entry(std::mem::take(entry), path, chain)?;
        }
    }

    let Some(bases) = own.remove(EXTENDS) else {
        return Ok(Value::Object(own));
    };
    let mut composed = Map::new();
    for specifier in base_specifiers(bases, path)? {
        if let Value::Object(mut base) = load_base(&specifier, path, chain)? {
            for key in DOCUMENT_ONLY_KEYS {
                base.remove(key);
            }
            merge(&mut composed, base);
        }
    }
    merge(&mut composed, own);
    Ok(Value::Object(composed))
}

fn compose_entry(entry: Value, path: &Path, chain: &mut Vec<PathBuf>) -> Result<Value, LoadError> {
    let Value::Object(mut own) = entry else {
        return Ok(entry);
    };
    let Some(bases) = own.remove(EXTENDS) else {
        return Ok(Value::Object(own));
    };
    let mut composed = Map::new();
    for specifier in base_specifiers(bases, path)? {
        let Value::Object(mut base) = load_base(&specifier, path, chain)? else {
            continue;
        };
        if matches!(base.get(ENTRIES), Some(Value::Array(entries)) if !entries.is_empty()) {
            return Err(invalid(crate::cstr!(
                "{}: an entry cannot extend {specifier}, because that config declares entries",
                path.display()
            )));
        }
        base.remove(ENTRIES);
        for key in DOCUMENT_ONLY_KEYS.iter().chain(ENTRY_SCOPE_KEYS.iter()) {
            base.remove(*key);
        }
        merge(&mut composed, base);
    }
    merge(&mut composed, own);
    Ok(Value::Object(composed))
}

fn base_specifiers(bases: Value, path: &Path) -> Result<Vec<std::string::String>, LoadError> {
    let wrong_type = || {
        invalid(crate::cstr!(
            "{}: extends must be a string or an array of strings",
            path.display()
        ))
    };
    match bases {
        Value::Null => Ok(Vec::new()),
        Value::String(specifier) => Ok(Vec::from([specifier])),
        Value::Array(specifiers) => specifiers
            .into_iter()
            .map(|specifier| match specifier {
                Value::String(specifier) => Ok(specifier),
                _ => Err(wrong_type()),
            })
            .collect(),
        _ => Err(wrong_type()),
    }
}

fn load_base(specifier: &str, from: &Path, chain: &mut Vec<PathBuf>) -> Result<Value, LoadError> {
    let resolved = canonical(&resolve(specifier, from)?);
    if chain.contains(&resolved) {
        let mut cycle = std::string::String::new();
        for path in chain.iter() {
            cycle.push_str(&path.to_string_lossy());
            cycle.push_str(" -> ");
        }
        cycle.push_str(&resolved.to_string_lossy());
        return Err(invalid(crate::cstr!("Circular extends: {cycle}")));
    }
    chain.push(resolved.clone());
    let composed =
        read_value(&resolved).and_then(|value| compose_document(value, &resolved, chain));
    chain.pop();
    composed
}

fn resolve(specifier: &str, from: &Path) -> Result<PathBuf, LoadError> {
    let directory = from.parent().unwrap_or_else(|| Path::new("."));
    let is_path = specifier == "."
        || specifier == ".."
        || specifier.starts_with("./")
        || specifier.starts_with("../")
        || specifier.starts_with(".\\")
        || specifier.starts_with("..\\")
        || Path::new(specifier).is_absolute();
    let candidate = if is_path {
        directory.join(specifier)
    } else {
        resolve_package(specifier, directory)?
    };
    if candidate.is_file() {
        return Ok(candidate);
    }
    if candidate.is_dir()
        && let Some(found) = CONFIG_FILE_NAMES[..5]
            .iter()
            .map(|name| candidate.join(name))
            .find(|path| path.is_file())
    {
        return Ok(found);
    }
    Err(Box::new(IoError::new(
        ErrorKind::NotFound,
        crate::cstr!(
            "{}: cannot find the extended config {specifier}",
            from.display()
        )
        .to_string(),
    )))
}

/// Resolve a package specifier with Node, from the extending config's directory.
fn resolve_package(specifier: &str, directory: &Path) -> Result<PathBuf, LoadError> {
    const SCRIPT: &str = r#"
import { createRequire } from "node:module";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const specifier = process.argv[1];
let resolved;
try {
  resolved = fileURLToPath(import.meta.resolve(specifier));
} catch {
  resolved = createRequire(pathToFileURL(join(process.cwd(), "vize.config.json"))).resolve(specifier);
}
process.stdout.write(resolved);
"#;
    let output = Command::new("node")
        .arg("--input-type=module")
        .arg("-e")
        .arg(SCRIPT)
        .arg(specifier)
        .current_dir(directory)
        .output()?;
    if !output.status.success() {
        return Err(Box::new(IoError::new(
            ErrorKind::NotFound,
            crate::cstr!(
                "cannot resolve the extended config {specifier} from {}: {}",
                directory.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            )
            .to_string(),
        )));
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

fn read_value(path: &Path) -> Result<Value, LoadError> {
    let json = match path.extension().and_then(|extension| extension.to_str()) {
        Some("json") => std::fs::read(path)?,
        Some("ts" | "js" | "mjs" | "cjs" | "mts" | "cts") => js::evaluate_js_config(path)?.1,
        Some("pkl") => pkl::evaluate_pkl_config(path)?,
        _ => {
            return Err(invalid(crate::cstr!(
                "{}: an extended config must be a JSON, JavaScript, TypeScript or PKL file",
                path.display()
            )));
        }
    };
    serde_json::from_slice(&json)
        .map_err(|error| invalid(crate::cstr!("failed to parse {}: {error}", path.display())))
}

fn merge(target: &mut Map<std::string::String, Value>, source: Map<std::string::String, Value>) {
    for (key, value) in source {
        match (target.get_mut(&key), value) {
            (Some(Value::Object(target)), Value::Object(source)) => merge(target, source),
            (Some(Value::Array(target)), Value::Array(mut source)) if key == ENTRIES => {
                target.append(&mut source);
            }
            (_, value) => {
                target.insert(key, value);
            }
        }
    }
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn invalid(message: crate::String) -> LoadError {
    Box::new(IoError::new(ErrorKind::InvalidData, message.to_string()))
}
