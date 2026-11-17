//! Declaration-portability diagnostics of a mirror declaration emit.
//!
//! A mirror emit runs with `noCheck` and only writes declarations another
//! program of the same check reads in place of this program's sources. Its
//! remaining errors say that a declaration spells an inferred type through a
//! path a published package could not name (`TS2742`/`TS2883`, `TS2527`, the
//! `TS4xxx` private-name family). TypeScript still writes those declarations,
//! and the consumer resolves the spelled path on this machine. The producing
//! program's own check reports its real diagnostics, so these must not abort
//! the whole check; any other failure still does.

use std::process::Output;

use super::cli::is_cli_diagnostic_line;

/// Whether a failed mirror emit reported only declaration-portability errors.
pub(super) fn only_declaration_portability_errors(output: &Output) -> bool {
    if output.status.code() != Some(1) || !output.stderr.is_empty() {
        return false;
    }
    let Ok(stdout) = std::str::from_utf8(&output.stdout) else {
        return false;
    };
    let mut lines = stdout.lines().filter(|line| !line.trim().is_empty());
    let mut any = false;
    lines.all(|line| {
        any = true;
        is_cli_diagnostic_line(line)
            && line
                .split_once("): error TS")
                .and_then(|(_, rest)| rest.split_once(':'))
                .and_then(|(code, _)| code.parse::<u32>().ok())
                .is_some_and(is_declaration_portability_code)
    }) && any
}

fn is_declaration_portability_code(code: u32) -> bool {
    matches!(code, 2527 | 2742 | 2883 | 4000..=4125 | 9005 | 9006)
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

    use super::only_declaration_portability_errors;

    fn output(code: i32, stdout: &str, stderr: &str) -> Output {
        Output {
            status: ExitStatus::from_raw(code << 8),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[test]
    fn non_portable_inferred_types_do_not_fail_a_mirror_emit() {
        let packet = "src/components/Shell.vue.ts(23,10): error TS2883: The inferred type of '__setup' cannot be named without a reference to 'DebouncedFuncLeading' from '../node_modules/.pnpm/@types+lodash@4.17.23/node_modules/@types/lodash'. This is likely not portable. A type annotation is necessary.\n\
            src/a.ts(1,1): error TS2742: The inferred type of 'a' cannot be named without a reference to 'b'.\n\
            src/b.ts(2,3): error TS4025: Exported variable 'b' has or is using private name 'C'.\n";
        assert!(only_declaration_portability_errors(&output(1, packet, "")));
    }

    #[test]
    fn other_failures_still_fail_a_mirror_emit() {
        let portable = "src/a.ts(1,1): error TS2742: The inferred type of 'a' cannot be named.\n";
        for (code, stdout, stderr) in [
            (1, "", ""),
            (2, portable, ""),
            (1, portable, "fatal\n"),
            (1, "error TS5083: Cannot read file 'tsconfig.json'.\n", ""),
            (
                1,
                "src/a.ts(1,1): error TS2742: x\nsrc/c.ts(1,1): error TS2307: Cannot find module 'd'.\n",
                "",
            ),
            (1, "panic: runtime error\n", ""),
        ] {
            assert!(
                !only_declaration_portability_errors(&output(code, stdout, stderr)),
                "{code} {stdout:?} {stderr:?}"
            );
        }
    }
}
