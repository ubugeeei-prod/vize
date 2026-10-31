use super::{RULE, output_details, write_file};
use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

#[test]
fn registered_expose_last_policy_keeps_the_original_source_until_manual_repair() {
    let inputs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/define-expose-last-6097");
    let original = fs::read_to_string(inputs.join("ExposeBefore.vue.txt")).unwrap();
    let repaired = fs::read_to_string(inputs.join("ExposeLast.vue.txt")).unwrap();
    let project = tempfile::tempdir().unwrap();
    let mut observations = Vec::new();
    for (phase, source, enabled, fix) in [
        ("enabled", original.clone(), true, false),
        ("automatic-fix-stays-manual", original.clone(), true, true),
        ("disabled", original.clone(), false, false),
        (
            "ordinary-script",
            original.replace("<script setup", "<script"),
            true,
            false,
        ),
        ("manual-repair", repaired.clone(), true, false),
        ("repair-fixed-point", repaired, true, false),
    ] {
        write_file(project.path(), "Expose.vue", &source);
        write_file(
            project.path(),
            "vite.config.mjs",
            &vite_config::source(
                json!({
                    "linter": {
                        "preset":"incremental", "typeAware":false,
                        "rules":{RULE:"error"},
                        "ruleOptions":{RULE:{"defineExposeLast":enabled}}
                    }
                })
                .to_string(),
            ),
        );
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command.current_dir(project.path()).args([
            "lint",
            "--config",
            "vite.config.mjs",
            "--format",
            "json",
            "Expose.vue",
        ]);
        if fix {
            command.arg("--fix");
        }
        let output = command.output().unwrap();
        let bytes_after = fs::read(project.path().join("Expose.vue")).unwrap();
        observations.push((phase, source, output, bytes_after));
    }
    for (phase, source, output, bytes_after) in observations {
        println!("vize-expose-last-cli {phase} {}", output_details(&output));
        let violation = matches!(phase, "enabled" | "automatic-fix-stays-manual");
        assert_eq!(
            output.status.code(),
            Some(i32::from(violation)),
            "{phase}: {}",
            output_details(&output)
        );
        assert_eq!(output.stderr, b"", "{phase}: {}", output_details(&output));
        assert_eq!(
            bytes_after,
            source.as_bytes(),
            "{phase}: manual-only finding must not edit input"
        );
        let messages = if violation {
            json!([{
                "ruleId":RULE, "ruleDocsPath":"docs/content/rules/type-and-script.md", "severity":2,
                "message":"[vize:script/define-macros-order] `defineExpose` should be the last statement in `<script setup>`.",
                "line":3, "column":1, "endLine":3, "endColumn":25,
                "help":"Put defineExpose as the last statement in <script setup>."
            }])
        } else {
            json!([])
        };
        let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            actual,
            json!([{"file":"Expose.vue","messages":messages,"errorCount":usize::from(violation),"warningCount":0}]),
            "{phase}"
        );
    }
}
