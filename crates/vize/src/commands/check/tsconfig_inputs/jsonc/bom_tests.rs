//! Shared whole JSONC controls for the two existing config readers.
use super::parse_jsonc_value;
use serde_json::Value;
use vize_carton::cstr;

const CONTROLS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-bom-3984/parser-controls.json"
));

#[test]
fn utf8_bom_keeps_complete_options_unknown_fields_and_string_data() {
    let controls: Value = serde_json::from_str(CONTROLS).unwrap();
    let plain = controls["plain"].as_str().unwrap();
    assert_eq!(parse_jsonc_value(plain).unwrap(), controls["expected"]);
    let bom = cstr!("\u{feff}{plain}");
    assert_eq!(parse_jsonc_value(&bom).unwrap(), controls["expected"]);
}

#[test]
fn malformed_plain_and_bom_configs_keep_failure() {
    let controls: Value = serde_json::from_str(CONTROLS).unwrap();
    for invalid in controls["invalid"].as_array().unwrap() {
        let invalid = invalid.as_str().unwrap();
        assert!(parse_jsonc_value(invalid).is_err());
        assert!(parse_jsonc_value(&cstr!("\u{feff}{invalid}")).is_err());
    }
}

#[test]
fn only_one_byte_zero_bom_is_ignored_and_empty_content_keeps_failure() {
    assert!(parse_jsonc_value("").is_err());
    assert!(parse_jsonc_value("\u{feff}").is_err());
    assert!(parse_jsonc_value("\u{feff}\u{feff}{}").is_err());
    assert!(parse_jsonc_value(" \u{feff}{}").is_err());
}
