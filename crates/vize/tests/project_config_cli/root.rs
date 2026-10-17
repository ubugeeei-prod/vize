//! The public CLI follows Vite's selected project without changing explicit inputs.
#![expect(
    clippy::disallowed_types,
    reason = "complete fixture artifacts use std strings"
)]

use super::support::{assert_success, run, write};
use serde_json::{Value, json};
use std::fs;

#[path = "root/support.rs"]
mod support;
use support::*;
#[path = "../support/config_refusal.rs"]
mod config_refusal;
#[path = "root/ignore_syntax.rs"]
mod ignore_syntax;

#[test]
fn omitted_build_inputs_use_vite_root_and_explicit_inputs_remain_local() {
    let project = project();
    let root = project.path();
    assert_success(&run(root, &["build", "-o", "configured"]));
    assert_eq!(
        inventory(&root.join("configured")),
        oracle!("build-configured.json")
    );
    assert_configuration(root, &SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    assert_success(&run(
        root,
        &[
            "build",
            "src/Decoy.vue",
            "-o",
            "explicit",
            "--config",
            "vite.config.mjs",
        ],
    ));
    assert_eq!(
        inventory(&root.join("explicit")),
        oracle!("build-explicit.json")
    );
    assert_configuration(root, &SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    assert_success(&run(root, &["build", "--no-config", "-o", "defaults"]));
    assert_eq!(
        inventory(&root.join("defaults")),
        oracle!("build-defaults.json")
    );
    assert_configuration(root, &SETTINGS, false, None);
}

#[test]
fn omitted_lint_inputs_scope_entry_rules_and_ignores_to_vite_root() {
    let project = project();
    let root = project.path();
    let (output, report) = lint_report(root, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("lint-configured.json"));
    assert_configuration(root, &SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    let (explicit, report) = lint_report(root, &["src/Decoy.vue", "--config", "vite.config.mjs"]);
    assert_eq!(explicit.status.code(), Some(1), "{explicit:?}");
    assert_eq!(report, oracle!("lint-explicit.json"));
    assert_configuration(root, &SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    let (defaults, report) = lint_report(root, &["--no-config"]);
    assert_eq!(defaults.status.code(), Some(0), "{defaults:?}");
    assert_eq!(report, oracle!("lint-defaults.json"));
    assert_configuration(root, &SETTINGS, false, None);
}

#[cfg(feature = "glyph")]
#[test]
fn omitted_format_inputs_follow_vite_root_without_writing_siblings() {
    let project = project();
    let root = project.path();
    write(root, "src/Decoy.vue", APP);
    assert_success(&run(root, &["fmt", "--write"]));
    let mut expected = oracle!("fmt-configured.json");
    assert_eq!(inventory(root), expected);
    assert_configuration(root, &SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    assert_success(&run(
        root,
        &[
            "fmt",
            "--write",
            "src/Decoy.vue",
            "--config",
            "vite.config.mjs",
        ],
    ));
    expected["src/Decoy.vue"] = json!({"text":include_str!("../../../../tests/_fixtures/differential/config/vite-root-8371/oracles/fmt-explicit.vue.txt")});
    assert_eq!(inventory(root), expected);
    assert_configuration(root, &SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    let missing = run(root, &["fmt", "--write", "src/Image.vue"]);
    assert_eq!(missing.status.code(), Some(1), "{missing:?}");
    assert_eq!(inventory(root), expected);
    assert_configuration(root, &SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    write(root, "src/Decoy.vue", APP);
    assert_success(&run(
        root,
        &["fmt", "--write", "src/Decoy.vue", "--no-config"],
    ));
    expected["src/Decoy.vue"] = json!({"text":include_str!("../../../../tests/_fixtures/differential/config/vite-root-8371/oracles/fmt-default.vue.txt")});
    expected.as_object_mut().unwrap().remove(".evaluations");
    assert_eq!(inventory(root), expected);
    assert_configuration(root, &SETTINGS, false, None);
}

#[test]
fn canonical_vite_scopes_keep_global_settings_and_root_relative_entries() {
    let project = project();
    let root = project.path();
    write(root, "vite.config.mjs", ARRAY);
    let (output, report) = lint_report(root, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("lint-array.json"));
    assert_configuration(root, ARRAY, false, None);
}

#[test]
fn explicit_native_configuration_keeps_the_existing_invocation_root() {
    let project = project();
    let root = project.path();
    write(root, "vize.config.json", DEDICATED);
    write(root, "manual.json", DEDICATED);
    let (output, report) = lint_report(root, &["--config", "manual.json"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("lint-dedicated.json"));
    assert_success(&run(
        root,
        &["build", "-o", "dedicated", "--config", "manual.json"],
    ));
    assert_eq!(
        inventory(&root.join("dedicated")),
        oracle!("build-dedicated.json")
    );
    assert_eq!(
        fs::read_to_string(root.join("vize.config.json")).unwrap(),
        DEDICATED
    );
    assert_eq!(
        fs::read_to_string(root.join("manual.json")).unwrap(),
        DEDICATED
    );
    assert_configuration(root, &SETTINGS, true, None);
}

#[test]
fn stale_dedicated_configuration_does_not_override_vite_and_explicit_use_rejects() {
    let project = project();
    let root = project.path();
    write(root, "vize.config.json", DEDICATED);
    let (output, report) = lint_report(root, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("lint-configured.json"));
    assert_configuration(root, &SETTINGS, true, Some("evaluated\n"));

    reset_evaluations(root);
    assert_success(&run(root, &["build", "-o", "configured"]));
    assert_eq!(
        inventory(&root.join("configured")),
        oracle!("build-configured.json")
    );
    assert_configuration(root, &SETTINGS, true, Some("evaluated\n"));

    reset_evaluations(root);
    super::support::assert_legacy_config_rejected(&run(
        root,
        &["lint", "--config", "vize.config.json", "--format", "json"],
    ));
    assert_eq!(
        fs::read_to_string(root.join("vize.config.json")).unwrap(),
        DEDICATED
    );
    assert_configuration(root, &SETTINGS, true, None);
}

#[test]
fn omitted_check_inputs_use_vite_tsconfig_and_explicit_tsconfig_stays_local() {
    use super::{corsa_path, corsa_requirement, typecheck};
    use std::process::Command;

    let Some(corsa) =
        corsa_requirement::required_or_skip(corsa_path::resolve(typecheck::workspace_root()))
    else {
        return;
    };
    let project = project();
    let root = project.path();
    typecheck::link_vue(root);
    let check = |args: &[&str]| {
        reset_evaluations(root);
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root)
            .env("CORSA_PATH", &corsa)
            .env("RAYON_NUM_THREADS", "1")
            .args(["check", "--format", "json"])
            .args(args)
            .output()
            .unwrap();
        let mut report: Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{error}: {output:?}"));
        normalize_report(root, &mut report);
        (output, report)
    };
    let (output, report) = check(&[]);
    assert_success(&output);
    assert_eq!(report, oracle!("check-configured.json"));
    assert_configuration(root, &SETTINGS, false, Some("evaluated\n"));

    let cases = [
        (
            &["--tsconfig", "tsconfig.json"][..],
            oracle!("check-tsconfig.json"),
            Some("evaluated\n"),
        ),
        (
            &["src/invalid.ts"][..],
            oracle!("check-explicit.json"),
            Some("evaluated\n"),
        ),
        (&["--no-config"][..], oracle!("check-defaults.json"), None),
    ];
    for (args, expected, evaluation) in cases {
        let (output, report) = check(args);
        assert_eq!(output.status.code(), Some(1), "{args:?}: {output:?}");
        assert_eq!(report, expected);
        assert_configuration(root, &SETTINGS, false, evaluation);
    }
    write(
        root,
        "manual.json",
        r#"{"typeChecker":{"tsconfig":"tsconfig.json"}}"#,
    );
    let (output, report) = check(&["--config", "manual.json"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("check-manual.json"));
    assert_eq!(
        fs::read_to_string(root.join("manual.json")).unwrap(),
        r#"{"typeChecker":{"tsconfig":"tsconfig.json"}}"#
    );
    assert_configuration(root, &SETTINGS, false, None);
}

#[test]
fn malformed_discovered_vue_settings_refuse_build_before_writing_outputs() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    let source = "<template><p>unchanged authored source</p></template>\n";
    let config = "export default {vue:{compiler:{whitespace:'unsupported'}}};\n";
    write(root, "App.vue", source);
    write(root, "vite.config.mjs", config);
    write(
        root,
        "vite.config.cts",
        "throw new Error('lower-priority config evaluated');",
    );
    let refused = std::process::Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("RAYON_NUM_THREADS", "1")
        .env("NODE_OPTIONS", config_refusal::NODE_OPTIONS)
        .args(["build", "App.vue", "-o", "refused"])
        .output()
        .unwrap();
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    assert_eq!(refused.stdout, b"");
    let directory = root.canonicalize().unwrap();
    config_refusal::assert_error(
        &refused,
        &directory.join("vite.config.mjs"),
        &directory,
        "TypeError: vue.compiler.whitespace: unsupported native option value",
        1,
    );
    assert!(!root.join("refused").exists());
    assert_eq!(fs::read_to_string(root.join("App.vue")).unwrap(), source);
    assert_eq!(
        fs::read_to_string(root.join("vite.config.mjs")).unwrap(),
        config
    );
    assert_success(&run(
        root,
        &["build", "App.vue", "--no-config", "-o", "defaults"],
    ));
    assert!(root.join("defaults").is_dir());
}
