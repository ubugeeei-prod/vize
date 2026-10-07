use std::{fs, process::Command};

#[test]
fn cli_config_permits_preview_globals_and_retains_missing_component_errors() {
    let project = tempfile::tempdir().unwrap();
    fs::write(
        project.path().join("Card.art.vue"),
        include_str!("../../vize_patina/tests/fixtures/global-component-registration/Card.art.vue"),
    )
    .unwrap();
    fs::write(
        project.path().join("vize.config.json"),
        include_str!(
            "../../vize_patina/tests/fixtures/global-component-registration/vize.config.json"
        ),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "Card.art.vue",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let messages = report[0]["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["ruleId"], "vue/require-component-registration");
    assert_eq!(messages[0]["severity"], 2);
    assert_eq!(messages[0]["line"], 10);
    assert_eq!(messages[0]["column"], 8);
}
