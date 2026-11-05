//! Reconstruct only the already-current #11600 layer before frozen history.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/current-loose-emitter-update-11863.json";
const RECORD_SHA: &str = "1cf8f7853ae0ffed96432d50e6db812e502faac15fe994d51a393cc0ef7aa8f9";
const PATHS: [&str; 2] = [
    "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt",
];
fn authenticate(text: &str, identity: &Value) {
    assert_eq!(json!(text.len()), identity["bytes"]);
    assert_eq!(support::hash(text.as_bytes()), identity["sha256"]);
}
fn load() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], "exact-current-loose-emitter-source-inverse");
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["historicalReconstructionOnly"], true);
    assert_eq!(record["sdkAssetChanges"], 0);
    assert_eq!(record["files"].as_array().unwrap().len(), PATHS.len());
    for (entry, path) in record["files"].as_array().unwrap().iter().zip(PATHS) {
        assert_eq!(entry["path"], path);
        let current = fs::read_to_string(support::repository().join(path)).unwrap();
        let current = super::integrated_source::source_before(path, current);
        authenticate(&current, &entry["after"]);
    }
    record
}
fn inverse(current: &str, transition: &Value) -> String {
    authenticate(current, &transition["after"]);
    let mut previous = current.to_owned();
    for edit in transition["edits"].as_array().unwrap() {
        let after = edit["after"].as_str().unwrap();
        assert!(!after.is_empty());
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, edit["before"].as_str().unwrap(), 1);
    }
    authenticate(&previous, &transition["before"]);
    previous
}
pub(super) fn source_before(path: &str, current: String) -> String {
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
#[cfg(test)]
pub(super) fn preamble_after(previous: &str) -> String {
    let record = load();
    let transition = &record["preamble"];
    authenticate(previous, &transition["before"]);
    let mut current = previous.to_owned();
    for edit in transition["edits"].as_array().unwrap() {
        let before = edit["before"].as_str().unwrap();
        assert!(!before.is_empty());
        assert_eq!(current.matches(before).count(), 1);
        current = current.replacen(before, edit["after"].as_str().unwrap(), 1);
    }
    authenticate(&current, &transition["after"]);
    current
}
pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentLooseEmitterUpdate"] = load();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn current_loose_layer_restores_exact_historical_sources_and_preamble() {
        let record = load();
        for entry in record["files"].as_array().unwrap() {
            let path = entry["path"].as_str().unwrap();
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            let current = super::super::integrated_source::source_before(path, current);
            authenticate(&source_before(path, current.clone()), &entry["before"]);
            let before = entry["edits"][0]["before"].as_str().unwrap();
            let after = entry["edits"][0]["after"].as_str().unwrap();
            for mutation in [
                String::new(),
                format!("{current}{current}"),
                current.replacen(after, before, 1),
                format!("{current}\n"),
            ] {
                assert!(std::panic::catch_unwind(|| source_before(path, mutation)).is_err());
            }
        }
        let physical = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        let current = super::super::integrated_source::preamble_before(physical);
        let previous = preamble_before(&current);
        assert_eq!(preamble_after(&previous), current);
        let edit = &record["preamble"]["edits"][0];
        let after = edit["after"].as_str().unwrap();
        for mutation in [
            current.replacen(after, "", 1),
            format!("{current}{after}"),
            previous,
            format!("{current}\n"),
        ] {
            assert!(std::panic::catch_unwind(|| preamble_before(&mutation)).is_err());
        }
    }
}
