//! Reverse the private-module repair before authenticating the original SDK sync.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/private-emit-sdk-update-11815.json";
const RECORD_SHA: &str = "e32afbdd3041677ca2ff7f26b2e8f0cab75aec58980ba30b529fd80f6f281a3c";

fn authenticate(current: &str, identity: &Value) {
    assert_eq!(json!(current.len()), identity["bytes"]);
    assert_eq!(support::hash(current.as_bytes()), identity["sha256"]);
}
fn load() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], "exact-private-emit-sdk-source-inverse");
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["canonicalAssetsChangedByThisFix"], false);
    assert_eq!(record["files"].as_array().unwrap().len(), 5);
    assert_eq!(record["newAssets"].as_array().unwrap().len(), 1);
    for asset in record["newAssets"].as_array().unwrap() {
        let current =
            fs::read_to_string(support::repository().join(asset["path"].as_str().unwrap()))
                .unwrap();
        let current = super::generic_event_specialization::source_before(
            asset["path"].as_str().unwrap(),
            current,
        );
        authenticate(&current, asset);
    }
    record
}
pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::generic_event_specialization::source_before(path, current);
    let record = load();
    let Some(entry) = record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path)
    else {
        return current;
    };
    inverse(&current, entry)
}
fn inverse(current: &str, entry: &Value) -> String {
    authenticate(current, &entry["after"]);
    let mut previous = current.to_owned();
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
    manifest["provenance"]["currentPrivateEmitSdkUpdate"] = load();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_module_repair_restores_the_exact_previous_sdk() {
        let record = load();
        for entry in record["files"].as_array().unwrap() {
            let path = entry["path"].as_str().unwrap();
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            authenticate(&source_before(path, current.clone()), &entry["before"]);
            for changed in [format!("{current}\n"), format!("{current}{current}")] {
                assert!(std::panic::catch_unwind(|| source_before(path, changed)).is_err());
            }
            let added = entry["edits"][0]["after"].as_str().unwrap();
            let current = super::super::generic_event_specialization::source_before(path, current);
            assert_eq!(current.matches(added).count(), 1);
            assert!(
                std::panic::catch_unwind(|| inverse(&current.replacen(added, "", 1), entry))
                    .is_err()
            );
        }
    }
}
