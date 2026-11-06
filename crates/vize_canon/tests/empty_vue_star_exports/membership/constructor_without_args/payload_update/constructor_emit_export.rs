//! Reverse the #11982 declaration export before authenticating frozen SDK history.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/constructor-emit-export-update-11982.json";
const RECORD_SHA: &str = "a245df16bcb622ea50b6c3a9db839d986ec5413c3b9a63dc0a45d5e894284187";

fn authenticate(current: &str, identity: &Value) {
    assert_eq!(json!(current.len()), identity["bytes"]);
    assert_eq!(support::hash(current.as_bytes()), identity["sha256"]);
}
fn load() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(
        record["kind"],
        "exact-constructor-emit-export-source-inverse"
    );
    assert_eq!(record["issue"], 11982);
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["files"].as_array().unwrap().len(), 4);
    record
}
fn edits_before(path: &str, mut current: String) -> String {
    let record = load();
    let Some(entry) = record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path)
    else {
        return current;
    };
    let counts = entry["edits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|edit| current.matches(edit["after"].as_str().unwrap()).count())
        .collect::<Vec<_>>();
    if counts.iter().all(|count| *count == 0) {
        return current;
    }
    assert!(counts.iter().all(|count| *count == 1));
    for edit in entry["edits"].as_array().unwrap() {
        let before = edit["before"].as_str().unwrap();
        let after = edit["after"].as_str().unwrap();
        assert!(!after.is_empty());
        current = current.replacen(after, before, 1);
    }
    current
}
pub(super) fn source_edits_before(path: &str, current: String) -> String {
    edits_before(path, current)
}
pub(super) fn source_before(path: &str, current: String) -> String {
    let record = load();
    let Some(entry) = record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path)
    else {
        return current;
    };
    authenticate(&current, &entry["after"]);
    let previous = edits_before(path, current);
    authenticate(&previous, &entry["before"]);
    previous
}
pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentConstructorEmitExportUpdate"] = load();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_export_reconstructs_the_exact_previous_sdk_source() {
        let record = load();
        for entry in record["files"].as_array().unwrap() {
            let path = entry["path"].as_str().unwrap();
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            authenticate(&source_before(path, current.clone()), &entry["before"]);
            for changed in [format!("{current}\n"), format!("{current}{current}")] {
                assert!(std::panic::catch_unwind(|| source_before(path, changed)).is_err());
            }
        }
    }
}
