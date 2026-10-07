//! #7868: actual CLI check/write/write/write/check hugs typed last-argument arrows.
#![expect(clippy::disallowed_macros, reason = "whole expected CLI streams")]
#![expect(clippy::disallowed_types, reason = "filesystem and process fixtures")]
use std::{fs, process::Command};

const TS: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/a.ts.txt"
);
const TS_CRLF: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/a.ts.crlf.txt"
);
const TS_EXPECTED: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/a.reference.expected.txt"
);
const SFC: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/App.vue.txt"
);
const SFC_CRLF: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/App.vue.crlf.txt"
);
const SFC_EXPECTED: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/reference.expected.txt"
);

#[test]
fn original_ts_and_sfc_hug_after_one_write_and_remain_stable_through_check() {
    for (filename, original, windows, expected) in [
        ("a.ts", TS, TS_CRLF, TS_EXPECTED),
        ("App.vue", SFC, SFC_CRLF, SFC_EXPECTED),
    ] {
        for crlf in [false, true] {
            let project = tempfile::tempdir().unwrap();
            let input = project.path().join(filename);
            fs::write(&input, if crlf { windows } else { original }).unwrap();
            for (pass, flag) in ["--check", "--write", "--write", "--write", "--check"]
                .into_iter()
                .enumerate()
            {
                let changed = pass < 2;
                let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                    .current_dir(project.path())
                    .args(["fmt", flag, "--no-config", filename])
                    .output()
                    .unwrap();
                let code = i32::from(changed && flag == "--check");
                assert_eq!(output.status.code(), Some(code), "{output:?}");
                assert_eq!(output.stdout, b"");
                let detail = if changed {
                    if flag == "--check" {
                        format!("Would reformat: {filename}\n")
                    } else {
                        format!("Reformatted: {filename}\n")
                    }
                } else {
                    String::new()
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
                    if pass == 0 {
                        if crlf {
                            windows.as_bytes()
                        } else {
                            original.as_bytes()
                        }
                    } else {
                        expected.as_bytes()
                    }
                );
            }
        }
    }
}
