//! Read-only whole-file custody through the official metadata transformation.
use super::{github, metadata};
use serde_json::{json, Value};
use std::{fs, path::Path};

fn minor(version: &str) -> Result<u64, String> {
    let value = version
        .strip_prefix("0.")
        .and_then(|v| v.strip_suffix(".0"))
        .ok_or("Producer metadata requires a stable 0.N.0 version.")?;
    let number: u64 = value
        .parse()
        .map_err(|_| "Invalid producer minor version.")?;
    if number.to_string() != value {
        return Err("Producer minor versions must use canonical decimal spelling.".into());
    }
    Ok(number)
}

pub(super) fn verify(frozen: &Path, root: &Path) -> Result<Value, String> {
    let before_root = fs::read_to_string(frozen.join("Cargo.toml")).map_err(|e| e.to_string())?;
    let current_root = fs::read_to_string(root.join("Cargo.toml")).map_err(|e| e.to_string())?;
    let old = github::version_text(&before_root)?;
    let new = github::version_text(&current_root)?;
    if minor(&new)? < minor(&old)? {
        return Err("Producer metadata cannot move to an older minor.".into());
    }
    let mut files = Vec::new();
    for path in ["Cargo.toml", "Cargo.lock"] {
        let before = fs::read_to_string(frozen.join(path)).map_err(|e| e.to_string())?;
        let current = fs::read_to_string(root.join(path)).map_err(|e| e.to_string())?;
        let expected = metadata::rewrite(path, &before, &old, &new);
        if current != expected || metadata::rewrite(path, &current, &new, &old) != before {
            return Err(format!(
                "{path} is not the complete reversible official metadata rewrite."
            ));
        }
        // A release must actually advance both files; a partial lock update is not custody.
        if old != new && current == before {
            return Err(format!("{path} did not advance with the producer minor."));
        }
        files.push(json!({"path": path, "changed": current != before}));
    }
    Ok(json!({
        "schema": 1, "baselineVersion": old, "currentVersion": new,
        "wholeForwardAndInverse": true, "files": files,
        "readOnly": true, "publicationQualification": false
    }))
}

pub fn verify_producer_metadata(frozen: &str, root: &Path) -> Result<(), String> {
    let receipt = verify(&root.join(frozen), root)?;
    println!("{receipt}");
    Ok(())
}
