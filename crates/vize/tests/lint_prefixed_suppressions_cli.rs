#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::{fs, process::Command};

fn mutation_count(source: &str) -> usize {
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("App.vue"), source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--preset",
            "opinionated",
            "--format",
            "json",
            "App.vue",
        ])
        .output()
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "vize lint did not return JSON:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    result[0]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|message| message["ruleId"] == "vue/no-mutating-props")
        .count()
}

#[test]
fn lint_cli_honors_plugin_rule_ids_in_eslint_and_oxlint_comments() {
    let source = |comment: &str| {
        format!(
            "<script setup lang=\"ts\">\nconst props = defineProps<{{ items: string[] }}>()\n{comment}\nObject.assign(props.items, [])\n</script>\n"
        )
    };
    assert_eq!(mutation_count(&source("// unrelated comment")), 1);
    assert_eq!(
        mutation_count(&source(
            "// eslint-disable-next-line vize/vue/no-mutating-props"
        )),
        0
    );
    assert_eq!(
        mutation_count(&source(
            "// oxlint-disable-next-line vize/vue/no-mutating-props"
        )),
        0
    );
}
