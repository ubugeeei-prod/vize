//! Exact generated successors for the #12523 `NoInfer` constructor return.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/constructor-noinfer-generated-12431.json";
const RECORD_SHA: &str = "2c5a7746e8ad7f2f16861664b2537957159fdf32e21ac07e6c63684b220d3bcd";
const PREDECESSOR: &str = "malformed-membership/canon-leftovers-generated-11499.json";
const PREDECESSOR_SHA: &str = "efca7fb4583447052c5886cfa3562247af74b32aa4b73741b15e38d85636c4cd";

fn load(path: &str, sha256: &str) -> Value {
    let bytes = fs::read(support::corpus().join(path)).unwrap();
    assert_eq!(support::hash(&bytes), sha256);
    serde_json::from_slice(&bytes).unwrap()
}

fn records() -> (Value, Value) {
    let predecessor = load(PREDECESSOR, PREDECESSOR_SHA);
    let record = load(RECORD, RECORD_SHA);
    assert_eq!(record["version"], 1);
    assert_eq!(
        record["kind"],
        "exact-constructor-noinfer-generated-successor"
    );
    assert_eq!(record["issue"], 12431);
    assert_eq!(record["pullRequest"], 12523);
    assert_eq!(record["observedRun"], 42663572515_i64);
    assert_eq!(record["observedJob"], 129323540869_i64);
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["observedMismatchEntry"], 74);
    assert_eq!(record["observedActual"], record["entries"][0]["after"]);
    (predecessor, record)
}

/// The edit on the authenticated predecessor bytes, checked against `entry`.
fn successor_body(current: &str, record: &Value, entry: &Value) -> String {
    assert_eq!(json!(current.len()), entry["before"][1]);
    assert_eq!(support::hash(current.as_bytes()), entry["before"][2]);
    let before = record["edit"]["before"].as_str().unwrap();
    let after = record["edit"]["after"].as_str().unwrap();
    assert_ne!(before, after);
    assert_eq!(current.matches(before).count(), 1);
    assert_eq!(current.matches(after).count(), 0);
    let successor = current.replacen(before, after, 1);
    assert_eq!(json!(successor.len()), entry["after"][1]);
    assert_eq!(support::hash(successor.as_bytes()), entry["after"][2]);
    assert_eq!(json!(successor.len() - current.len()), entry["deltaBytes"]);
    successor
}

/// The authenticated successor bodies, for the next layer's reconstruction.
#[cfg(test)]
pub(super) fn current_bodies() -> Vec<String> {
    let (_, record) = records();
    super::canon_leftovers_generated::current_bodies()
        .iter()
        .zip(record["entries"].as_array().unwrap())
        .map(|(body, entry)| successor_body(body, &record, entry))
        .collect()
}

pub(super) fn apply(manifest: &mut Value, members: &mut [Value]) {
    let (predecessor, record) = records();
    let entries = record["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    for (index, entry) in entries.iter().enumerate() {
        let member_index = index + 74;
        assert_eq!(entry["entry"], member_index);
        assert_eq!(entry["before"], predecessor["entries"][index]["after"]);
        assert_eq!(entry["after"][0], entry["before"][0]);
        let member = members.get_mut(member_index).unwrap();
        assert_eq!(*member, entry["before"]);
        *member = entry["after"].clone();
    }
    manifest["provenance"]["currentConstructorNoInferGenerated12431"] = record;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn predecessor_bodies() -> Vec<String> {
        super::super::canon_leftovers_generated::current_bodies()
    }

    #[test]
    fn exact_generated_successors_reconstruct_from_the_authenticated_predecessors() {
        let (_, record) = records();
        for (body, entry) in predecessor_bodies()
            .iter()
            .zip(record["entries"].as_array().unwrap())
        {
            successor_body(body, &record, entry);
        }
    }

    #[test]
    fn missing_duplicate_stale_and_foreign_generated_bytes_are_refused() {
        let (_, record) = records();
        let before = record["edit"]["before"].as_str().unwrap();
        for (body, entry) in predecessor_bodies()
            .iter()
            .zip(record["entries"].as_array().unwrap())
        {
            for changed in [
                format!("{body}\n"),
                body.replacen(before, "", 1),
                format!("{body}{before}"),
            ] {
                assert!(
                    std::panic::catch_unwind(|| successor_body(&changed, &record, entry)).is_err()
                );
            }
        }
    }
}
