use super::{RULE, output_details, write_file};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

#[test]
fn registered_authored_inputs_reject_only_the_split_group() {
    let inputs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/repeated-macro-groups-10610");
    let project = tempfile::tempdir().unwrap();
    write_file(
        project.path(),
        "vite.config.mjs",
        &fs::read_to_string(inputs.join("vite.config.mjs")).unwrap(),
    );
    for (name, input) in [
        ("TwoModels.lf.vue", "TwoModels.lf.vue.txt"),
        ("TwoModels.crlf.vue", "TwoModels.crlf.vue.txt"),
        ("SplitGroup.vue", "SplitGroup.vue.txt"),
    ] {
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
        let expected = if name == "SplitGroup.vue" {
            json!([{
                "ruleId":RULE,"ruleDocsPath":"docs/content/rules/type-and-script.md",
                "severity":1,"message":"[vize:script/define-macros-order] `defineModel` should be above `defineProps`.",
                "line":4,"column":1,"endLine":4,"endColumn":66,
                "help":"Declare defineProps and then defineEmits first in <script setup>, right after imports and type declarations."
            }])
        } else {
            json!([])
        };
        let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            actual,
            json!([{"file":name,"messages":expected,"errorCount":0,"warningCount":usize::from(name == "SplitGroup.vue")}]),
            "{}",
            output_details(&output)
        );
    }
}
