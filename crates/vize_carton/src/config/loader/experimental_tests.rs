use super::{load_compiler_vapor, load_config_with_features_and_source};
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
    assert_eq!(load_compiler_vapor(Some(&config_path)), Some(false));
}
