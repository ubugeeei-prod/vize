//! Invert the component export memmem identifier search (#12835) before older custody.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "tests/_fixtures/tooling/component-export-memmem-12835.json";
const RECORD_SHA: &str = "a5de8cec59e1bb29d3220724016dfcfe8c08d84d8ab0d7b019a34d3aca14fbe7";

fn record() -> Value {
    let bytes = fs::read(support::repository().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["schema"], "vize.componentExportMemmemSource.v1");
    assert_eq!(
        record["credit"],
        json!({ "compiled": false, "native": false, "performance": false })
    );
    record
}

fn restore(current: String, file: &Value) -> String {
    assert_eq!(json!(current.len()), file["after"]["bytes"]);
    let mut previous = current;
    for hunk in file["hunks"].as_array().unwrap() {
        let after = hunk["after"].as_str().unwrap();
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, hunk["before"].as_str().unwrap(), 1);
    }
    assert_eq!(json!(previous.len()), file["before"]["bytes"]);
    assert_eq!(support::hash(previous.as_bytes()), file["before"]["sha256"]);
    previous
}

/// The exact pre-memmem bytes of a recorded producer. Older images pass through.
pub(super) fn source_before(path: &str, current: String) -> String {
    let record = record();
    match record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == path)
    {
        Some(file) if support::hash(current.as_bytes()) == file["after"]["sha256"] => {
            restore(current, file)
        }
        _ => current,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_producers_restore_their_exact_predecessors() {
        let record = record();
        for file in record["files"].as_array().unwrap() {
            let path = file["path"].as_str().unwrap();
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            assert_eq!(support::hash(current.as_bytes()), file["after"]["sha256"]);
            let previous = source_before(path, current.clone());
            assert_eq!(support::hash(previous.as_bytes()), file["before"]["sha256"]);
            assert_eq!(source_before(path, previous.clone()), previous);
            assert_eq!(source_before("other.rs", current.clone()), current);
            let first = file["hunks"][0]["after"].as_str().unwrap();
            for mutated in [
                format!("{current}{current}"),
                current.replacen(first, "", 1),
            ] {
                assert_eq!(source_before(path, mutated.clone()), mutated);
                assert!(std::panic::catch_unwind(|| restore(mutated, file)).is_err());
            }
        }
    }
}
