//! Exact generated successors for the #11499 public-instance projection split.
use super::support;
use serde_json::{Value, json};
use std::fs;

const OWNED: &str = "malformed-membership/owned-constructor-20261105.json";
const OWNED_SHA: &str = "377ce83df86cc659c5e5e9dfa94ef4d3a7265d5048aaa37f399d48be911c619b";
const INTEGRATED: &str = "malformed-membership/integrated-constructor-11964.json";
const INTEGRATED_SHA: &str = "8b744fba21c4b1c1f12865cfafd6788bc06c06b8f0c765008fa3c97db9c85cd4";
const SUCCESSOR: &str = "malformed-membership/canon-leftovers-generated-11499.json";
const SUCCESSOR_SHA: &str = "efca7fb4583447052c5886cfa3562247af74b32aa4b73741b15e38d85636c4cd";

fn load(path: &str, sha256: &str) -> Value {
    let bytes = fs::read(support::corpus().join(path)).unwrap();
    assert_eq!(support::hash(&bytes), sha256);
    serde_json::from_slice(&bytes).unwrap()
}

fn integrated_body(owned: &Value, integrated: &Value, index: usize) -> String {
    let prior = owned["entries"][index]["afterWholeUtf8"].as_str().unwrap();
    let before = integrated["edit"]["before"].as_str().unwrap();
    let after = integrated["edit"]["after"].as_str().unwrap();
    assert_eq!(prior.matches(before).count(), 1);
    assert_eq!(prior.matches(after).count(), 0);
    let current = prior.replacen(before, after, 1);
    let entry = &integrated["entries"][index];
    assert_eq!(json!(current.len()), entry["after"][1]);
    assert_eq!(support::hash(current.as_bytes()), entry["after"][2]);
    current
}

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

fn records() -> (Value, Value, Value) {
    let owned = load(OWNED, OWNED_SHA);
    let integrated = load(INTEGRATED, INTEGRATED_SHA);
    let successor = load(SUCCESSOR, SUCCESSOR_SHA);
    assert_eq!(successor["version"], 1);
    assert_eq!(
        successor["kind"],
        "exact-canon-leftovers-generated-constructor-successor"
    );
    assert_eq!(successor["issue"], 11499);
    assert_eq!(successor["pullRequest"], 12162);
    assert_eq!(successor["observedRun"], 42458808867_i64);
    assert_eq!(successor["observedJob"], 128663581942_i64);
    assert_eq!(successor["artifact"], 13608737806_i64);
    assert_eq!(successor["nativeAcceptanceCredit"], false);
    assert_eq!(successor["observedMismatchEntry"], 74);
    assert_eq!(
        successor["observedActual"],
        successor["entries"][0]["after"]
    );
    (owned, integrated, successor)
}

pub(super) fn apply(manifest: &mut Value, members: &mut [Value]) {
    let (owned, integrated, successor) = records();
    let entries = successor["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    for (index, (entry, path)) in entries
        .iter()
        .zip(["project/A.vue.ts", "project/B.vue.ts"])
        .enumerate()
    {
        let member_index = index + 74;
        assert_eq!(entry["entry"], member_index);
        assert_eq!(entry["before"], integrated["entries"][index]["after"]);
        assert_eq!(entry["before"][0], path);
        assert_eq!(entry["after"][0], path);
        let current = integrated_body(&owned, &integrated, index);
        successor_body(&current, &successor, entry);
        let member = members.get_mut(member_index).unwrap();
        assert_eq!(*member, entry["before"]);
        *member = entry["after"].clone();
    }
    manifest["provenance"]["currentCanonLeftoversGenerated11499"] = successor;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_generated_successors_reconstruct_from_the_authenticated_predecessors() {
        let (owned, integrated, successor) = records();
        for (index, entry) in successor["entries"].as_array().unwrap().iter().enumerate() {
            let current = integrated_body(&owned, &integrated, index);
            successor_body(&current, &successor, entry);
        }
    }

    #[test]
    fn missing_duplicate_stale_and_foreign_generated_bytes_are_refused() {
        let (owned, integrated, successor) = records();
        let before = successor["edit"]["before"].as_str().unwrap();
        for (index, entry) in successor["entries"].as_array().unwrap().iter().enumerate() {
            let current = integrated_body(&owned, &integrated, index);
            for changed in [
                format!("{current}\n"),
                current.replacen(before, "", 1),
                format!("{current}{before}"),
            ] {
                assert!(
                    std::panic::catch_unwind(|| successor_body(&changed, &successor, entry))
                        .is_err()
                );
            }
        }
    }
}
