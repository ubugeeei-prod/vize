//! Invert the component hover producer wiring (#12461) before older custody.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "tests/_fixtures/tooling/component-hover-gaps-12461.json";
const RECORD_SHA: &str = "c8ccbc39b13f6b947115e56be72bceed09592662f4fa6cb0f08751bbe51634ee";

fn record() -> Value {
    let bytes = fs::read(support::repository().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["schema"], "vize.componentHoverGapsSource.v1");
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

/// The exact pre-#12461 bytes of a recorded producer. Older images pass through.
pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::component_export_memmem::source_before(path, current);
    let current = super::check_mode_generator::source_before(path, current);
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

/// A repository source with this newest layer removed.
pub(super) fn read_current(path: &str) -> String {
    source_before(
        path,
        fs::read_to_string(support::repository().join(path)).unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_producers_restore_their_exact_predecessors() {
        let record = record();
        for file in record["files"].as_array().unwrap() {
            let path = file["path"].as_str().unwrap();
            let current = super::super::check_mode_generator::source_before(
                path,
                super::super::component_export_memmem::source_before(
                    path,
                    fs::read_to_string(support::repository().join(path)).unwrap(),
                ),
            );
            assert_eq!(support::hash(current.as_bytes()), file["after"]["sha256"]);
            let previous = source_before(path, current.clone());
            assert_eq!(support::hash(previous.as_bytes()), file["before"]["sha256"]);
            assert_eq!(source_before(path, previous.clone()), previous);
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
