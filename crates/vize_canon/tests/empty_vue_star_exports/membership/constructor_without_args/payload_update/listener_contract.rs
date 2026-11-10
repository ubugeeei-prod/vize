//! Invert the shared listener contract before older emitter and SDK custody.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "tests/_fixtures/tooling/listener-contract-source.json";
const RECORD_SHA: &str = "cafa40d3371b70f521b218afd628431e671625266aca9d310d02013c4b765be5";
const ASSET: &str =
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-listener-args.d.ts.txt";
/// The compiled preambles before and after the asset's contract lines.
const PREAMBLE: [(&str, &str); 2] = [
    (
        "f99aa98799587d3f65f3c343deb75846d8a37b86c9fe18cc4cd29cce8f929d22",
        "46f6322da31954de7b764473f58164a3d4d18e499c2d4fdbf953a5911bfc4633",
    ),
    (
        "451e8d0ab8fcd0bd257d88026e87b460262e3cb5501fd839a167f811f4f182a0",
        "d0bf925c109244866118d44c9a005559a0f307739978884e1b43ac08a516771d",
    ),
];

fn record() -> Value {
    let bytes = fs::read(support::repository().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["schema"], "vize.listenerContractSource.v1");
    assert_eq!(
        record["credit"],
        json!({ "compiled": false, "native": false, "performance": false })
    );
    record
}

fn invert(current: &str, hunks: &[Value]) -> String {
    let mut previous = current.to_owned();
    for hunk in hunks {
        let after = hunk["after"].as_str().unwrap();
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, hunk["before"].as_str().unwrap(), 1);
    }
    previous
}

fn restore(current: String, file: &Value) -> String {
    assert_eq!(json!(current.len()), file["after"]["bytes"]);
    let previous = invert(&current, file["hunks"].as_array().unwrap());
    assert_eq!(json!(previous.len()), file["before"]["bytes"]);
    assert_eq!(support::hash(previous.as_bytes()), file["before"]["sha256"]);
    previous
}

/// The pre-contract bytes of a recorded source's current image. Older images
/// pass through; the layers below authenticate them.
pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::lib_type_qualification_listeners::source_before(path, current);
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

/// A repository source with this layer removed.
pub(super) fn read_current(path: &str) -> String {
    source_before(
        path,
        fs::read_to_string(support::repository().join(path)).unwrap(),
    )
}

/// The compiled preamble without the contract lines its asset gained.
pub(super) fn preamble_before(current: &str) -> String {
    let current = super::component_boxes::preamble_before(current);
    let hash = support::hash(current.as_bytes());
    let Some((_, before)) = PREAMBLE.iter().find(|(after, _)| *after == hash) else {
        return current;
    };
    let record = record();
    let asset = record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == ASSET)
        .unwrap();
    let previous = invert(&current, asset["hunks"].as_array().unwrap());
    assert_eq!(support::hash(previous.as_bytes()), *before);
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
            let current = super::super::lib_type_qualification_listeners::read_current(path);
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
        for (current, (after, before)) in [
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
            vize_canon::virtual_ts::SHARED_PREAMBLE_STRICT_DTS,
        ]
        .into_iter()
        .zip(PREAMBLE)
        {
            let current = &super::super::component_boxes::preamble_before(current);
            assert_eq!(support::hash(current.as_bytes()), after);
            let previous = preamble_before(current);
            assert_eq!(support::hash(previous.as_bytes()), before);
            assert_eq!(preamble_before(&previous), previous);
        }
    }
}
