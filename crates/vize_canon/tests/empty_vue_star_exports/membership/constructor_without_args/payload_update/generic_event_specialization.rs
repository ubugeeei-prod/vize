//! Authenticate the generic-event repair before reading frozen producer history.
use super::support;
use serde_json::{Value, json};
use std::fs;
const RECORD: &str = "malformed-membership/generic-event-specialization-update-11815.json";
const RECORD_SHA: &str = "d9eee3c8eb982eba5e2144cca457ecf0b15e1edbb5948b019dd404e6c10ab845";
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
        "exact-generic-event-specialization-source-inverse"
    );
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["canonicalAssetsChangedByThisFix"], true);
    assert_eq!(record["files"].as_array().unwrap().len(), 5);
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
pub(super) fn source_edits_before(path: &str, mut current: String) -> String {
    current = super::constructor_emit_export::source_edits_before(path, current);
    let record = load();
    let Some(entry) = record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path)
    else {
        return current;
    };
    for edit in entry["edits"].as_array().unwrap() {
        let before = edit["before"].as_str().unwrap();
        let after = edit["after"].as_str().unwrap();
        assert_eq!(current.matches(after).count(), 1);
        current = current.replacen(after, before, 1);
    }
    current
}
pub(super) fn preamble_edits_before(mut current: String) -> String {
    let record = load();
    for edit in record["preamble"]["edits"].as_array().unwrap() {
        let before = edit["before"].as_str().unwrap();
        let after = edit["after"].as_str().unwrap();
        assert_eq!(current.matches(after).count(), 1);
        current = current.replacen(after, before, 1);
    }
    current
}
pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::constructor_emit_export::source_edits_before(path, current);
    source_before_base(path, current)
}
fn source_before_base(path: &str, current: String) -> String {
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
pub(super) fn preamble_before(current: &str) -> String {
    inverse(current, &load()["preamble"])
}
pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentGenericEventSpecializationUpdate"] = load();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn specialization_repair_restores_the_exact_previous_source_and_preamble() {
        let record = load();
        for entry in record["files"].as_array().unwrap() {
            let path = entry["path"].as_str().unwrap();
            let actual = fs::read_to_string(support::repository().join(path)).unwrap();
            let actual = super::super::emit_member_infer::source_before(path, actual);
            let current = super::super::constructor_emit_export::source_before(path, actual);
            authenticate(&source_before_base(path, current.clone()), &entry["before"]);
            for changed in [
                format!("{current}\n"),
                format!("{current}{current}"),
                current.replacen(entry["edits"][0]["after"].as_str().unwrap(), "", 1),
            ] {
                assert!(std::panic::catch_unwind(|| source_before_base(path, changed)).is_err());
            }
        }
        let physical = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        let current = super::super::integrated_source::preamble_before_input_and_overload(physical);
        authenticate(&preamble_before(&current), &record["preamble"]["before"]);
        assert!(std::panic::catch_unwind(|| preamble_before(&format!("{current}\n"))).is_err());
    }
}
