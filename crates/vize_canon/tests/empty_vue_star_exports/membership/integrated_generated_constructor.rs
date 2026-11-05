//! Exact generated successors after joining the generic constructor source.
use super::support;
use serde_json::{Value, json};
use std::fs;

fn load(path: &str, sha256: &str) -> Value {
    let bytes = fs::read(support::corpus().join(path)).unwrap();
    assert_eq!(support::hash(&bytes), sha256);
    serde_json::from_slice(&bytes).unwrap()
}

fn current(previous: &Value, record: &Value, entry: &Value) -> String {
    assert_eq!(entry["before"], previous["after"]);
    let historical = previous["afterWholeUtf8"].as_str().unwrap();
    assert_eq!(json!(historical.len()), entry["before"][1]);
    assert_eq!(support::hash(historical.as_bytes()), entry["before"][2]);
    let before = record["edit"]["before"].as_str().unwrap();
    let after = record["edit"]["after"].as_str().unwrap();
    assert_eq!(historical.matches(before).count(), 1);
    assert_eq!(historical.matches(after).count(), 0);
    let current = historical.replacen(before, after, 1);
    assert_eq!(historical.len() - current.len(), 255);
    assert_eq!(json!(current.len()), entry["after"][1]);
    assert_eq!(support::hash(current.as_bytes()), entry["after"][2]);
    current
}

pub(super) fn apply(manifest: &mut Value, members: &mut [Value]) {
    let previous = load(
        "malformed-membership/owned-constructor-20261105.json",
        "377ce83df86cc659c5e5e9dfa94ef4d3a7265d5048aaa37f399d48be911c619b",
    );
    let record = load(
        "malformed-membership/integrated-constructor-11964.json",
        "8b744fba21c4b1c1f12865cfafd6788bc06c06b8f0c765008fa3c97db9c85cd4",
    );
    assert_eq!(record["version"], 1);
    assert_eq!(
        record["kind"],
        "exact-integrated-generic-constructor-successor"
    );
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["observedRun"], 42070445017_i64);
    assert_eq!(record["observedMismatchEntry"], 74);
    assert_eq!(record["observedActual"], record["entries"][0]["after"]);
    let entries = record["entries"].as_array().unwrap();
    let previous_entries = previous["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(previous_entries.len(), 2);
    for ((entry, previous), (index, path)) in entries.iter().zip(previous_entries).zip([
        (74, "project/A.vue.ts"),
        (75, "project/B.vue.ts"),
    ]) {
        assert_eq!(entry["entry"], index);
        assert_eq!(entry["before"][0], path);
        assert_eq!(entry["after"][0], path);
        current(previous, &record, entry);
        let member = members.get_mut(index).unwrap();
        assert_eq!(*member, entry["before"]);
        *member = entry["after"].clone();
    }
    manifest["provenance"]["currentIntegratedConstructor"] = record;
}

#[test]
fn integrated_constructor_transition_refuses_unrelated_or_duplicate_bytes() {
    let previous = load(
        "malformed-membership/owned-constructor-20261105.json",
        "377ce83df86cc659c5e5e9dfa94ef4d3a7265d5048aaa37f399d48be911c619b",
    );
    let record = load(
        "malformed-membership/integrated-constructor-11964.json",
        "8b744fba21c4b1c1f12865cfafd6788bc06c06b8f0c765008fa3c97db9c85cd4",
    );
    for (entry, previous) in record["entries"]
        .as_array()
        .unwrap()
        .iter()
        .zip(previous["entries"].as_array().unwrap())
    {
        let current = current(previous, &record, entry);
        let after = record["edit"]["after"].as_str().unwrap();
        for changed in [
            format!("{current}\n"),
            current.replace(after, ""),
            format!("{current}{after}"),
        ] {
            assert_ne!(support::hash(changed.as_bytes()), entry["after"][2]);
        }
    }
}
