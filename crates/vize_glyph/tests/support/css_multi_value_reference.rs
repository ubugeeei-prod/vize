//! The same reviewed #7968 authority used by the public CLI/browser controls.
use serde_json::Value;
use sha2::{Digest, Sha256};

const AUTHORITY: &str = include_str!(
    "../../../../tests/_fixtures/differential/formatter-regressions/css-multi-value-7968/current-references.json"
);

fn digest(bytes: &[u8]) -> std::string::String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn current_reference(
    owner: &str,
    corpus: &[u8],
    fixture: &Value,
    input: &str,
    historical: &str,
) -> Option<std::string::String> {
    assert_eq!(
        digest(AUTHORITY.as_bytes()),
        "8e53853840d474c0d9cd6a29eaa0ab061a8e3e46df7f1c45fd82d705e0a6220c",
        "reviewed complete current authority changed"
    );
    let authority: Value = serde_json::from_str(AUTHORITY).expect("reviewed current authority");
    assert_eq!(
        authority["schema"],
        "vize.css-multi-value-current-references"
    );
    assert_eq!(authority["version"], 1);
    assert_eq!(authority["issue"], 7968);
    let rows = authority["cases"].as_array().expect("current rows");
    assert_eq!(rows.len(), 12);
    let owner_row = rows
        .iter()
        .find(|row| row["owner"] == owner)
        .expect("known original owner");
    assert_eq!(
        digest(corpus),
        owner_row["corpusSha256"].as_str().unwrap(),
        "whole original corpus"
    );
    let row = rows
        .iter()
        .find(|row| row["owner"] == owner && row["id"] == fixture["id"])?;
    assert_eq!(
        digest(input.as_bytes()),
        row["inputSha256"].as_str().unwrap(),
        "original input"
    );
    assert_eq!(
        digest(historical.as_bytes()),
        row["historicalExpectedSha256"].as_str().unwrap(),
        "original output"
    );
    let empty = serde_json::json!({});
    assert_eq!(fixture.get("options").unwrap_or(&empty), &row["options"]);
    // The whole corpus hash above also owns every original CLI action and stream.
    let current = row["currentExpected"]
        .as_str()
        .expect("whole current output");
    assert_eq!(
        digest(current.as_bytes()),
        row["currentExpectedSha256"].as_str().unwrap()
    );
    assert_ne!(
        historical, current,
        "current equality cannot replace old mismatch"
    );
    Some(current.to_owned())
}
