use super::{RULE, output_details, write_file};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

#[test]
fn registered_authored_inputs_keep_macro_group_and_ambient_barrier_diagnostics() {
    let inputs =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/differential/linter");
    for (family, name, input, violation) in [
        (
            "repeated-macro-groups-10610",
            "TwoModels.lf.vue",
            "TwoModels.lf.vue.txt",
            None,
        ),
        (
            "repeated-macro-groups-10610",
            "TwoModels.crlf.vue",
            "TwoModels.crlf.vue.txt",
            None,
        ),
        (
            "repeated-macro-groups-10610",
            "SplitGroup.vue",
            "SplitGroup.vue.txt",
            Some((66, super::ABOVE)),
        ),
        (
            "ambient-macro-declarations-10656",
            "AmbientPrefix.vue",
            "AmbientPrefix.vue.txt",
            None,
        ),
        (
            "ambient-macro-declarations-10656",
            "AmbientBarrier.vue",
            "AmbientBarrier.vue.txt",
            Some((30, super::MODEL_TOP)),
        ),
    ] {
        let inputs = inputs.join(family);
        let project = tempfile::tempdir().unwrap();
        write_file(
            project.path(),
            "vite.config.mjs",
            &fs::read_to_string(inputs.join("vite.config.mjs")).unwrap(),
        );
        write_file(
            project.path(),
            name,
            &fs::read_to_string(inputs.join(input)).unwrap(),
        );
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .args([
                "lint",
                "--config",
                "vite.config.mjs",
                "--format",
                "json",
                name,
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{}", output_details(&output));
        let expected = if let Some((end_column, message)) = violation {
            json!([{
                "ruleId":RULE,"ruleDocsPath":"docs/content/rules/type-and-script.md",
                "severity":1,"message":format!("[vize:{RULE}] {message}"),
                "line":4,"column":1,"endLine":4,"endColumn":end_column,
                "help":"Declare defineProps and then defineEmits first in <script setup>, right after imports and type declarations."
            }])
        } else {
            json!([])
        };
        let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            actual,
            json!([{"file":name,"messages":expected,"errorCount":0,"warningCount":usize::from(violation.is_some())}]),
            "{}",
            output_details(&output)
        );
    }
}
