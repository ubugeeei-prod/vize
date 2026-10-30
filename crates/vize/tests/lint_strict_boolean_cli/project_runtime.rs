//! A project-selected tsrs must perform the same complete authored checks.
use super::{RULE, lint, project, read_native_config, vite_config};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

#[test]
fn project_tsrs_discovery_preserves_nullable_boolean_findings() {
    let Some(tsrs) = std::env::var_os("VIZE_TEST_TSRS_PACKAGE") else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSRS").is_none(),
            "the required tsrs provider package is missing"
        );
        return;
    };
    let source = include_str!(
        "../../../../tests/_fixtures/differential/linter/native-boolean-tsrs-11147/App.vue.txt"
    );
    let Some(borrowed) = project(source, json!({})) else {
        return;
    };
    // Keep this consumer outside the checkout: an ancestor workspace's stable
    // native package would otherwise outrank the project's tsrs wrapper.
    let project = tempfile::Builder::new()
        .prefix("vize-tsrs-11147-")
        .tempdir()
        .unwrap();
    for name in [
        "App.vue",
        "package.json",
        "tsconfig.json",
        "vite.config.mjs",
    ] {
        fs::copy(borrowed.path().join(name), project.path().join(name)).unwrap();
    }
    fs::create_dir(project.path().join("node_modules")).unwrap();
    std::os::unix::fs::symlink(
        borrowed
            .path()
            .join("node_modules/vue")
            .canonicalize()
            .unwrap(),
        project.path().join("node_modules/vue"),
    )
    .unwrap();
    let expected = json!([{
        "file":"App.vue",
        "messages":[{
            "ruleId":RULE,
            "ruleDocsPath":"docs/content/rules/type-and-script.md",
            "severity":2,
            "message":"[vize:type/strict-boolean-expressions] Unexpected nullable number value in conditional. Please handle the nullish and zero cases explicitly.",
            "line":6,"column":12,"endLine":6,"endColumn":17
        }],
        "errorCount":1,"warningCount":0
    }]);
    assert_eq!(lint(project.path()), (Some(1), expected.clone()));

    let config_path = project.path().join("vite.config.mjs");
    let mut config = read_native_config(&config_path);
    config.as_object_mut().unwrap().remove("typeChecker");
    fs::write(
        &config_path,
        vite_config::source(serde_json::to_vec(&config).unwrap()),
    )
    .unwrap();
    fs::write(
        project.path().join("package.json"),
        r#"{"type":"module","devDependencies":{"tsrs":"0.20.1-ts7.1.0-beta"}}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink(&tsrs, project.path().join("node_modules/tsrs")).unwrap();
    let selected = vize_carton::corsa_resolver::discover_corsa_in_ancestors(project.path())
        .expect("project tsrs native image")
        .canonicalize()
        .unwrap();
    let provider_modules = Path::new(&tsrs)
        .canonicalize()
        .unwrap()
        .parent()
        .unwrap()
        .join("@tsrs");
    assert!(selected.starts_with(provider_modules));
    assert_eq!(selected.file_name().unwrap(), "tsrs");
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(project.path())
        .args(["lint", "--format", "json", "App.vue"]);
    for name in vize_carton::corsa_resolver::CORSA_ENV_VARS {
        command.env_remove(name);
    }
    let output = command.output().expect("auto-discovered tsrs lint");
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        expected
    );
    assert_eq!(
        fs::read_to_string(project.path().join("App.vue")).unwrap(),
        source
    );
}
