//! Genuine public CLI and formatter controls; the existing archive supplies this binary.
use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, process::Command};

const ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-cli-runtime-reads/"
);
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-cli-runtime-reads/cases.json"
));

#[test]
fn component_value_reads_use_the_actual_ecosystem_cli_and_selected_error_disabled_configs() {
    let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
    assert_eq!(manifest["schema"], "vize.lint.component-cli-runtime-reads");
    assert_eq!(manifest["version"], 1);
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(
        cases
            .iter()
            .map(|case| case["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "ecosystem-template-value",
            "ecosystem-unused",
            "ecosystem-comment-only",
            "configured-error",
            "configured-disabled",
        ]
    );
    let project = tempfile::tempdir().unwrap();
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let input = case["source"].as_str().unwrap();
        let source = fs::read(std::path::Path::new(ROOT).join(input)).unwrap();
        let receipt = manifest["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["source"] == input)
            .unwrap();
        assert_eq!(source.len() as u64, receipt["bytes"].as_u64().unwrap());
        fs::write(project.path().join("ValueReader.vue"), &source).unwrap();
        let config = if let Some(name) = case["config"].as_str() {
            let body = fs::read(std::path::Path::new(ROOT).join(name)).unwrap();
            fs::write(
                project.path().join("vite.config.mjs"),
                vite_config::source(&body),
            )
            .unwrap();
            Some(body)
        } else {
            None
        };
        let mut args: Vec<_> = case["argv"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect();
        assert_eq!(
            &args[..6],
            [
                "lint",
                "ValueReader.vue",
                "--format",
                "json",
                "--preset",
                "ecosystem"
            ]
        );
        assert_eq!(
            &args[6..],
            if config.is_some() {
                &["--config", "vize.config.json"][..]
            } else {
                &["--no-config"][..]
            }
        );
        if config.is_some() {
            args[7] = "vite.config.mjs";
        }
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .args(&args)
            .output()
            .unwrap();
        println!(
            "{}",
            json!({"case": id, "program": env!("CARGO_BIN_EXE_vize"), "actualCwd": project.path(), "argv": args, "sourceBytes": source, "configBytes": config, "status": output.status.code(), "stdoutBytes": output.stdout, "stderrBytes": output.stderr})
        );
        assert_eq!(
            output.status.code().map(i64::from),
            case["status"].as_i64(),
            "{id}"
        );
        assert_eq!(
            serde_json::to_value(&output.stderr).unwrap(),
            case["stderrBytes"],
            "{id}: complete stderr"
        );
        let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            actual, case["expected"],
            "{id}: entire formatter report, no rule filtering"
        );
        assert_eq!(
            fs::read(project.path().join("ValueReader.vue")).unwrap(),
            source,
            "{id}: physical original unchanged"
        );
        if let Some(config) = config {
            assert_eq!(
                fs::read(project.path().join("vite.config.mjs")).unwrap(),
                vite_config::source(&config).as_bytes(),
                "{id}: selected config unchanged"
            );
        }
    }
}
