use super::extract_slot_prop_names;

#[test]
fn extracts_slot_prop_names_with_ts_ast() {
    assert_eq!(
        extract_slot_prop_names("Readonly<{ foo: string; $bar?: number; 'not-valid': Date }>"),
        Some(vec!["foo".to_string(), "$bar".to_string()])
    );
}

#[test]
fn returns_none_for_non_object_slot_props() {
    assert_eq!(extract_slot_prop_names("Props"), None);
}
