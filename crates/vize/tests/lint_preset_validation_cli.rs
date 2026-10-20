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

#[test]
fn lint_preset_help_lists_every_public_selection_and_rejects_unknown_names() {
    let help = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["lint", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success(), "{}", output_details(&help));
    let stdout = String::from_utf8_lossy(&help.stdout);
    for preset in [
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
    ] {
        assert!(stdout.contains(preset), "missing {preset} from:\n{stdout}");
    }

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
    assert!(stderr.contains("invalid value 'bogus'"), "{stderr}");
    assert!(stderr.contains("possible values"), "{stderr}");
    assert!(stderr.contains("all"), "{stderr}");
}

#[test]
fn all_preset_runs_opt_in_rules_from_cli_and_config() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(
        project.path().join("src/GreetingCard.vue"),
        r#"<script setup lang="ts">
import { nextTick } from "vue";

await nextTick();
</script>

<template>
  <p>Hello</p>
</template>
"#,
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
    assert!(
        cli_rules.contains("script/no-next-tick"),
        "{}",
        output_details(&cli)
    );
    assert!(
        cli_rules.contains("vue/no-bare-strings-in-template"),
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
    assert!(
        configured_rules.contains("script/no-next-tick"),
        "{}",
        output_details(&configured)
    );
    assert!(
        configured_rules.contains("vue/no-bare-strings-in-template"),
        "{}",
        output_details(&configured)
    );
}
