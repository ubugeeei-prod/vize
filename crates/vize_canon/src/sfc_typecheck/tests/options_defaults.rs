use super::*;

#[test]
fn test_type_check_options_default() {
    let options = SfcTypeCheckOptions::new("test.vue");
    assert_eq!(options.filename, "test.vue");
    assert!(!options.strict);
    assert!(options.check_props);
    assert!(options.check_emits);
    assert!(options.check_template_bindings);
    assert!(!options.include_virtual_ts);
}
