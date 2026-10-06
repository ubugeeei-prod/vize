//! #7924: original no-config writes converge and checks never change the input.
#![expect(
    clippy::disallowed_macros,
    reason = "independently authored whole CLI streams"
)]
#![expect(
    clippy::disallowed_types,
    reason = "whole filesystem and process fixtures"
)]
use std::{fs, process::Command};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-leading-line-comment-directive/Min.vue.txt"
);
const EXPECTED: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-leading-line-comment-directive/reference.expected.txt"
);

#[test]
fn original_write_write_write_check_is_stable_without_project_configuration() {
    for ignored_config in [false, true] {
        let project = tempfile::tempdir().unwrap();
        let input = project.path().join("Min.vue");
        fs::write(&input, ORIGINAL).unwrap();
        let config = "{\"formatter\":{\"tabWidth\":9}}\n";
        if ignored_config {
            fs::write(project.path().join("vize.config.json"), config).unwrap();
        }
        for (flag, changed, code, expected_bytes) in [
            ("--check", true, 1, ORIGINAL),
            ("--write", true, 0, EXPECTED),
            ("--write", false, 0, EXPECTED),
            ("--write", false, 0, EXPECTED),
            ("--check", false, 0, EXPECTED),
        ] {
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(project.path())
                .args(["fmt", flag, "--no-config", "Min.vue"])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(code), "{output:?}");
            assert_eq!(output.stdout, b"");
            let detail = match (flag, changed) {
                ("--check", true) => "Would reformat: Min.vue\n",
                ("--write", true) => "Reformatted: Min.vue\n",
                _ => "",
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
            assert_eq!(fs::read(&input).unwrap(), expected_bytes.as_bytes());
            if ignored_config {
                assert_eq!(
                    fs::read(project.path().join("vize.config.json")).unwrap(),
                    config.as_bytes()
                );
            }
        }
    }
}
