//! Reverse #13420 before authenticating the earlier complete SDK sources.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/forwarded-slot-exports-13420.json";
const RECORD_SHA: &str = "e5eca256e2ad07a35480aedc57ee371e05540a0d0d07ea3f1a91c79a4eb33312";
const CHANGED_PATHS: [&str; 2] = [
    "npm/cli/virtual-ts-slots.d.ts",
    "npm/cli/scripts/generate-content-mapper-types.ts",
];
const ADDED_OWNERS: [&str; 1] = ["npm/cli/scripts/slot-family-exports.ts"];

fn authenticate(source: &str, identity: &Value) {
    assert_eq!(json!(source.len()), identity["bytes"]);
    assert_eq!(support::hash(source.as_bytes()), identity["sha256"]);
    assert_eq!(source, identity["wholeUtf8"].as_str().unwrap());
}

fn has_identity(source: &str, identity: &Value) -> bool {
    json!(source.len()) == identity["bytes"]
        && support::hash(source.as_bytes()) == identity["sha256"]
}

fn load() -> Value {
    static LOADED: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    LOADED.get_or_init(load_uncached).clone()
}

fn load_uncached() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], "exact-forwarded-slot-export-source-inverse");
    assert_eq!(record["issue"], 13420);
    assert_eq!(
        record["sourceRevision"],
        "65e78e02fcb7769cb7cf6419cf1d0f137e4a4e37"
    );
    assert_eq!(
        record["beforeSourceRevision"],
        "ebc81117104d7afdb6d05ec8428e3c6d05348cc1"
    );
    assert_eq!(record["nativeAcceptanceCredit"], false);
    for (entry, path) in record["files"]
        .as_array()
        .unwrap()
        .iter()
        .zip(CHANGED_PATHS)
    {
        assert_eq!(entry["path"], path);
        assert_eq!(entry["before"]["path"], path);
        assert_eq!(entry["after"]["path"], path);
    }
    for (entry, path) in record["addedOwners"]
        .as_array()
        .unwrap()
        .iter()
        .zip(ADDED_OWNERS)
    {
        assert_eq!(entry["path"], path);
    }
    record
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::optional_jsx_witness::source_before(path, current);
    let record = load();
    let Some(entry) = record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path)
    else {
        return current;
    };
    if has_identity(&current, &entry["before"]) {
        return current;
    }
    authenticate(&current, &entry["after"]);
    let previous = entry["before"]["wholeUtf8"].as_str().unwrap().to_owned();
    authenticate(&previous, &entry["before"]);
    previous
}

pub(super) fn source_edits_before(path: &str, current: String) -> String {
    let record = load();
    let Some(entry) = record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path)
    else {
        return current;
    };
    if has_identity(&current, &entry["after"]) {
        return entry["before"]["wholeUtf8"].as_str().unwrap().to_owned();
    }
    current
}

fn authenticate_added_owners(record: &Value) {
    for identity in record["addedOwners"].as_array().unwrap() {
        let path = identity["path"].as_str().unwrap();
        let current = fs::read_to_string(support::repository().join(path)).unwrap();
        authenticate(&current, identity);
    }
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    let record = load();
    authenticate_added_owners(&record);
    manifest["provenance"]["forwardedSlotExports"] = record;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_slot_module_and_generator_reconstruct_the_exact_previous_sources() {
        let record = load();
        authenticate_added_owners(&record);
        for path in CHANGED_PATHS {
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            let previous = source_before(path, current.clone());
            let entry = record["files"]
                .as_array()
                .unwrap()
                .iter()
                .find(|entry| entry["path"] == path)
                .unwrap();
            authenticate(&previous, &entry["before"]);
            for changed in [format!("{current}\n"), format!("{current}{current}")] {
                assert!(std::panic::catch_unwind(|| source_before(path, changed)).is_err());
            }
        }
    }
}
