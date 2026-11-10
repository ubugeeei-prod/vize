//! Invert the generated-comment byte cut (debug-only `@vize-map` comments and
//! the reasons after `@ts-ignore`) before older emitter custody.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "tests/_fixtures/tooling/generated-comment-bytes.json";
const RECORD_SHA: &str = "e0f9efc86d1d2ac94b84bcace95a209b4e0aa0c2d1e3f505fd25d96c010c4381";

fn record() -> Value {
    let bytes = fs::read(support::repository().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["schema"], "vize.generatedCommentBytesSource.v1");
    assert_eq!(
        record["credit"],
        json!({ "compiled": false, "native": false, "performance": false })
    );
    record
}

fn is_after(current: &str, transition: &Value) -> bool {
    json!(current.len()) == transition["after"]["bytes"]
        && support::hash(current.as_bytes()) == transition["after"]["sha256"]
}

fn restore(current: &str, transition: &Value) -> String {
    assert!(is_after(current, transition));
    let mut previous = current.to_owned();
    for hunk in transition["hunks"].as_array().unwrap() {
        let after = hunk["after"].as_str().unwrap();
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, hunk["before"].as_str().unwrap(), 1);
    }
    assert_eq!(json!(previous.len()), transition["before"]["bytes"]);
    assert_eq!(
        support::hash(previous.as_bytes()),
        transition["before"]["sha256"]
    );
    previous
}

/// The producer bytes before the generated-comment cut. Older images pass
/// through; the layers below authenticate them.
pub(super) fn source_before(path: &str, current: String) -> String {
    let record = record();
    match record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == path)
    {
        Some(file) if is_after(&current, file) => restore(&current, file),
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
            assert!(is_after(&current, file), "{path}");
            let previous = source_before(path, current.clone());
            assert_eq!(support::hash(previous.as_bytes()), file["before"]["sha256"]);
            assert_eq!(source_before(path, previous.clone()), previous);
            let first = file["hunks"][0]["after"].as_str().unwrap();
            for mutated in [
                format!("{current}{current}"),
                current.replacen(first, "", 1),
            ] {
                assert_eq!(source_before(path, mutated.clone()), mutated);
                assert!(std::panic::catch_unwind(|| restore(&mutated, file)).is_err());
            }
        }
    }
}
