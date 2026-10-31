//! Real CLI fixing over the whole independently pinned original and boundary inputs.

use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, io::Write, path::PathBuf, process::Command};

const RULE: &str = "vue/v-on-event-hyphenation";

fn directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/v-on-event-autofix-6097")
}

fn presentation(finding: &Value) -> (String, &'static str) {
    let caption = finding["message"].as_str().unwrap();
    let key = caption.split('\'').nth(1).unwrap();
    let name = key
        .strip_prefix('@')
        .or_else(|| key.strip_prefix("v-on:"))
        .unwrap()
        .split('.')
        .next()
        .unwrap();
    if finding["messageId"] == "cannotBeHyphenated" {
        return (
            format!("Custom event listeners on components should not be hyphenated: '{name}'"),
            "Use camelCase or a single-word event name instead of kebab-case.",
        );
    }
    // Existing English presentation, independent of the production converter.
    let suggestion = match name {
        "Change" => "change",
        "dropdownVisibleChange" => "dropdown-visible-change",
        "update:modelValue" | "update:ModelValue" => "update:model-value",
        "pressEnter" => "press-enter",
        "node_Expand" => "node-expand",
        "XMLParsed" => "x-m-l-parsed",
        "vue:beforeMount" => "vue:before-mount",
        "rowClick" => "row-click",
        "shapeMove" => "shape-move",
        "Événement" => "événement",
        "fooÉvent" => "fooévent",
        "foo-Évent" | "foo_Évent" => "foo-évent",
        "AΣ" => "aς",
        _ => panic!("unrecorded authored event name: {name}"),
    };
    (
        format!("Custom event listeners on components should be hyphenated: use '{suggestion}'"),
        "Rename the listener to kebab-case so it matches the emitted event name.",
    )
}

fn expected(findings: &Value) -> Value {
    let messages: Vec<_> = findings.as_array().unwrap().iter().map(|finding| {
        let (message, help) = presentation(finding);
        json!({"ruleId":RULE,"ruleDocsPath":"docs/content/rules/vue.md","severity":2,
            "message":format!("[vize:{RULE}] {message}"),"help":help,
            "line":finding["line"],"column":finding["column"],"endLine":finding["endLine"],"endColumn":finding["endColumn"]})
    }).collect();
    json!([{"file":"src/Probe.vue","errorCount":messages.len(),"warningCount":0,"messages":messages}])
}

#[test]
fn vite_config_cli_applies_only_requested_arguments_and_stops_on_identity_edits() {
    let root = directory();
    let corpus: Value =
        serde_json::from_str(&fs::read_to_string(root.join("cases.json")).unwrap()).unwrap();
    let rows = corpus["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 22);
    let project = tempfile::tempdir().unwrap();
    fs::create_dir(project.path().join("src")).unwrap();
    let path = project.path().join("src/Probe.vue");
    for row in rows {
        let options = row["options"].as_array().unwrap();
        let mut config = json!({"linter":{"preset":"incremental","rules":{RULE:"error"}}});
        if let Some(style) = options.first() {
            let value = if let Some(payload) = options.get(1) {
                let mut payload = payload.as_object().unwrap().clone();
                payload.insert("style".into(), style.clone());
                Value::Object(payload)
            } else {
                style.clone()
            };
            config["linter"]["ruleOptions"] = json!({RULE:value});
        }
        fs::write(
            project.path().join("vite.config.mjs"),
            vite_config::source(config.to_string()),
        )
        .unwrap();
        let original = fs::read_to_string(root.join(row["file"].as_str().unwrap())).unwrap();
        let reference: Value = serde_json::from_str(
            &fs::read_to_string(root.join(row["reference"]["file"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        fs::write(&path, &original).unwrap();
        let mut previous_modified = None;
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
            if pass > 0 {
                command.arg("--fix");
            }
            let output = command.output().unwrap();
            let actual_source = fs::read_to_string(&path).unwrap();
            let findings = if pass == 0 {
                &reference["messages"]
            } else {
                &reference["requery"]
            };
            let packet = serde_json::to_vec(&json!({"id":row["id"],"pass":pass,"status":output.status.code(),"source":actual_source,
                "stdout":std::str::from_utf8(&output.stdout).unwrap(),"stderr":std::str::from_utf8(&output.stderr).unwrap()})).unwrap();
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(b"vize-v-on-autofix-cli ").unwrap();
            stdout.write_all(&packet).unwrap();
            stdout.write_all(b"\n").unwrap();
            assert_eq!(output.stderr, b"", "{} pass{pass}", row["id"]);
            assert_eq!(
                output.status.code(),
                Some(i32::from(!findings.as_array().unwrap().is_empty())),
                "{} pass{pass}",
                row["id"]
            );
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                expected(findings),
                "{} pass{pass}",
                row["id"]
            );
            assert_eq!(
                actual_source,
                if pass == 0 {
                    original.as_str()
                } else {
                    reference["fixResult"]["output"].as_str().unwrap()
                },
                "{} pass{pass}",
                row["id"]
            );
            let modified = fs::metadata(&path).unwrap().modified().unwrap();
            if pass == 2 {
                assert_eq!(
                    Some(modified),
                    previous_modified,
                    "identity/no-op must not rewrite the file"
                );
            }
            previous_modified = Some(modified);
        }
    }
}
