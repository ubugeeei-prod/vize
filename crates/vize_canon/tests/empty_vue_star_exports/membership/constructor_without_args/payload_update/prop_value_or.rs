//! Invert the lazy fallthrough fallback `$PropValueOr` (#12431) before older helper custody.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "tests/_fixtures/tooling/prop-value-or-12431.json";
const RECORD_SHA: &str = "3d59c50e8efb0b0b983302d422397af84a830a4b8ab2477886e91f0520ab9c6b";

fn record() -> Value {
    let bytes = fs::read(support::repository().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["schema"], "vize.propValueOrSource.v1");
    assert_eq!(record["issue"], 12431);
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

fn restore_if_current(current: String, file: &Value) -> String {
    if support::hash(current.as_bytes()) == file["after"]["sha256"] {
        restore(current, file)
    } else {
        current
    }
}

/// The pre-rewrite bytes of a recorded helper's current image. Older images
/// pass through; the layers below authenticate them.
pub(super) fn source_before(path: &str, current: String) -> String {
    let record = record();
    match record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == path)
    {
        Some(file) => restore_if_current(current, file),
        None => current,
    }
}

/// The compiled shared preamble with this layer removed.
pub(super) fn preamble_before(current: &str) -> String {
    restore_if_current(current.to_owned(), &record()["compiledPreamble"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_helpers_and_compiled_preamble_restore_their_exact_predecessors() {
        let record = record();
        let physical = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS.to_owned();
        let mut images: Vec<(&Value, String)> = record["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|file| {
                let path = file["path"].as_str().unwrap();
                (
                    file,
                    fs::read_to_string(support::repository().join(path)).unwrap(),
                )
            })
            .collect();
        images.push((&record["compiledPreamble"], physical));
        for (file, current) in images {
            assert_eq!(support::hash(current.as_bytes()), file["after"]["sha256"]);
            let previous = restore_if_current(current.clone(), file);
            assert_eq!(support::hash(previous.as_bytes()), file["before"]["sha256"]);
            assert_eq!(restore_if_current(previous.clone(), file), previous);
            let first = file["hunks"][0]["after"].as_str().unwrap();
            for mutated in [
                format!("{current}{current}"),
                current.replacen(first, "", 1),
            ] {
                assert_eq!(restore_if_current(mutated.clone(), file), mutated);
                assert!(std::panic::catch_unwind(|| restore(mutated, file)).is_err());
            }
        }
    }
}
