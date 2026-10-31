//! Actual canonical Vite configuration, complete diagnostics, file fixes and requery.
use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, io::Write, path::Path, process::Command};

const RULE: &str = "vue/attribute-hyphenation";
const CORPUS: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/attribute-hyphenation-ignore-6097/cases.json"
);
const REFERENCE: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/attribute-hyphenation-ignore-6097/eslint-plugin-vue.original.json"
);
fn source(file: &str) -> &'static str {
    match file {
        "RegisteredCamel.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/attribute-hyphenation-ignore-6097/RegisteredCamel.vue.txt"
        ),
        "RegisteredHyphen.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/attribute-hyphenation-ignore-6097/RegisteredHyphen.vue.txt"
        ),
        _ => panic!("unknown independent attribute input"),
    }
}
fn write(root: &Path, file: &str, source: &str) {
    let target = root.join(file);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(target, source).unwrap();
}
fn clean() -> Value {
    json!([{"file":"src/Probe.vue","messages":[],"errorCount":0,"warningCount":0}])
}
fn expected(reference: &Value) -> Value {
    let messages = reference["verify"][0]["messages"].as_array().unwrap().iter().map(|finding| {
        let (message,help) = match finding["messageId"].as_str().unwrap() {
            "mustBeHyphenated" => ("Attribute should be hyphenated","Use kebab-case for attribute names"),
            "cannotBeHyphenated" => ("Attribute should not be hyphenated","Use camelCase or a single-word attribute name instead of kebab-case."),
            _ => panic!("unknown independent attribute finding"),
        };
        json!({"ruleId":RULE,"ruleDocsPath":"docs/content/rules/vue.md","severity":2,
        "message":format!("[vize:{RULE}] {message}"),"help":help,
        "line":finding["line"],"column":finding["column"],"endLine":finding["endLine"],"endColumn":finding["endColumn"]})
    }).collect::<Vec<_>>();
    json!([{"file":"src/Probe.vue","errorCount":messages.len(),"warningCount":0,"messages":messages}])
}
#[test]
fn attribute_ignore_vite_cli_preserves_nine_whole_reports_actual_fixes_and_fixed_points() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let original: Value = serde_json::from_str(REFERENCE).unwrap();
    let rows = corpus["valid"].as_array().unwrap();
    assert_eq!(rows.len(), 9);
    let project = tempfile::tempdir().unwrap();
    for row in rows {
        let source = source(row["file"].as_str().unwrap());
        let reference = &original["rows"][row["referenceRow"].as_u64().unwrap() as usize];
        assert_eq!(reference["id"], row["id"]);
        assert_eq!(reference["source"], source);
        write(project.path(),"vite.config.mjs",&vite_config::source(json!({"linter":{"preset":"incremental","rules":{RULE:"error"},"ruleOptions":{RULE:row["policy"]}}}).to_string()));
        write(project.path(), "src/Probe.vue", source);
        for pass in 0..3 {
            let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
            command.current_dir(project.path()).args([
                "lint",
                "--config",
                "vite.config.mjs",
                "--format",
                "json",
                "src/Probe.vue",
            ]);
            if pass == 1 {
                command.arg("--fix");
            }
            let output = command.output().unwrap();
            let file = fs::read_to_string(project.path().join("src/Probe.vue")).unwrap();
            let packet = serde_json::to_vec(&json!({"id":row["id"],"pass":pass,"status":output.status.code(),"source":source,"file":file,"stdout":std::str::from_utf8(&output.stdout).unwrap(),"stderr":std::str::from_utf8(&output.stderr).unwrap()})).unwrap();
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(b"vize-attribute-ignore-cli ").unwrap();
            stdout.write_all(&packet).unwrap();
            stdout.write_all(b"\n").unwrap();
            assert_eq!(
                output.status.code(),
                Some(i32::from(
                    pass == 0 && row["expectedCount"].as_u64().unwrap() > 0
                )),
                "{}",
                row["id"]
            );
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                if pass == 0 {
                    expected(reference)
                } else {
                    clean()
                },
                "{}",
                row["id"]
            );
            assert_eq!(
                file,
                if pass == 0 {
                    source
                } else {
                    reference["fixedSource"].as_str().unwrap()
                }
            );
        }
    }
    let input = source("RegisteredCamel.vue.txt");
    write(project.path(), "src/Probe.vue", input);
    write(project.path(),"vite.config.mjs",&vite_config::source(json!({"linter":{"preset":"incremental","rules":{RULE:"off"},"ruleOptions":{RULE:{"ignore":[]}}}}).to_string()));
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--config",
            "vite.config.mjs",
            "--format",
            "json",
            "--fix",
            "src/Probe.vue",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        clean()
    );
    assert_eq!(
        fs::read_to_string(project.path().join("src/Probe.vue")).unwrap(),
        input
    );
}
