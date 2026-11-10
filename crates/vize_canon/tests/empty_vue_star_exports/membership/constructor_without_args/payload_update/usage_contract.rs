//! Invert the shared usage contract before older emitter and SDK custody.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "tests/_fixtures/tooling/usage-contract-12431.json";
const RECORD_SHA: &str = "f085f1bd6949db53fea8b0af056c7841cd6dd90dd27ad4bb5bd3c316014ac0c6";
const ASSET: &str =
    "crates/vize_canon/src/virtual_ts/helpers/declarations/component-checks.d.ts.txt";
/// The compiled preambles after and before the asset's usage-contract lines.
const PREAMBLE: [(&str, &str); 2] = [
    (
        "ddc460b5e3157eba9774376acd38ad4f0007ab62413a12688b0cf06eaf0a9d2b",
        "f99aa98799587d3f65f3c343deb75846d8a37b86c9fe18cc4cd29cce8f929d22",
    ),
    (
        "adfbb735a3891196f7c1162d9212bee0abac5ad25e092de3d92b183488d8ccfb",
        "451e8d0ab8fcd0bd257d88026e87b460262e3cb5501fd839a167f811f4f182a0",
    ),
];

fn record() -> Value {
    let bytes = fs::read(support::repository().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["schema"], "vize.usageContractSource.v1");
    assert_eq!(record["issue"], 12431);
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

/// The compiled preamble without the usage-contract lines its asset gained.
pub(super) fn preamble_before(current: &str) -> String {
    let hash = support::hash(current.as_bytes());
    let Some((_, before)) = PREAMBLE.iter().find(|(after, _)| *after == hash) else {
        return current.to_owned();
    };
    let record = record();
    let asset = record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == ASSET)
        .unwrap();
    let previous = invert(current, asset["hunks"].as_array().unwrap());
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
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
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
            assert_eq!(support::hash(current.as_bytes()), after);
            let previous = preamble_before(current);
            assert_eq!(support::hash(previous.as_bytes()), before);
            assert_eq!(preamble_before(&previous), previous);
        }
    }
}
