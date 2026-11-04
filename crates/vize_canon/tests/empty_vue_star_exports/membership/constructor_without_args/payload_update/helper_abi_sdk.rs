//! Remove only the ABI generator hook before authenticating frozen SDK history.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/helper-abi-generator-update-11809.json";
const RECORD_SHA: &str = "edd6b540841d10aa680e954a73f5cb9592077632c8ce2bb250cbc605951dad9d";

fn authenticate(current: &str, identity: &Value) {
    assert_eq!(json!(current.len()), identity["bytes"]);
    assert_eq!(support::hash(current.as_bytes()), identity["sha256"]);
}

fn load() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], "exact-helper-abi-generator-source-inverse");
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["canonicalAssetsChangedByThisFix"], false);
    assert_eq!(record["files"].as_array().unwrap().len(), 1);
    record
}

pub(super) fn source_before(path: &str, current: String) -> String {
    if path != "npm/cli/scripts/generate-content-mapper-types.ts" {
        return current;
    }
    let record = load();
    let entry = &record["files"][0];
    assert_eq!(entry["path"], path);
    authenticate(&current, &entry["after"]);
    let mut previous = current;
    for edit in entry["edits"].as_array().unwrap() {
        let after = edit["after"].as_str().unwrap();
        assert!(!after.is_empty());
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, edit["before"].as_str().unwrap(), 1);
    }
    authenticate(&previous, &entry["before"]);
    previous
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentHelperAbiGeneratorUpdate"] = load();
}
