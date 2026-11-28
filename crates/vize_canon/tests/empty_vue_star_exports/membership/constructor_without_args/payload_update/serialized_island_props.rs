//! Invert only the authenticated #13820 guard before frozen helper history.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "tests/_fixtures/tooling/serialized-island-props-13820.json";
const RECORD_SHA: &str = "7fdc992cd4e36af75c849cf4c4aa8f4928a6a57823a1cf264c32e31482d9c174";

fn record() -> Value {
    let bytes = fs::read(support::repository().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["schema"], "vize.serializedIslandPropsSource.v1");
    assert_eq!(record["issue"], 13820);
    assert_eq!(record["historicalReconstructionOnly"], true);
    assert_eq!(
        record["credit"],
        json!({"compiled":false,"native":false,"performance":false})
    );
    record
}

fn is_image(current: &str, identity: &Value) -> bool {
    json!(current.len()) == identity["bytes"]
        && support::hash(current.as_bytes()) == identity["sha256"]
}

fn restore(current: &str, transition: &Value) -> String {
    assert!(is_image(current, &transition["after"]));
    let mut previous = current.to_owned();
    for hunk in transition["hunks"].as_array().unwrap() {
        let after = hunk["after"].as_str().unwrap();
        assert!(!after.is_empty());
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, hunk["before"].as_str().unwrap(), 1);
    }
    assert!(is_image(&previous, &transition["before"]));
    previous
}

/// Older images remain owned by their unchanged historical layers.
pub(super) fn source_before(path: &str, current: String) -> String {
    let record = record();
    match record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == path)
    {
        Some(entry) if is_image(&current, &entry["after"]) => restore(&current, entry),
        _ => current,
    }
}

pub(super) fn preamble_before(current: &str) -> String {
    let record = record();
    match record["compiledPreambles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| is_image(current, &entry["after"]))
    {
        Some(entry) => restore(current, entry),
        None => current.to_owned(),
    }
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["serializedIslandProps"] = record();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_current_sources_and_both_preambles_restore_without_new_credit() {
        let record = record();
        if let Some(path) = std::env::var_os("VIZE_CANON_CAPTURE_ISLAND_PREAMBLES") {
            fs::write(
                path,
                serde_json::to_vec(&[
                    vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
                    vize_canon::virtual_ts::SHARED_PREAMBLE_STRICT_DTS,
                ])
                .unwrap(),
            )
            .unwrap();
        }
        let mut images: Vec<(&Value, String)> = record["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                let path = entry["path"].as_str().unwrap();
                (
                    entry,
                    fs::read_to_string(support::repository().join(path)).unwrap(),
                )
            })
            .collect();
        images.extend(
            record["compiledPreambles"]
                .as_array()
                .unwrap()
                .iter()
                .zip([
                    vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
                    vize_canon::virtual_ts::SHARED_PREAMBLE_STRICT_DTS,
                ])
                .map(|(entry, current)| (entry, current.to_owned())),
        );
        assert_eq!(images.len(), 4);
        for (entry, current) in images {
            let previous = restore(&current, entry);
            let edit = &entry["hunks"][0];
            let after = edit["after"].as_str().unwrap();
            let mut roundtrip = previous.clone();
            for hunk in entry["hunks"].as_array().unwrap() {
                let before = hunk["before"].as_str().unwrap();
                assert!(!before.is_empty());
                assert_eq!(roundtrip.matches(before).count(), 1);
                roundtrip = roundtrip.replacen(before, hunk["after"].as_str().unwrap(), 1);
            }
            assert_eq!(roundtrip, current);
            for malformed in [
                String::new(),
                format!("{current}{current}"),
                current.replacen(after, "", 1),
                previous,
                format!("{current}\n"),
            ] {
                assert!(std::panic::catch_unwind(|| restore(&malformed, entry)).is_err());
            }
        }
    }
}
