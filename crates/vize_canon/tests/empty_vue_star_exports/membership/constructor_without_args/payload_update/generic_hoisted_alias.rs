//! Invert the hoisted generic alias forwarding before older custody.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "tests/_fixtures/tooling/generic-hoisted-alias-forwarding.json";
const RECORD_SHA: &str = "2d9612946f753c1507d13cc1adfe958dde3bc6b620b9d0abd652de92e34dd08f";

fn record() -> Value {
    let bytes = fs::read(support::repository().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["schema"], "vize.genericHoistedAliasSource.v1");
    assert_eq!(
        record["credit"],
        json!({ "compiled": false, "native": false, "performance": false })
    );
    record
}

fn is_after(current: &str, file: &Value) -> bool {
    json!(current.len()) == file["after"]["bytes"]
        && support::hash(current.as_bytes()) == file["after"]["sha256"]
}

fn restore(current: &str, file: &Value) -> String {
    assert!(is_after(current, file));
    let mut previous = current.to_owned();
    for hunk in file["hunks"].as_array().unwrap() {
        let after = hunk["after"].as_str().unwrap();
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, hunk["before"].as_str().unwrap(), 1);
    }
    assert_eq!(json!(previous.len()), file["before"]["bytes"]);
    assert_eq!(support::hash(previous.as_bytes()), file["before"]["sha256"]);
    previous
}

/// The exact pre-forwarding bytes of a recorded producer. Older images pass
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
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            assert!(is_after(&current, file), "{path}");
            let previous = read_current(path);
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
