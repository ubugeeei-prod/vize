//! One observed factory-only insertion, before the unchanged C3 inverse.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/ref-factory-update-20261031.json";
const SOURCE: &str = "crates/vize_canon/src/virtual_ts/helpers/preamble.rs";

pub(super) fn load() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(
        support::hash(&bytes),
        "43717139ecb09bdfe6f9d44613807c69644e2efb41536cb3651663e6ac548643"
    );
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], "exact-ref-factory-source-preamble-inverse");
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["projectionAcceptanceCredit"], false);
    assert_eq!(
        record["sourceRevision"],
        "3091e1e4aa7209b6a0c82374bece3dc59bdf84b2"
    );
    assert_eq!(
        record["compiledCapture"]["executionRevision"],
        "28a4dff5c4b9c9b3a1801856934b15993b5d2e85"
    );
    assert_eq!(
        record["compiledCapture"]["tree"],
        "2ffb4d697de75c1b48b48fda1c1707854e1dad42"
    );
    assert_eq!(record["compiledCapture"]["cliStatus"], 0);
    let frozen = &record["frozenC3"];
    let original = fs::read(support::repository().join(frozen["path"].as_str().unwrap())).unwrap();
    assert_eq!(support::hash(&original), frozen["sha256"].as_str().unwrap());
    let original: Value = serde_json::from_slice(&original).unwrap();
    assert_eq!(
        record["preamble"]["beforeWholeUtf8"],
        original["logicalPreamble"]["afterWholeUtf8"]
    );
    assert_eq!(record["source"]["path"], SOURCE);
    let declaration = &record["declaration"];
    assert_eq!(
        declaration["path"],
        "crates/vize_canon/src/virtual_ts/helpers/declarations/ref-instance-factory.d.ts.txt"
    );
    let actual =
        fs::read(support::repository().join(declaration["path"].as_str().unwrap())).unwrap();
    assert_eq!(
        actual,
        declaration["wholeUtf8"].as_str().unwrap().as_bytes()
    );
    assert_eq!(json!(actual.len()), declaration["bytes"]);
    assert_eq!(
        support::hash(&actual),
        declaration["sha256"].as_str().unwrap()
    );
    record
}

fn previous(current: &str, record: &Value) -> String {
    assert_eq!(current, record["afterWholeUtf8"].as_str().unwrap());
    assert_eq!(json!(current.len()), record["afterBytes"]);
    assert_eq!(
        support::hash(current.as_bytes()),
        record["afterSha256"].as_str().unwrap()
    );
    let inserted = record["insertionWholeUtf8"].as_str().unwrap();
    assert!(!inserted.is_empty());
    assert_eq!(json!(inserted.len()), record["insertionBytes"]);
    assert_eq!(
        support::hash(inserted.as_bytes()),
        record["insertionSha256"].as_str().unwrap()
    );
    assert_eq!(current.matches(inserted).count(), 1);
    let before = current.replacen(inserted, "", 1);
    assert_eq!(before, record["beforeWholeUtf8"].as_str().unwrap());
    assert_eq!(json!(before.len()), record["beforeBytes"]);
    assert_eq!(
        support::hash(before.as_bytes()),
        record["beforeSha256"].as_str().unwrap()
    );
    before
}

pub(super) fn source_before(path: &str, current: &str) -> String {
    if path == SOURCE {
        previous(current, &load()["source"])
    } else {
        current.to_owned()
    }
}

pub(super) fn preamble_before(current: &str) -> String {
    previous(current, &load()["preamble"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genuine_factory_preamble_and_source_reconstruct_exact_prior_bodies() {
        let record = load();
        assert_eq!(
            preamble_before(vize_canon::virtual_ts::SHARED_PREAMBLE_DTS),
            record["preamble"]["beforeWholeUtf8"].as_str().unwrap()
        );
        let actual = fs::read_to_string(support::repository().join(SOURCE)).unwrap();
        assert_eq!(
            source_before(SOURCE, &actual),
            record["source"]["beforeWholeUtf8"].as_str().unwrap()
        );
    }

    #[test]
    fn unknown_factory_preamble_bytes_are_not_an_allowance() {
        let record = load();
        let body = record["preamble"]["afterWholeUtf8"].as_str().unwrap();
        let block = record["preamble"]["insertionWholeUtf8"].as_str().unwrap();
        for changed in [
            body.replacen("__vizeResolveRef", "__foreignResolveRef", 1),
            body.replacen(block, "", 1),
            [body, block].concat(),
            [body, "\n// foreign\n"].concat(),
        ] {
            assert!(std::panic::catch_unwind(|| previous(&changed, &record["preamble"])).is_err());
        }
    }
}
