//! Reverse the member-inferred emit factories before authenticating older SDK history.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/emit-member-infer-update-20261108.json";
const RECORD_SHA: &str = "6139d1767031cee439c9747af609f20d43aedcf64863a2277e4e1459b44d3902";

fn authenticate(current: &str, identity: &Value) {
    assert_eq!(json!(current.len()), identity["bytes"]);
    assert_eq!(support::hash(current.as_bytes()), identity["sha256"]);
}
fn load() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], "exact-emit-member-infer-source-inverse");
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["canonicalAssetsChangedByThisFix"], true);
    assert_eq!(record["files"].as_array().unwrap().len(), 4);
    record
}
/// Replace every recorded successor span by its predecessor; text without any
/// successor span (older history, or an already restored image) passes through.
fn edits_before(mut current: String, edits: &Value) -> String {
    let edits = edits.as_array().unwrap();
    let counts = edits
        .iter()
        .map(|edit| current.matches(edit["after"].as_str().unwrap()).count())
        .collect::<Vec<_>>();
    if counts.iter().all(|count| *count == 0) {
        return current;
    }
    assert!(counts.iter().all(|count| *count == 1));
    for edit in edits {
        let after = edit["after"].as_str().unwrap();
        assert!(!after.is_empty());
        current = current.replacen(after, edit["before"].as_str().unwrap(), 1);
    }
    current
}
fn inverse(current: &str, entry: &Value) -> String {
    authenticate(current, &entry["after"]);
    let previous = edits_before(current.to_owned(), &entry["edits"]);
    authenticate(&previous, &entry["before"]);
    previous
}
fn entry<'a>(record: &'a Value, path: &str) -> Option<&'a Value> {
    record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path)
}
pub(super) fn source_before(path: &str, current: String) -> String {
    let record = load();
    match entry(&record, path) {
        Some(entry) => edits_before(current, &entry["edits"]),
        None => current,
    }
}
pub(super) fn preamble_before(current: &str) -> String {
    edits_before(current.to_owned(), &load()["preamble"]["edits"])
}
pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentEmitMemberInferUpdate"] = load();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_inferred_factories_restore_the_exact_previous_source_and_preamble() {
        let record = load();
        for entry in record["files"].as_array().unwrap() {
            let path = entry["path"].as_str().unwrap();
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            let previous = inverse(&current, entry);
            assert_eq!(source_before(path, current.clone()), previous);
            assert_eq!(source_before(path, previous.clone()), previous);
            for changed in [
                format!("{current}\n"),
                format!("{current}{current}"),
                current.replacen(entry["edits"][0]["after"].as_str().unwrap(), "", 1),
            ] {
                assert!(std::panic::catch_unwind(|| inverse(&changed, entry)).is_err());
            }
        }
        let physical = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        let previous = inverse(physical, &record["preamble"]);
        assert_eq!(preamble_before(physical), previous);
        assert_eq!(preamble_before(&previous), previous);
        assert!(
            std::panic::catch_unwind(|| inverse(&format!("{physical}\n"), &record["preamble"]))
                .is_err()
        );
        let other = "crates/vize_canon/src/virtual_ts/helpers/declarations/vue3-types.d.ts.txt";
        let body = fs::read_to_string(support::repository().join(other)).unwrap();
        assert_eq!(source_before(other, body.clone()), body);
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
        assert_eq!(
            manifest["provenance"]["currentEmitMemberInferUpdate"],
            record
        );
    }
}
