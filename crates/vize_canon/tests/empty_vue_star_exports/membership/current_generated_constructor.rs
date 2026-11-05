//! Two exact generated bodies after moving public instance extras outside the mapped props.
use super::support;
use serde_json::{Value, json};
use std::fs;

fn load() -> Value {
    let bytes =
        fs::read(support::corpus().join("malformed-membership/owned-constructor-20261105.json"))
            .unwrap();
    assert_eq!(
        support::hash(&bytes),
        "377ce83df86cc659c5e5e9dfa94ef4d3a7265d5048aaa37f399d48be911c619b"
    );
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(
        record["kind"],
        "exact-owned-constructor-generated-contract-inverse"
    );
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["observedRun"], 41975392372_i64);
    assert_eq!(record["artifact"], 13392301468_i64);
    record
}

fn previous(current: &str, record: &Value, entry: &Value) -> String {
    assert_eq!(current, entry["afterWholeUtf8"].as_str().unwrap());
    assert_eq!(json!(current.len()), entry["after"][1]);
    assert_eq!(
        support::hash(current.as_bytes()),
        entry["after"][2].as_str().unwrap()
    );
    let after = record["edit"]["after"].as_str().unwrap();
    let before = record["edit"]["before"].as_str().unwrap();
    assert_eq!(current.matches(after).count(), 1);
    let prior = current.replacen(after, before, 1);
    assert_eq!(prior, entry["beforeWholeUtf8"].as_str().unwrap());
    assert_eq!(json!(prior.len()), entry["before"][1]);
    assert_eq!(
        support::hash(prior.as_bytes()),
        entry["before"][2].as_str().unwrap()
    );
    assert_eq!(current.len() - prior.len(), 121);
    prior
}

pub(super) fn apply(manifest: &mut Value, members: &mut [Value]) {
    let record = load();
    let entries = record["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    for (entry, (index, path)) in entries
        .iter()
        .zip([(74, "project/A.vue.ts"), (75, "project/B.vue.ts")])
    {
        assert_eq!(entry["entry"], index);
        assert_eq!(entry["before"][0], path);
        assert_eq!(entry["after"][0], path);
        previous(entry["afterWholeUtf8"].as_str().unwrap(), &record, entry);
        let member = members.get_mut(index).unwrap();
        assert_eq!(*member, entry["before"]);
        *member = entry["after"].clone();
    }
    manifest["provenance"]["currentOwnedConstructor"] = record;
}

#[test]
fn generated_constructor_transition_refuses_unrelated_or_duplicate_bytes() {
    let record = load();
    for entry in record["entries"].as_array().unwrap() {
        let current = entry["afterWholeUtf8"].as_str().unwrap();
        previous(current, &record, entry);
        let after = record["edit"]["after"].as_str().unwrap();
        for changed in [
            format!("{current}\n"),
            current.replace(after, ""),
            format!("{current}{after}"),
        ] {
            assert!(std::panic::catch_unwind(|| previous(&changed, &record, entry)).is_err());
        }
    }
}
