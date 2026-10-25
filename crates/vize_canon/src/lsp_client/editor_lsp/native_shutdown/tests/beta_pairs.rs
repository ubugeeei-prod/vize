use super::super::known_provider_pair;
use serde_json::{Value, json};

#[test]
fn exact_beta_images_require_the_complete_original_initialize() {
    let rows: Vec<Value> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/native-beta-provider-pairs-10741/provider-pairs.json"
    )))
    .unwrap();
    assert_eq!(rows.len(), 17);
    for row in rows {
        let input = row.get("input").unwrap();
        let admitted = known_provider_pair(
            input.get("imageSha256").unwrap().as_str().unwrap(),
            input.get("initialize").unwrap().get("result").unwrap(),
        );
        assert_eq!(
            json!({"kind": if admitted {"admitted"} else {"rejected"}}),
            *row.get("expected").unwrap(),
            "whole captured beta provider pair: {}",
            row.get("id").unwrap()
        );
    }
}
