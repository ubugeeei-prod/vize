#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::support::vite_config;

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-lint-preset-scope-cli-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

fn output_details(output: &std::process::Output) -> vize_l0::String {
    vize_l0::cstr!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn lint_without_config_uses_happy_path_instead_of_ecosystem() {
    let project_root = temp_project_dir("default-happy-path");
    write_project_file(
        &project_root,
        "src/App.vue",
        r#"<template><RouterLink to="/settings">Settings</RouterLink></template>"#,
    );

    let default = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--no-config", "--format", "json", "src/App.vue"])
        .output()
        .unwrap();
    assert!(default.status.success(), "{}", output_details(&default));
    let default_json: serde_json::Value =
        serde_json::from_slice(&default.stdout).unwrap_or_else(|_| {
            panic!(
                "default happy-path lint must emit JSON: {}",
                output_details(&default)
            )
        });
    assert_eq!(
        default_json[0]["messages"],
        serde_json::json!([]),
        "{}",
        output_details(&default)
    );

    let ecosystem = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--preset",
            "ecosystem",
            "--no-config",
            "--format",
            "json",
            "src/App.vue",
        ])
        .output()
        .unwrap();
    assert!(
        ecosystem.status.success(),
        "explicit ecosystem should report the static RouterLink target: {}",
        output_details(&ecosystem)
    );
    assert!(
        String::from_utf8_lossy(&ecosystem.stdout)
            .contains("ecosystem/vue-router-prefer-named-link"),
        "{}",
        output_details(&ecosystem)
    );

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn lint_limits_nuxt_link_guidance_to_nuxt_preset() {
    let project_root = temp_project_dir("nuxt-link-preset-boundary");
    write_project_file(
        &project_root,
        "app/pages/index.vue",
        r#"<template><a href="/settings">Settings</a></template>"#,
    );
    let ecosystem = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--preset",
            "ecosystem",
            "--no-config",
            "--format",
            "json",
            "app/pages/index.vue",
        ])
        .output()
        .unwrap();
    assert!(ecosystem.status.success(), "{}", output_details(&ecosystem));
    assert!(
        !String::from_utf8_lossy(&ecosystem.stdout).contains("ecosystem/nuxt-prefer-nuxt-link"),
        "{}",
        output_details(&ecosystem)
    );

    write_project_file(
        &project_root,
        "nuxt.config.ts",
        "export default defineNuxtConfig({})\n",
    );
    let nuxt = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--preset",
            "nuxt",
            "--no-config",
            "--format",
            "json",
            "app/pages/index.vue",
        ])
        .output()
        .unwrap();
    assert!(nuxt.status.success(), "{}", output_details(&nuxt));
    assert!(
        String::from_utf8_lossy(&nuxt.stdout).contains("ecosystem/nuxt-prefer-nuxt-link"),
        "{}",
        output_details(&nuxt)
    );

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn six_default_presets_preserve_complete_cli_warning_off_repair_and_alias_packets() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/lint-preset-default-warnings/cases.json"
    ))
    .unwrap();
    let project = tempfile::tempdir().unwrap();
    let mut commands = 0;
    let mut receipts = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let filename = case["filename"].as_str().unwrap();
        for preset in [
            "happy-path",
            "ecosystem",
            "opinionated",
            "nuxt",
            "essential",
            "incremental",
            "default",
        ] {
            let enabled = case["enabledPresets"]
                .as_array()
                .unwrap()
                .iter()
                .any(|name| {
                    name == if preset == "default" {
                        "happy-path"
                    } else {
                        preset
                    }
                });
            for mode in ["default", "off", "repair"] {
                if preset == "default" && mode != "default" {
                    continue;
                }
                let source = case[if mode == "repair" { "repair" } else { "source" }]
                    .as_str()
                    .unwrap();
                write_project_file(project.path(), filename, source);
                let mut config = serde_json::json!({"linter": {"preset": preset}});
                if mode == "off" {
                    config["linter"]["rules"] =
                        serde_json::json!({(case["rule"].as_str().unwrap()): "off"});
                }
                let original_config = serde_json::to_vec(&config).unwrap();
                let carrier = vite_config::source(&original_config);
                write_project_file(project.path(), "vite.config.mjs", &carrier);
                let argv = [
                    "lint",
                    "--config",
                    "vite.config.mjs",
                    "--format",
                    "json",
                    "--locale",
                    "en",
                    filename,
                ];
                let mut historical_argv = argv;
                historical_argv[2] = "vize.config.json";
                let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                    .current_dir(project.path())
                    .args(argv)
                    .output()
                    .unwrap();
                commands += 1;
                receipts.push(serde_json::json!({
                    "case": case["id"], "preset": preset, "mode": mode,
                    "source": source, "config": config,
                    "sourceSha256": digest(source.as_bytes()),
                    "argv": argv, "historicalArgv": historical_argv,
                    "originalConfigBytes": original_config,
                    "originalConfigSha256": digest(&original_config),
                    "configIdentity": {"historical": "vize.config.json", "current": "vite.config.mjs"},
                    "configBytes": carrier.as_bytes(),
                    "configSha256": digest(carrier.as_bytes()),
                    "configAfterBytes": fs::read(project.path().join("vite.config.mjs")).unwrap(),
                    "status": output.status.code(),
                    "stdoutBytes": &output.stdout, "stderrBytes": &output.stderr
                }));
                assert_eq!(
                    output.status.code(),
                    Some(0),
                    "{} / {preset} / {mode}: {}",
                    case["id"],
                    output_details(&output)
                );
                assert!(
                    output.stderr.is_empty(),
                    "{} / {preset} / {mode}: {}",
                    case["id"],
                    output_details(&output)
                );
                let warning = enabled && mode == "default";
                let mut messages = if warning {
                    vec![case["cliMessage"].clone()]
                } else {
                    vec![]
                };
                if mode != "repair"
                    && case["baselinePresets"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|name| {
                            name == if preset == "default" {
                                "happy-path"
                            } else {
                                preset
                            }
                        })
                {
                    let baseline = case["baselineCliMessages"].as_array().unwrap();
                    if case["baselineCliMessagesFirst"].as_bool() == Some(true) {
                        messages = baseline.iter().cloned().chain(messages).collect();
                    } else {
                        messages.extend(baseline.iter().cloned());
                    }
                }
                let expected = serde_json::json!([{
                    "file": filename, "errorCount": 0, "warningCount": messages.len(), "messages": messages
                }]);
                assert_eq!(
                    serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
                    expected,
                    "{} / {preset} / {mode}: {}",
                    case["id"],
                    output_details(&output)
                );
                assert_eq!(
                    fs::read_to_string(project.path().join(filename)).unwrap(),
                    source
                );
                assert_eq!(
                    fs::read_to_string(project.path().join("vite.config.mjs")).unwrap(),
                    carrier
                );
            }
        }
    }
    assert_eq!(commands, 76);
    println!(
        "{}",
        serde_json::json!({
            "schema": "vize.lint.preset-default-warning-cli", "version": 1,
            "commands": commands, "runs": receipts
        })
    );
}
