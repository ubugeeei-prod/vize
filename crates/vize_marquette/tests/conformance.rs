use std::{fs, path::PathBuf};

use vize_marquette::{ApplicationContract, canonical_json, contract_fingerprint};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/marquette")
        .join(name)
}

#[test]
fn accepts_the_shared_valid_contract_and_matches_canonical_artifacts() {
    let contract: ApplicationContract =
        serde_json::from_slice(&fs::read(fixture("valid.json")).unwrap()).unwrap();
    assert!(contract.validate().is_empty());

    let canonical = canonical_json(&contract).unwrap();
    let expected = fs::read_to_string(fixture("valid.canonical")).unwrap();
    assert_eq!(canonical.as_slice(), expected.trim().as_bytes());
    assert_eq!(
        contract_fingerprint(&contract).unwrap(),
        fs::read_to_string(fixture("valid.sha256")).unwrap().trim()
    );
}

#[test]
fn returns_the_shared_invalid_diagnostic_codes() {
    let contract: ApplicationContract =
        serde_json::from_slice(&fs::read(fixture("invalid.json")).unwrap()).unwrap();
    let actual = contract
        .validate()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    let expected: Vec<vize_s0::String> =
        serde_json::from_slice(&fs::read(fixture("invalid.expected.json")).unwrap()).unwrap();
    assert_eq!(actual, expected);
}
