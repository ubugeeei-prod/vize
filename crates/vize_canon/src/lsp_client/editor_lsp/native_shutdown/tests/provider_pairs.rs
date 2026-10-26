use super::super::{DARWIN_ARM64, LINUX_X64, MAPPER_LINUX_X64, known_provider_pair};
use serde_json::{Value, json};

#[test]
fn admission_matches_pinned_complete_provider_pairs() {
    let rows: Vec<Value> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/native-provider-pairs-10599/provider-pairs.json"
    )))
    .unwrap();
    assert_eq!(rows.len(), 14);
    for row in &rows {
        let input = &row["input"];
        let admitted = known_provider_pair(
            input["imageSha256"].as_str().unwrap(),
            &input["initialize"]["result"],
        );
        assert_eq!(
            json!({"kind": if admitted { "admitted" } else { "rejected" }}),
            row["expected"],
            "complete captured Initialize and physical image pair: {}",
            row["id"]
        );
    }
    // A product version, partial digest or borrowed server identity is never
    // authority for an unobserved physical executable.
    for digest in [
        "",
        "77dd575e",
        &"f".repeat(64),
        MAPPER_LINUX_X64.trim_end_matches('b'),
    ] {
        for initialized in [
            super::super::known_contract(),
            super::super::mapper_contract::initialize(),
        ] {
            assert!(!known_provider_pair(digest, &initialized));
        }
    }
    assert_ne!(LINUX_X64, MAPPER_LINUX_X64);
    assert_ne!(DARWIN_ARM64, MAPPER_LINUX_X64);
}
