//! Authenticate the new shared helper before reconstructing frozen source history.
use super::support;
use serde_json::{Value, json};
use std::fs;

const SOURCE: &str =
    "crates/vize_canon/src/virtual_ts/helpers/declarations/component-checks.d.ts.txt";
const RECORD: &str = "malformed-membership/instance-prop-metadata-update.json";

fn load() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(
        support::hash(&bytes),
        "b7dea15af178ef4594fc42e764a6875903d0861850065cfa958c392cfd8b9792"
    );
    let update: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(update["version"], 1);
    assert_eq!(
        update["kind"],
        "exact-instance-prop-metadata-source-inverse"
    );
    assert_eq!(update["nativeAcceptanceCredit"], false);
    assert_eq!(update["source"]["path"], SOURCE);
    assert_eq!(
        update["preamble"]["path"],
        "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS"
    );
    assert_eq!(
        update["logicalPreambleRole"],
        "source-composition-prediction-to-be-checked-against-compiled-constant-by-native-tests"
    );
    assert_eq!(
        update["frozenPayloadRecord"]["sha256"],
        "0a71c5c21535e50d02df527e6c00267dfd8b44bfc0d63c37c49a55d67ab1166c"
    );
    update
}

fn previous(current: &str, record: &Value) -> String {
    assert_eq!(json!(current.len()), record["afterBytes"]);
    assert_eq!(support::hash(current.as_bytes()), record["afterSha256"]);
    assert_eq!(current, record["afterWholeUtf8"].as_str().unwrap());
    let edits = record["edits"].as_array().unwrap();
    assert_eq!(edits.len(), 2);
    let mut before = current.to_owned();
    for edit in edits {
        let old = edit["before"].as_str().unwrap();
        let new = edit["after"].as_str().unwrap();
        assert_ne!(old, new);
        assert!(!new.is_empty());
        assert_eq!(before.matches(new).count(), 1);
        before = before.replacen(new, old, 1);
    }
    assert_eq!(before, record["beforeWholeUtf8"].as_str().unwrap());
    assert_eq!(json!(before.len()), record["beforeBytes"]);
    assert_eq!(support::hash(before.as_bytes()), record["beforeSha256"]);
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

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentInstancePropMetadataUpdate"] = load();
}

#[test]
fn complete_current_source_and_compiled_preamble_reconstruct_frozen_payload() {
    let update = load();
    let source = fs::read_to_string(support::repository().join(SOURCE)).unwrap();
    assert_eq!(
        source_before(SOURCE, &source),
        update["source"]["beforeWholeUtf8"].as_str().unwrap()
    );
    assert_eq!(
        preamble_before(vize_canon::virtual_ts::SHARED_PREAMBLE_DTS),
        update["preamble"]["beforeWholeUtf8"].as_str().unwrap()
    );
    assert_eq!(source_before("unrelated/source", "unchanged"), "unchanged");
}

#[test]
fn missing_duplicate_unknown_extra_and_forged_source_bodies_are_refused() {
    let update = load();
    for record in [&update["source"], &update["preamble"]] {
        let current = record["afterWholeUtf8"].as_str().unwrap();
        for changed in [
            String::new(),
            format!("{current}{current}"),
            format!("{current}\n"),
            current.replacen("readonly present:", "present:", 1),
            record["beforeWholeUtf8"].as_str().unwrap().to_owned(),
        ] {
            assert_ne!(changed, current);
            assert!(std::panic::catch_unwind(|| previous(&changed, record)).is_err());
        }
        for field in [
            "beforeBytes",
            "afterBytes",
            "beforeSha256",
            "afterSha256",
            "beforeWholeUtf8",
            "afterWholeUtf8",
            "edits",
        ] {
            let mut changed = record.clone();
            changed[field] = json!("unknown");
            assert!(std::panic::catch_unwind(|| previous(current, &changed)).is_err());
        }
    }
}
