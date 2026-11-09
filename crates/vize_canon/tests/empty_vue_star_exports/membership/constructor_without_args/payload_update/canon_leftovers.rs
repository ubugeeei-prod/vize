//! Invert only the #11499 helper and emit-overload successor.
use super::support;
use serde_json::{Value, json};

const RECEIPT_BYTES: &[u8] = include_bytes!("canon_leftovers_11499.json");
const RECEIPT_SHA: &str = "250e01138d163f77a5d98b20bc97f562a663ebb924c17c810b9d9cf02280128e";

fn identity(text: &str, expected: &Value) {
    assert_eq!(json!(text.len()), expected["bytes"]);
    assert_eq!(support::hash(text.as_bytes()), expected["sha256"]);
}

fn has_identity(text: &str, expected: &Value) -> bool {
    json!(text.len()) == expected["bytes"] && support::hash(text.as_bytes()) == expected["sha256"]
}

fn receipt() -> Value {
    assert_eq!(support::hash(RECEIPT_BYTES), RECEIPT_SHA);
    let receipt: Value = serde_json::from_slice(RECEIPT_BYTES).unwrap();
    assert_eq!(receipt["version"], 1);
    assert_eq!(receipt["kind"], "exact-canon-leftovers-source-inverse");
    assert_eq!(receipt["issue"], 11499);
    assert_eq!(receipt["nativeAcceptanceCredit"], false);
    assert_eq!(receipt["files"].as_array().unwrap().len(), 13);
    receipt
}

fn inverse(current: &str, transition: &Value) -> String {
    if has_identity(current, &transition["before"]) {
        return current.to_owned();
    }
    identity(current, &transition["after"]);
    let mut previous = current.to_owned();
    for edit in transition["edits"].as_array().unwrap() {
        let before = edit["before"].as_str().unwrap();
        let after = edit["after"].as_str().unwrap();
        assert!(!before.is_empty());
        assert!(!after.is_empty());
        assert_ne!(before, after);
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, before, 1);
    }
    identity(&previous, &transition["before"]);
    previous
}

#[cfg(test)]
fn forward(previous: &str, transition: &Value) -> String {
    identity(previous, &transition["before"]);
    let mut current = previous.to_owned();
    for edit in transition["edits"].as_array().unwrap().iter().rev() {
        let before = edit["before"].as_str().unwrap();
        let after = edit["after"].as_str().unwrap();
        assert_eq!(current.matches(before).count(), 1);
        current = current.replacen(before, after, 1);
    }
    identity(&current, &transition["after"]);
    current
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::instance_input_keys::source_before(path, current);
    let current = super::projection_text_buffer::source_before(path, current);
    let receipt = receipt();
    receipt["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|transition| transition["path"] == path)
        .map_or(current.clone(), |transition| inverse(&current, transition))
}

pub(super) fn preamble_before(current: &str) -> String {
    inverse(
        &super::instance_input_keys::preamble_before(current),
        &receipt()["compiledPreamble"],
    )
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    let receipt = receipt();
    for transition in receipt["files"].as_array().unwrap() {
        let path = transition["path"].as_str().unwrap();
        let current = super::instance_input_keys::source_before(
            path,
            super::projection_text_buffer::read_current(path),
        );
        identity(&current, &transition["after"]);
        inverse(&current, transition);
    }
    let preamble =
        super::instance_input_keys::preamble_before(vize_canon::virtual_ts::SHARED_PREAMBLE_DTS);
    identity(&preamble, &receipt["compiledPreamble"]["after"]);
    inverse(&preamble, &receipt["compiledPreamble"]);
    manifest["provenance"]["currentCanonLeftovers11499"] = receipt;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_sources_and_compiled_preamble_restore_the_exact_predecessors() {
        let receipt = receipt();
        for transition in receipt["files"].as_array().unwrap() {
            let path = transition["path"].as_str().unwrap();
            let current = super::super::instance_input_keys::source_before(
                path,
                super::super::projection_text_buffer::read_current(path),
            );
            let previous = inverse(&current, transition);
            assert_eq!(forward(&previous, transition), current);
        }
        let current = super::super::instance_input_keys::preamble_before(
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
        );
        let transition = &receipt["compiledPreamble"];
        let previous = inverse(&current, transition);
        assert_eq!(forward(&previous, transition), current);
    }

    #[test]
    fn missing_duplicate_stale_and_foreign_bytes_are_refused() {
        let receipt = receipt();
        for transition in receipt["files"]
            .as_array()
            .unwrap()
            .iter()
            .chain(std::iter::once(&receipt["compiledPreamble"]))
        {
            let current = if transition["path"] == "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS" {
                super::super::instance_input_keys::preamble_before(
                    vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
                )
            } else {
                let path = transition["path"].as_str().unwrap();
                super::super::instance_input_keys::source_before(
                    path,
                    super::super::projection_text_buffer::read_current(path),
                )
            };
            for mutated in [
                format!("{current}\n"),
                format!("{current}{current}"),
                current.replacen("type ", "interface ", 1),
            ] {
                assert_ne!(mutated, current);
                assert!(std::panic::catch_unwind(|| inverse(&mutated, transition)).is_err());
            }
        }
    }
}
