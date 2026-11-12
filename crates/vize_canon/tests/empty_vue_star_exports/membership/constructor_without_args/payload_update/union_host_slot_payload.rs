//! Invert the union-host slot payload (`$NamedSlotPayload` distributes over
//! the host's slot contract) before older helper custody.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "tests/_fixtures/tooling/union-host-slot-payload.json";
const RECORD_SHA: &str = "b5fc83b7d0cdfbf82f45dac0bf04674c88becb6050e16107dc26adc9eefed085";

fn record() -> Value {
    let bytes = fs::read(support::repository().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["schema"], "vize.unionHostSlotPayloadSource.v1");
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

/// The intersecting-payload bytes of a recorded asset's current image. Older
/// images pass through; the layers below authenticate them.
pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::generic_hoisted_alias::source_before(path, current);
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

/// A compiled preamble with the intersecting slot payload.
pub(super) fn preamble_before(current: &str) -> String {
    let record = record();
    match ["compiledPreamble", "compiledStrictPreamble"]
        .into_iter()
        .map(|key| &record[key])
        .find(|preamble| is_after(current, preamble))
    {
        Some(preamble) => restore(current, preamble),
        None => current.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_assets_and_preamble_restore_their_exact_predecessors() {
        let record = record();
        for file in record["files"].as_array().unwrap() {
            let path = file["path"].as_str().unwrap();
            let current = super::super::generic_hoisted_alias::read_current(path);
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
        for (physical, key) in [
            (
                vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
                "compiledPreamble",
            ),
            (
                vize_canon::virtual_ts::SHARED_PREAMBLE_STRICT_DTS,
                "compiledStrictPreamble",
            ),
        ] {
            let preamble = &record[key];
            assert!(is_after(physical, preamble));
            let previous = preamble_before(physical);
            assert_eq!(
                support::hash(previous.as_bytes()),
                preamble["before"]["sha256"]
            );
            assert_eq!(preamble_before(&previous), previous);
            assert!(
                std::panic::catch_unwind(|| restore(&format!("{physical}\n"), preamble)).is_err()
            );
        }
    }
}
