//! Invert synthesized listener return parity before older source custody.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "tests/_fixtures/tooling/listener-return-parity-12431.json";
const RECORD_SHA: &str = "a080316b111b872186a68e2dec2be3521ee73970ca5d65985a92c5038210cfb1";
const ASSET: &str =
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-listener-args.d.ts.txt";

fn record() -> Value {
    let bytes = fs::read(support::repository().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["schema"], "vize.listenerReturnParitySource.v1");
    assert_eq!(record["issue"], 12431);
    assert_eq!(
        record["credit"],
        json!({ "compiled": false, "native": false, "performance": false })
    );
    record
}

fn restore(mut current: String, file: &Value) -> String {
    assert_eq!(json!(current.len()), file["after"]["bytes"]);
    assert_eq!(
        support::hash(current.as_bytes()),
        file["after"]["sha256"].as_str().unwrap()
    );
    for hunk in file["hunks"].as_array().unwrap() {
        let after = hunk["after"].as_str().unwrap();
        assert_eq!(current.matches(after).count(), 1);
        current = current.replacen(after, hunk["before"].as_str().unwrap(), 1);
    }
    assert_eq!(json!(current.len()), file["before"]["bytes"]);
    assert_eq!(
        support::hash(current.as_bytes()),
        file["before"]["sha256"].as_str().unwrap()
    );
    current
}

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

pub(super) fn read_current(path: &str) -> String {
    source_before(
        path,
        fs::read_to_string(support::repository().join(path)).unwrap(),
    )
}

pub(super) fn preamble_before(current: &str) -> String {
    let record = record();
    let Some(preamble) = record["preambles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| support::hash(current.as_bytes()) == entry["after"]["sha256"])
    else {
        return current.to_owned();
    };
    assert_eq!(json!(current.len()), preamble["after"]["bytes"]);
    let asset = record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == ASSET)
        .unwrap();
    let mut previous = current.to_owned();
    for hunk in asset["hunks"].as_array().unwrap() {
        let after = hunk["after"].as_str().unwrap();
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, hunk["before"].as_str().unwrap(), 1);
    }
    assert_eq!(json!(previous.len()), preamble["before"]["bytes"]);
    assert_eq!(
        support::hash(previous.as_bytes()),
        preamble["before"]["sha256"].as_str().unwrap()
    );
    previous
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_sources_and_preambles_restore_their_exact_predecessors() {
        let record = record();
        for file in record["files"].as_array().unwrap() {
            let path = file["path"].as_str().unwrap();
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            assert_eq!(support::hash(current.as_bytes()), file["after"]["sha256"]);
            let previous = source_before(path, current.clone());
            assert_eq!(support::hash(previous.as_bytes()), file["before"]["sha256"]);
            assert_eq!(source_before(path, previous.clone()), previous);
        }
        for (current, expected) in [
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
            vize_canon::virtual_ts::SHARED_PREAMBLE_STRICT_DTS,
        ]
        .into_iter()
        .zip(record["preambles"].as_array().unwrap())
        {
            assert_eq!(
                support::hash(current.as_bytes()),
                expected["after"]["sha256"]
            );
            let previous = preamble_before(current);
            assert_eq!(
                support::hash(previous.as_bytes()),
                expected["before"]["sha256"]
            );
        }
    }
}
