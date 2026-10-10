//! Public tools consume project settings without a dedicated Vize config (#8371).
#![cfg(test)]
#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[cfg(all(feature = "maestro", feature = "glyph"))]
#[path = "project_config_cli/lsp.rs"]
mod lsp;
#[cfg(all(feature = "maestro", feature = "glyph"))]
#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "project_config_cli/support.rs"]
mod support;
#[path = "project_config_cli/typecheck.rs"]
mod typecheck;

use serde_json::{Value, json};
use std::fs;
#[cfg(feature = "glyph")]
use support::formatted;
use support::{assert_no_dedicated_config, assert_success, project, run, write};

#[test]
fn compiler_reads_whitespace_and_custom_elements_from_vite() {
    let project = project();
    let root = project.path();
    assert_success(&run(root, &["build", "App.vue", "-o", "configured"]));
    let configured = fs::read_to_string(root.join("configured/App.js")).unwrap();
    insta::assert_snapshot!("project_settings_build_vite", configured);

    assert_success(&run(
        root,
        &["build", "App.vue", "-o", "defaults", "--no-config"],
    ));
    let defaults = fs::read_to_string(root.join("defaults/App.js")).unwrap();
    insta::assert_snapshot!("project_settings_build_defaults", defaults);
    assert_ne!(configured, defaults);
    assert_no_dedicated_config(root);
    assert_eq!(
        fs::read_to_string(root.join("vite.config.mjs")).unwrap(),
        support::SETTINGS
    );
}

#[test]
fn configured_lint_rule_changes_the_public_diagnostic_and_can_be_overridden() {
    let project = project();
    let root = project.path();
    let configured = run(root, &["lint", "Image.vue", "--format", "json"]);
    let report: Value = serde_json::from_slice(&configured.stdout).unwrap();
    assert_eq!(configured.status.code(), Some(1), "{configured:?}");
    insta::assert_snapshot!(
        "project_settings_lint_vite",
        serde_json::to_string_pretty(&support::normalized_report(root, report)).unwrap()
    );

    let manual = root.join("manual.json");
    fs::write(
        &manual,
        serde_json::to_vec(
            &json!({"linter":{"preset":"essential","rules":{"a11y/alt-text":"off"}}}),
        )
        .unwrap(),
    )
    .unwrap();
    let overridden = run(
        root,
        &[
            "lint",
            "Image.vue",
            "--format",
            "json",
            "--config",
            manual.to_str().unwrap(),
        ],
    );
    // The independent component-name error stays reported when alt-text is off.
    assert_eq!(overridden.status.code(), Some(1), "{overridden:?}");
    let report: Value = serde_json::from_slice(&overridden.stdout).unwrap();
    insta::assert_snapshot!(
        "project_settings_lint_override",
        serde_json::to_string_pretty(&support::normalized_report(root, report)).unwrap()
    );
    assert_no_dedicated_config(root);
}

#[cfg(feature = "glyph")]
#[test]
fn formatter_applies_vite_settings_and_explicit_cli_switches() {
    let project = project();
    let root = project.path();
    let configured = formatted(root, &[]);
    assert_eq!(configured.as_str(), support::PRESERVED_SINGLE);
    let defaults = formatted(root, &["--no-config"]);
    assert_eq!(defaults.as_str(), support::FORMATTED_DOUBLE);
    let cli_override = formatted(root, &["--single-quote=false"]);
    assert_eq!(cli_override.as_str(), support::PRESERVED_DOUBLE);
    assert_no_dedicated_config(root);
}

#[cfg(feature = "glyph")]
#[test]
fn package_boundaries_precedence_and_sibling_settings_reach_the_formatter() {
    let workspace = tempfile::tempdir().unwrap();
    let root = workspace.path();
    write(root, "package.json", "{}");
    write(
        root,
        "vize.config.json",
        r#"{"formatter":{"singleQuote":false}}"#,
    );
    for package in ["packages/a", "packages/b"] {
        write(&root.join(package), "package.json", "{}");
        fs::create_dir_all(root.join(package).join("src")).unwrap();
    }
    write(
        &root.join("packages/a"),
        "vite.config.mjs",
        "export default {vize:{formatter:{singleQuote:true}}};",
    );
    write(
        &root.join("packages/b"),
        "vite.config.mjs",
        "export default {vize:{formatter:{singleQuote:false}}};",
    );
    let a = root.join("packages/a/src");
    let b = root.join("packages/b/src");
    assert_eq!(formatted(&a, &[]).as_str(), support::FORMATTED_SINGLE);
    assert_eq!(formatted(&b, &[]).as_str(), support::FORMATTED_DOUBLE);

    // A package with no settings keeps its own defaults instead of inheriting
    // an unrelated workspace config, or its sibling's cached settings.
    write(
        root,
        "vize.config.json",
        r#"{"formatter":{"singleQuote":true}}"#,
    );
    fs::remove_file(root.join("packages/b/vite.config.mjs")).unwrap();
    assert_eq!(formatted(&b, &[]).as_str(), support::FORMATTED_DOUBLE);

    // Existing same-directory dedicated configs still override Vite settings.
    write(
        &root.join("packages/a"),
        "vize.config.json",
        r#"{"formatter":{"singleQuote":false}}"#,
    );
    assert_eq!(formatted(&a, &[]).as_str(), support::FORMATTED_DOUBLE);
    write(
        &root.join("packages/a"),
        "manual.json",
        r#"{"formatter":{"singleQuote":true}}"#,
    );
    assert_eq!(
        formatted(&a, &["--config", "../manual.json"]).as_str(),
        support::FORMATTED_SINGLE
    );
    assert_eq!(
        formatted(&a, &["--no-config"]).as_str(),
        support::FORMATTED_DOUBLE
    );
}
