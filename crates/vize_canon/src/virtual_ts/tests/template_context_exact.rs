use super::*;

#[test]
fn setup_helpers_use_runtime_macro_types() {
    assert_virtual_ts_snapshot("virtual_ts_vue_setup_helpers", VUE_SETUP_HELPERS);
}

#[test]
fn vue3_default_excludes_legacy_members() {
    let v3 = template_context(&VirtualTsOptions::default(), VueVersion::V3);
    assert_eq!(v3.matches("$listeners").count(), 0);
    assert_eq!(v3.matches("$children").count(), 0);
    assert!(!v3.contains("$scopedSlots"));
    assert!(!v3.contains("$createElement"));
    assert!(!v3.contains("Vue 2 instance members"));
}
