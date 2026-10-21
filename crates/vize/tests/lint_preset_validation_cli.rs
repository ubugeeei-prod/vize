#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use format!")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

#[path = "support/vite_config.rs"]
mod vite_config;

use std::{collections::BTreeSet, fs, process::Command};

fn output_details(output: &std::process::Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn json_rule_ids(output: &std::process::Output) -> BTreeSet<String> {
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|_| panic!("lint must emit JSON: {}", output_details(output)));
    parsed
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["messages"].as_array().unwrap())
        .map(|message| message["ruleId"].as_str().unwrap().to_owned())
        .collect()
}

fn possible_values(output: &str) -> Vec<&str> {
    let (_, values) = output
        .split_once("[possible values:")
        .expect("possible values section");
    let (values, _) = values.split_once(']').expect("possible values terminator");
    values
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect()
}

#[test]
fn lint_preset_help_lists_every_public_selection_and_rejects_unknown_names() {
    let help = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["lint", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success(), "{}", output_details(&help));
    let stdout = String::from_utf8_lossy(&help.stdout);
    let expected = vec![
        "happy-path",
        "general-recommended",
        "default",
        "recommended",
        "ecosystem",
        "opinionated",
        "strict",
        "all",
        "essential",
        "incremental",
        "nuxt",
    ];
    assert_eq!(possible_values(&stdout), expected);

    let unknown = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["lint", "--preset", "bogus", "src"])
        .output()
        .unwrap();
    assert!(!unknown.status.success(), "{}", output_details(&unknown));
    assert_eq!(
        unknown.status.code(),
        Some(2),
        "{}",
        output_details(&unknown)
    );
    let stderr = String::from_utf8_lossy(&unknown.stderr);
    assert_eq!(stderr.split('\'').nth(1), Some("bogus"), "{stderr}");
    assert_eq!(possible_values(&stderr), expected);
}

#[test]
fn all_preset_runs_opt_in_rules_from_cli_and_config() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(
        project.path().join("src/GreetingCard.vue"),
        include_str!(
            "../../../tests/_fixtures/differential/linter/cli-preset-validation-9853/GreetingCard.vue.txt"
        ),
    )
    .unwrap();

    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .args(args)
            .output()
            .unwrap()
    };

    let cli = run(&[
        "lint",
        "--no-config",
        "--preset",
        "all",
        "--format",
        "json",
        "src/GreetingCard.vue",
    ]);
    let cli_rules = json_rule_ids(&cli);
    let expected = BTreeSet::from([
        "script/no-next-tick".to_owned(),
        "vue/no-bare-strings-in-template".to_owned(),
    ]);
    assert_eq!(
        cli_rules
            .intersection(&expected)
            .cloned()
            .collect::<BTreeSet<_>>(),
        expected,
        "{}",
        output_details(&cli)
    );

    fs::write(
        project.path().join("vite.config.mjs"),
        vite_config::source(r#"{ "linter": { "preset": "all" } }"#),
    )
    .unwrap();
    let configured = run(&[
        "lint",
        "--config",
        "vite.config.mjs",
        "--format",
        "json",
        "src/GreetingCard.vue",
    ]);
    let configured_rules = json_rule_ids(&configured);
    assert_eq!(
        configured_rules
            .intersection(&expected)
            .cloned()
            .collect::<BTreeSet<_>>(),
        expected,
        "{}",
        output_details(&configured)
    );
}
