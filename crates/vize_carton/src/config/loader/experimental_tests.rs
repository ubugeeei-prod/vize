use super::{
    load_compiler_vapor, load_config_experimental_vue_flags_with_source,
    load_config_with_features_and_source,
};
use crate::config::JsxMode;

#[test]
fn load_config_reads_experimentals() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{
          "experimentals": {
            "vapor": {},
            "jsxVapor": {},
            "intagComment": {},
            "pattenedTemplate": {},
            "selfComponent": {},
            "strictSlotChildren": {},
            "serverScript": {}
          }
        }"#,
    )
    .unwrap();

    let loaded = load_config_with_features_and_source(Some(&config_path));
    assert!(loaded.features.experimental_vapor);
    assert!(loaded.features.experimental_jsx_vapor);
    assert!(loaded.features.experimental_in_tag_comments);
    assert!(loaded.features.experimental_patterned_template);
    assert!(loaded.features.experimental_server_script);
    assert!(loaded.features.type_checker_jsx_typecheck);
    assert_eq!(loaded.features.jsx_mode, Some(JsxMode::Vapor));
    let experimental_vue = load_config_experimental_vue_flags_with_source(Some(&config_path));
    assert!(experimental_vue.flags.self_component);
    assert!(experimental_vue.flags.strict_slot_children);
    assert_eq!(load_compiler_vapor(Some(&config_path)), Some(true));
}

#[test]
fn load_config_accepts_experimental_aliases_and_false_switches() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{
          "compiler": { "jsxMode": "vdom", "vapor": false },
          "experimentals": {
            "vapor": {},
            "jsxVapor": {},
            "inTagComment": true,
            "patternedTemplate": false,
            "self_component": null,
            "strict_slot_children": true,
            "server script": null
          }
        }"#,
    )
    .unwrap();

    let loaded = load_config_with_features_and_source(Some(&config_path));
    assert!(loaded.features.experimental_in_tag_comments);
    assert!(!loaded.features.experimental_patterned_template);
    assert!(!loaded.features.experimental_server_script);
    assert!(loaded.features.type_checker_jsx_typecheck);
    assert_eq!(loaded.features.jsx_mode, Some(JsxMode::Vdom));
    let experimental_vue = load_config_experimental_vue_flags_with_source(Some(&config_path));
    assert!(!experimental_vue.flags.self_component);
    assert!(experimental_vue.flags.strict_slot_children);
    assert_eq!(load_compiler_vapor(Some(&config_path)), Some(false));
}
