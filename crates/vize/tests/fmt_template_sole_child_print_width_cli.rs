//! #7876: whole original/control CLI streams and three configured fixed points.
#![cfg(feature = "glyph")]
#![expect(clippy::disallowed_macros, reason = "whole expected CLI streams")]
#![expect(clippy::disallowed_types, reason = "filesystem and process fixtures")]
use std::{fs, process::Command};

#[test]
fn original_and_independent_children_keep_three_writes_then_check() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/sole-child-width-7876/references.json"
    )).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 16);
    for case in cases {
        let project = tempfile::tempdir().unwrap();
        let original = case["input"].as_str().unwrap();
        let expected = case["expected"].as_str().unwrap();
        let input = project.path().join("App.vue");
        let config =
            serde_json::to_vec(&serde_json::json!({"formatter": case["options"]})).unwrap();
        let config_path = project.path().join("vize.config.json");
        fs::write(&input, original).unwrap();
        fs::write(&config_path, &config).unwrap();
        for (pass, flag) in ["--check", "--write", "--write", "--write", "--check"]
            .into_iter()
            .enumerate()
        {
            let changed = pass < 2 && original != expected;
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(project.path())
                .args(["fmt", flag, "App.vue"])
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(i32::from(changed && flag == "--check")),
                "{} pass {pass}: {output:?}",
                case["id"]
            );
            assert_eq!(output.stdout, b"");
            let detail = if changed {
                if flag == "--check" {
                    "Would reformat: App.vue\n"
                } else {
                    "Reformatted: App.vue\n"
                }
            } else {
                ""
            };
            let summary = match (flag, changed) {
                ("--check", true) => "Checked 1 file(s)\n  1 file(s) would be reformatted\n",
                ("--check", false) => "Checked 1 file(s)\n  1 file(s) already formatted\n",
                ("--write", true) => "Formatted 1 file(s)\n  1 file(s) reformatted\n",
                _ => "Formatted 1 file(s)\n  1 file(s) unchanged\n",
            };
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                format!("Found 1 file(s)\n{detail}\n{summary}")
            );
            assert_eq!(
                fs::read(&input).unwrap(),
                if pass == 0 { original } else { expected }.as_bytes()
            );
            assert_eq!(fs::read(&config_path).unwrap(), config);
        }
    }
}
