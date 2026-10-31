use super::{output_details, write_file};
use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, process::Command};

const RULE: &str = "vue/v-bind-style";
const CORPUS: &str = include_str!(
    "../../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/cases.json"
);

fn source(file: &str) -> &'static str {
    match file {
        "Original.vue.txt" => include_str!(
            "../../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/Original.vue.txt"
        ),
        "Explicit.vue.txt" => include_str!(
            "../../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/Explicit.vue.txt"
        ),
        "Dynamic.vue.txt" => include_str!(
            "../../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/Dynamic.vue.txt"
        ),
        "Controls.vue.txt" => include_str!(
            "../../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/Controls.vue.txt"
        ),
        "Colon.vue.txt" => include_str!(
            "../../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/Colon.vue.txt"
        ),
        _ => panic!("unknown pinned v-bind input"),
    }
}
fn expected(row: &Value) -> Value {
    let source = source(row["file"].as_str().unwrap());
    let messages = row["reference"]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            let (message, help) = match finding["messageId"].as_str().unwrap() {
                "expectedLonghandForProp" => (
                    "Expected 'v-bind:' instead of '.'.",
                    "Use v-bind:attr.prop=\"value\" instead of .attr=\"value\"",
                ),
                "expectedLonghand" => (
                    "Prefer `v-bind:` over shorthand `:`",
                    "Use v-bind:attr=\"value\" instead of :attr=\"value\"",
                ),
                "unexpectedLonghand" => (
                    "Prefer shorthand `:` over `v-bind:`",
                    "Use :attr=\"value\" instead of v-bind:attr=\"value\"",
                ),
                _ => panic!("unknown independent finding"),
            };
            json!({"ruleId":RULE,"ruleDocsPath":"docs/content/rules/vue.md","severity":2,
            "message":format!("[vize:{RULE}] {message}"),"help":help,
            "line":finding["line"],"column":scalar_column(source,finding,"line","column"),
            "endLine":finding["endLine"],"endColumn":scalar_column(source,finding,"endLine","endColumn")})
        })
        .collect::<Vec<_>>();
    json!([{"file":"src/VBindProp.vue","errorCount":messages.len(),"warningCount":0,"messages":messages}])
}
// JSON CLI SourceLineIndex reports Unicode scalar columns; the independent
// ESLint corpus and public NAPI report UTF-16. Keep all three native contracts.
fn scalar_column(source: &str, finding: &Value, line_key: &str, column_key: &str) -> usize {
    let line = source
        .lines()
        .nth(finding[line_key].as_u64().unwrap() as usize - 1)
        .unwrap();
    let target = finding[column_key].as_u64().unwrap() as usize - 1;
    let mut units = 0;
    let mut column = 1;
    for character in line.chars() {
        if units == target {
            return column;
        }
        units += character.len_utf16();
        column += 1;
        assert!(
            units <= target,
            "reference boundary must not split a surrogate pair"
        );
    }
    assert_eq!(units, target);
    column
}
fn clean() -> Value {
    json!([{"file":"src/VBindProp.vue","messages":[],"errorCount":0,"warningCount":0}])
}

#[test]
fn property_shorthand_vite_config_cli_preserves_nine_whole_reports_fixes_and_fixed_points() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let project = tempfile::tempdir().unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 9);
    for row in cases {
        let mut config = json!({"linter":{"preset":"incremental","rules":{RULE:"error"}}});
        if !row["style"].is_null() {
            config["linter"]["ruleOptions"] = json!({RULE:row["style"]});
        }
        write_file(
            project.path(),
            "vite.config.mjs",
            &vite_config::source(config.to_string()),
        );
        let source = source(row["file"].as_str().unwrap());
        write_file(project.path(), "src/VBindProp.vue", source);
        for pass in 0..3 {
            let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
            command.current_dir(project.path()).args([
                "lint",
                "--config",
                "vite.config.mjs",
                "--format",
                "json",
                "src/VBindProp.vue",
            ]);
            if pass > 0 {
                command.arg("--fix");
            }
            let output = command.output().unwrap();
            let wanted = if pass == 0 { expected(row) } else { clean() };
            assert_eq!(
                output.status.code(),
                Some(i32::from(
                    pass == 0 && !row["reference"]["messages"].as_array().unwrap().is_empty()
                )),
                "{} {}",
                row["id"],
                output_details(&output)
            );
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                wanted,
                "{} {}",
                row["id"],
                output_details(&output)
            );
            assert_eq!(
                fs::read_to_string(project.path().join("src/VBindProp.vue")).unwrap(),
                if pass == 0 {
                    source
                } else {
                    row["reference"]["fixResult"]["output"].as_str().unwrap()
                }
            );
        }
    }
    let source = source("Original.vue.txt");
    write_file(project.path(), "src/VBindProp.vue", source);
    write_file(project.path(),"vite.config.mjs",&vite_config::source(json!({"linter":{"preset":"incremental","rules":{RULE:"off"},"ruleOptions":{RULE:"longform"}}}).to_string()));
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--config",
            "vite.config.mjs",
            "--format",
            "json",
            "--fix",
            "src/VBindProp.vue",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", output_details(&output));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        clean()
    );
    assert_eq!(
        fs::read_to_string(project.path().join("src/VBindProp.vue")).unwrap(),
        source
    );
}
