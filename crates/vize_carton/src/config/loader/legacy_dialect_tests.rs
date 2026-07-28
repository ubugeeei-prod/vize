//! A Vue 2 / 2.7 dialect implies legacy template lowering (#3297).
//!
//! `typeChecker.legacyVue2` gates slot-scope and filter lowering. Deriving it
//! from the dialect keeps `vize check` and `vize lsp` agreeing with each other
//! and with the compiler, which already treats `vue.version` 2/2.7 as legacy.

use super::load_config_with_features_and_source;
use crate::config::VueVersion;

fn features_for(config_json: &str) -> crate::config::ConfigFeatureFlags {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vize.config.json");
    std::fs::write(&path, config_json).unwrap();
    load_config_with_features_and_source(Some(&path)).features
}

#[test]
fn vue_version_2_7_implies_legacy_vue2() {
    let features = features_for(r#"{ "vue": { "version": "2.7" } }"#);
    assert_eq!(features.vue_version, Some(VueVersion::V2_7));
    assert!(features.type_checker_legacy_vue2);
}

#[test]
fn vue_version_2_implies_legacy_vue2() {
    let features = features_for(r#"{ "vue": { "version": "2" } }"#);
    assert_eq!(features.vue_version, Some(VueVersion::V2));
    assert!(features.type_checker_legacy_vue2);
}

#[test]
fn compiler_compatibility_vue_version_implies_legacy_vue2() {
    let features = features_for(r#"{ "compiler": { "compatibility": { "vueVersion": "2" } } }"#);
    assert_eq!(features.vue_version, Some(VueVersion::V2));
    assert!(features.type_checker_legacy_vue2);
}

#[test]
fn explicit_legacy_vue2_still_wins_without_a_dialect() {
    let features = features_for(r#"{ "typeChecker": { "legacyVue2": true } }"#);
    assert_eq!(features.vue_version, None);
    assert!(features.type_checker_legacy_vue2);
}

#[test]
fn vue_3_keeps_legacy_vue2_disabled() {
    for config in [
        r#"{ "vue": { "version": "3" } }"#,
        r#"{ "typeChecker": { "optionsApi": true } }"#,
    ] {
        let features = features_for(config);
        assert!(!features.type_checker_legacy_vue2, "config {config}");
    }
}
