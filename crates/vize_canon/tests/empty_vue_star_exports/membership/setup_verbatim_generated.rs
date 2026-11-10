//! Exact generated successors for #12481: authored setup lines lose their two-space prefix.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/setup-verbatim-generated-12481.json";
const RECORD_SHA: &str = "c874749fd13c56cb6a7d842321dee8eae2074345e06795cda5b2cf99853866aa";

fn record() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], "exact-setup-verbatim-generated-successor");
    assert_eq!(record["issue"], 12481);
    assert_eq!(record["pullRequest"], 12550);
    assert_eq!(record["nativeAcceptanceCredit"], false);
    record
}

/// The entry's edit on the authenticated predecessor bytes: only setup indentation and its map comment move.
fn successor_body(current: &str, entry: &Value) -> String {
    assert_eq!(json!(current.len()), entry["before"][1]);
    assert_eq!(support::hash(current.as_bytes()), entry["before"][2]);
    let before = entry["edit"]["before"].as_str().unwrap();
    let after = entry["edit"]["after"].as_str().unwrap();
    assert!(
        before.starts_with("  // User setup code\n") && after.starts_with("  // User setup code\n")
    );
    let dedented = before
        .lines()
        .map(|line| match line {
            "  ;" => line,
            _ if line.starts_with("  // @vize-map:") || line == "  // User setup code" => line,
            _ => line.strip_prefix("  ").unwrap_or(line),
        })
        .collect::<Vec<_>>();
    let expected = after.lines().collect::<Vec<_>>();
    assert_eq!(dedented.len(), expected.len());
    for (left, right) in dedented.iter().zip(&expected) {
        if !left.starts_with("  // @vize-map:") {
            assert_eq!(left, right, "only the two-space setup prefix may change");
        }
    }
    assert_eq!(current.matches(before).count(), 1);
    let successor = current.replacen(before, after, 1);
    assert_eq!(json!(successor.len()), entry["after"][1]);
    assert_eq!(support::hash(successor.as_bytes()), entry["after"][2]);
    assert_eq!(
        json!(successor.len() as i64 - current.len() as i64),
        entry["deltaBytes"]
    );
    successor
}

pub(super) fn apply(manifest: &mut Value, members: &mut [Value]) {
    let record = record();
    let entries = record["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    for (index, entry) in entries.iter().enumerate() {
        let member_index = index + 74;
        assert_eq!(entry["entry"], member_index);
        assert_eq!(entry["after"][0], entry["before"][0]);
        let member = members.get_mut(member_index).unwrap();
        assert_eq!(*member, entry["before"]);
        *member = entry["after"].clone();
    }
    manifest["provenance"]["currentSetupVerbatimGenerated12481"] = record;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_generated_successors_reconstruct_from_the_authenticated_predecessors() {
        let record = record();
        for (body, entry) in super::super::constructor_noinfer_generated::current_bodies()
            .iter()
            .zip(record["entries"].as_array().unwrap())
        {
            successor_body(body, entry);
            for changed in [
                format!("{body}\n"),
                body.replacen("  // User setup code\n", "", 1),
            ] {
                assert!(std::panic::catch_unwind(|| successor_body(&changed, entry)).is_err());
            }
        }
    }
}
