//! Keep the existing 11600 producer delta separate from the public SDK repair.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/canonical-loose-emit-sdk-update-11815.json";
const RECORD_SHA: &str = "b39a4a47a11d2e343c2cec25a899830c0ac13403b9445d2bc73715f533e22247";

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
        "exact-canonical-loose-emit-sdk-source-inverse"
    );
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["canonicalAssetsChangedByThisFix"], false);
    assert_eq!(record["files"].as_array().unwrap().len(), 6);
    assert_eq!(record["newAssets"].as_array().unwrap().len(), 1);
    for asset in record["newAssets"].as_array().unwrap() {
        let current =
            fs::read_to_string(support::repository().join(asset["path"].as_str().unwrap()))
                .unwrap();
        let current =
            super::emit_member_infer::source_before(asset["path"].as_str().unwrap(), current);
        let previous =
            super::private_emit_sdk::source_before(asset["path"].as_str().unwrap(), current);
        authenticate(&previous, asset);
    }
    record
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
pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::private_emit_sdk::source_before(path, current);
    let record = load();
    match record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path)
    {
        Some(entry) => inverse(&current, entry),
        None => current,
    }
}
pub(super) fn source_edits_before(path: &str, current: String) -> String {
    let current = super::private_emit_sdk::source_edits_before(path, current);
    let record = load();
    let Some(entry) = record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path)
    else {
        return current;
    };
    let mut previous = current;
    for edit in entry["edits"].as_array().unwrap() {
        let before = edit["before"].as_str().unwrap();
        let after = edit["after"].as_str().unwrap();
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, before, 1);
    }
    previous
}
pub(super) fn preamble_before(current: &str) -> String {
    let current = super::generic_event_specialization::preamble_before(current);
    inverse(&current, &load()["preamble"])
}
pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentCanonicalLooseEmitSdkUpdate"] = load();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn current_sdk_and_existing_producer_restore_the_exact_old_boundaries() {
        let record = load();
        for entry in record["files"].as_array().unwrap() {
            let path = entry["path"].as_str().unwrap();
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            let current = super::super::listener_relations::source_before(path, current);
            let current = super::super::integrated_source::overload_before(path, current);
            authenticate(&source_before(path, current.clone()), &entry["before"]);
            for changed in [format!("{current}\n"), format!("{current}{current}")] {
                assert!(std::panic::catch_unwind(|| source_before(path, changed)).is_err());
            }
            let added = entry["edits"][0]["after"].as_str().unwrap();
            let canonical = super::super::private_emit_sdk::source_before(path, current);
            assert_eq!(canonical.matches(added).count(), 1);
            assert!(
                std::panic::catch_unwind(|| inverse(&canonical.replacen(added, "", 1), entry))
                    .is_err()
            );
        }
        let physical = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        let current = super::super::integrated_source::preamble_before_input_and_overload(physical);
        let previous = preamble_before(&current);
        authenticate(&previous, &record["preamble"]["before"]);
        let edit = &record["preamble"]["edits"][0];
        assert_eq!(
            previous.replacen(
                edit["before"].as_str().unwrap(),
                edit["after"].as_str().unwrap(),
                1
            ),
            super::super::generic_event_specialization::preamble_before(&current)
        );
        assert!(std::panic::catch_unwind(|| preamble_before(&format!("{current}\n"))).is_err());
    }
}
