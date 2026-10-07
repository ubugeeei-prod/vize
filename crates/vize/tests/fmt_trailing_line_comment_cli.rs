//! #7929: public no-config writes and checks retain both complete conditions.
#![expect(clippy::disallowed_macros, reason = "whole expected CLI streams")]
#![expect(clippy::disallowed_types, reason = "filesystem and process fixtures")]
use std::{fs, process::Command};

const A: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/script-trailing-line-comment-parens/a.ts.txt"
);
const B: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/script-trailing-line-comment-parens/b.ts.txt"
);
const A_CRLF: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/script-trailing-line-comment-parens/a.ts.crlf.txt"
);
const B_CRLF: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/script-trailing-line-comment-parens/b.ts.crlf.txt"
);

#[test]
fn both_original_conditions_write_three_times_then_check_without_redundant_parens() {
    for (filename, original, windows) in [("a.ts", A, A_CRLF), ("b.ts", B, B_CRLF)] {
        for crlf in [false, true] {
            let project = tempfile::tempdir().unwrap();
            let input = project.path().join(filename);
            fs::write(&input, if crlf { windows } else { original }).unwrap();
            for (pass, flag) in ["--check", "--write", "--write", "--write", "--check"]
                .into_iter()
                .enumerate()
            {
                let changed = crlf && pass < 2;
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
                    if crlf && pass == 0 {
                        windows.as_bytes()
                    } else {
                        original.as_bytes()
                    }
                );
            }
        }
    }
}
