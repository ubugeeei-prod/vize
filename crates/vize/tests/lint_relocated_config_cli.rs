//! #7307: serialized temporary configs and ./ config paths preserve lint scopes.

use std::{fs, process::Command};

#[test]
fn temporary_config_and_dot_config_keep_original_project_scopes() {
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let root = project.path();
    let source = include_str!("fixtures/lint-config-paths/Image.vue");
    for file in ["gen/a.vue", "src/legacy/b.vue", "src/c.vue"] {
        let path = root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    let relative: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/lint-config-paths/vize.config.json")).unwrap();
    fs::write(
        root.join("vize.config.json"),
        serde_json::to_vec(&relative).unwrap(),
    )
    .unwrap();
    let mut relocated = relative.clone();
    relocated["ignores"] = serde_json::json!([root.join("gen/**")]);
    relocated["entries"][0]["basePath"] = serde_json::json!(root);
    let temporary = outside.path().join(".vize-vp-config.json");
    fs::write(&temporary, serde_json::to_vec(&relocated).unwrap()).unwrap();

    let mut reference = None;
    for config_path in [
        root.join("vize.config.json"),
        std::path::PathBuf::from("./vize.config.json"),
        temporary,
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root)
            .args(["lint", "--format", "json", "--config"])
            .arg(&config_path)
            .arg(".")
            .output()
            .unwrap();
        assert!(output.status.success(), "{config_path:?}: {:?}", output);
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let findings: Vec<_> = report
            .as_array()
            .unwrap()
            .iter()
            .filter(|file| !file["messages"].as_array().unwrap().is_empty())
            .collect();
        assert_eq!(findings.len(), 1, "{config_path:?}: {report}");
        assert_eq!(findings[0]["file"], "src/c.vue");
        assert_eq!(findings[0]["messages"][0]["ruleId"], "a11y/alt-text");
        assert_eq!(findings[0]["messages"].as_array().unwrap().len(), 1);
        if let Some(expected) = &reference {
            assert_eq!(&report, expected, "{config_path:?}");
        } else {
            reference = Some(report);
        }
    }
}
