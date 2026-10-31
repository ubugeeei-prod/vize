//! Public Vite-config route over the same independently pinned authored inputs.
use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, io::Write, path::Path, process::Command};
use vize_l0::cstr;

const RULE: &str = "vue/v-on-event-hyphenation";
const DIRECTORY: &str = "tests/_fixtures/differential/linter/v-on-event-ignore-6097";
const CORPUS: &str =
    include_str!("../../../tests/_fixtures/differential/linter/v-on-event-ignore-6097/cases.json");

fn source(file: &str) -> &'static str {
    match file {
        "ColorPalette.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/v-on-event-ignore-6097/ColorPalette.vue.txt"
        ),
        "Never.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/v-on-event-ignore-6097/Never.vue.txt"
        ),
        _ => panic!("unknown independently pinned event input"),
    }
}
fn write(root: &Path, file: &str, source: &str) {
    let target = root.join(file);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(target, source).unwrap();
}
fn presentation(message: &str) -> (&'static str, &'static str) {
    match message {
        "v-on event '@update:modelValue' must be hyphenated." => (
            "Custom event listeners on components should be hyphenated: use 'update:model-value'",
            "Rename the listener to kebab-case so it matches the emitted event name.",
        ),
        "v-on event '@rowClick' must be hyphenated." => (
            "Custom event listeners on components should be hyphenated: use 'row-click'",
            "Rename the listener to kebab-case so it matches the emitted event name.",
        ),
        "v-on event '@close-window' can't be hyphenated." => (
            "Custom event listeners on components should not be hyphenated: 'close-window'",
            "Use camelCase or a single-word event name instead of kebab-case.",
        ),
        "v-on event '@update:model-value' can't be hyphenated." => (
            "Custom event listeners on components should not be hyphenated: 'update:model-value'",
            "Use camelCase or a single-word event name instead of kebab-case.",
        ),
        _ => panic!("unknown independently captured event finding"),
    }
}
fn expected(row: &Value) -> Value {
    let file = row["reference"]["file"].as_str().unwrap();
    let reference: Value = serde_json::from_str(
        &fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(DIRECTORY)
                .join(file),
        )
        .unwrap(),
    )
    .unwrap();
    let messages = reference["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            let (message, help) = presentation(finding["message"].as_str().unwrap());
            json!({"ruleId":RULE,"ruleDocsPath":"docs/content/rules/vue.md","severity":2,
            "message":cstr!("[vize:{RULE}] {message}"),"help":help,
            "line":finding["line"],"column":finding["column"],
            "endLine":finding["endLine"],"endColumn":finding["endColumn"]})
        })
        .collect::<Vec<_>>();
    json!([{"file":"src/Probe.vue","errorCount":messages.len(),"warningCount":0,"messages":messages}])
}

#[test]
fn event_ignore_vite_config_preserves_thirteen_whole_cli_vectors_and_null_fix_frontier() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let rows = corpus["valid"].as_array().unwrap();
    assert_eq!(rows.len(), 13);
    assert_eq!(
        source("ColorPalette.vue.txt"),
        include_str!(
            "../../../tests/_fixtures/differential/linter/v-on-hyphenation-custom-tags-10822/ColorPalette.vue.txt"
        )
    );
    let project = tempfile::tempdir().unwrap();
    for row in rows {
        let mut config = json!({"linter":{"preset":"incremental","rules":{RULE:"error"}}});
        if !row["policy"].is_null() {
            config["linter"]["ruleOptions"] = json!({RULE:row["policy"]});
        }
        write(
            project.path(),
            "vite.config.mjs",
            &vite_config::source(config.to_string()),
        );
        let source = source(row["file"].as_str().unwrap());
        write(project.path(), "src/Probe.vue", source);
        for fix in [false, true] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
            command.current_dir(project.path()).args([
                "lint",
                "--config",
                "vite.config.mjs",
                "--format",
                "json",
                "src/Probe.vue",
            ]);
            if fix {
                command.arg("--fix");
            }
            let output = command.output().unwrap();
            let report = serde_json::from_slice::<Value>(&output.stdout);
            let packet = serde_json::to_vec(&json!({"id":row["id"],"fix":fix,"status":output.status.code(),"source":source,"stdout":std::str::from_utf8(&output.stdout).unwrap(),"stderr":std::str::from_utf8(&output.stderr).unwrap()})).unwrap();
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(b"vize-v-on-ignore-cli ").unwrap();
            stdout.write_all(&packet).unwrap();
            stdout.write_all(b"\n").unwrap();
            assert_eq!(
                output.status.code(),
                Some(i32::from(row["expectedCount"].as_u64().unwrap() > 0)),
                "{}",
                row["id"]
            );
            assert_eq!(report.unwrap(), expected(row), "{}", row["id"]);
            assert_eq!(
                fs::read_to_string(project.path().join("src/Probe.vue")).unwrap(),
                source
            );
        }
    }
    write(project.path(),"vite.config.mjs",&vite_config::source(json!({"linter":{"preset":"incremental","rules":{RULE:"off"},"ruleOptions":{RULE:{"ignore":[]}}}}).to_string()));
    write(
        project.path(),
        "src/Probe.vue",
        source("ColorPalette.vue.txt"),
    );
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
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!([{"file":"src/Probe.vue","messages":[],"errorCount":0,"warningCount":0}])
    );
    assert_eq!(
        fs::read_to_string(project.path().join("src/Probe.vue")).unwrap(),
        source("ColorPalette.vue.txt")
    );
}
