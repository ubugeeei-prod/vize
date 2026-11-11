//! `vize lint` hands the eslint-plugin-vue compatibility options to their rules (#12719).

use super::{output_details, write_file};
use crate::support::vite_config;
use serde_json::{Value, json};
use std::process::Command;

const PLAIN: &str = "<script setup lang=\"ts\">\nconst ok = true;\n</script>\n\n<template>\n  <p class=\"card\">{{ ok ? \"yes\" : \"no\" }}</p>\n</template>\n\n<style>\n.card {\n  color: red;\n}\n</style>\n";
const SCOPED: &str = "<script setup lang=\"ts\">\nconst label = \"a\";\n</script>\n\n<template>\n  <p class=\"card\">{{ label }}</p>\n</template>\n\n<style scoped>\n.card {\n  color: red;\n}\n</style>\n";

/// `(file, rule, line, column)` of every reported message.
fn lint(rules: Value, rule_options: Value) -> Vec<(String, String, u64, u64)> {
    let project = tempfile::tempdir().unwrap();
    let config = json!({ "linter": {
        "preset": "incremental", "rules": rules, "ruleOptions": rule_options
    }});
    write_file(
        project.path(),
        "vite.config.mjs",
        &vite_config::source(config.to_string()),
    );
    write_file(project.path(), "src/plain/Case.vue", PLAIN);
    write_file(project.path(), "src/scoped/Case.vue", SCOPED);
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["lint", "--config", "vite.config.mjs", "--format", "json"])
        .args(["src/plain/Case.vue", "src/scoped/Case.vue"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let files = serde_json::from_slice::<Value>(&output.stdout).unwrap();
    let mut found = Vec::new();
    for file in files.as_array().into_iter().flatten() {
        for message in file["messages"].as_array().into_iter().flatten() {
            found.push((
                file["file"].as_str().unwrap_or_default().to_owned(),
                message["ruleId"].as_str().unwrap_or_default().to_owned(),
                message["line"].as_u64().unwrap_or_default(),
                message["column"].as_u64().unwrap_or_default(),
            ));
        }
    }
    found
}

#[test]
fn compatibility_rule_options_reach_their_rules() {
    let both =
        json!({ "vue/enforce-style-attribute": "error", "vue/no-restricted-syntax": "error" });
    let found = lint(
        both,
        json!({
            "vue/enforce-style-attribute": { "allow": ["scoped", "plain"] },
            "vue/no-restricted-syntax": { "selectors": ["ConditionalExpression"] },
        }),
    );
    assert_eq!(
        found,
        [(
            "src/plain/Case.vue".to_owned(),
            "vue/no-restricted-syntax".to_owned(),
            6,
            22
        )]
    );

    let found = lint(
        json!({ "vue/enforce-style-attribute": "error" }),
        json!({ "vue/enforce-style-attribute": { "allow": ["plain"] } }),
    );
    assert_eq!(
        found,
        [(
            "src/scoped/Case.vue".to_owned(),
            "vue/enforce-style-attribute".to_owned(),
            9,
            1
        )]
    );
}
