use super::{
    CliPathResolver, Decoded, DiagnosticMapper, is_cli_diagnostic_line, is_global_diagnostic_line,
    parse_cli_diagnostic_line,
};
use crate::batch::VirtualProject;
use std::process::Output;

/// The TS backend emits reachability assertions as errors. Accept its nonzero
/// exit only when *every* raw diagnostic is a verified generated warning;
/// unrelated or unmapped backend errors must still fail the check.
pub(super) fn only_pattern_warnings(output: &Output, project: &VirtualProject) -> bool {
    if !matches!(output.status.code(), Some(1 | 2)) {
        return false;
    }
    let mut found = 0;
    let mut mapper = DiagnosticMapper::new(project);
    let mut paths = CliPathResolver::new(project.virtual_root());
    for stream in [&output.stdout, &output.stderr] {
        #[expect(clippy::disallowed_types, reason = "from_utf8_lossy yields std Cow")]
        let text = std::string::String::from_utf8_lossy(stream);
        let mut follows_warning = false;
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if is_global_diagnostic_line(line) {
                return false;
            }
            if !is_cli_diagnostic_line(line) {
                if follows_warning && is_warning_continuation(line) {
                    continue;
                }
                if found > 0 && warning_summary_count(line) == Some(found) {
                    follows_warning = false;
                    continue;
                }
                return false;
            }
            if !line.contains("): error TS2322:") {
                return false;
            }
            let Some(Decoded::Raw {
                raw,
                reachability: true,
            }) = parse_cli_diagnostic_line(line, project, &mut mapper, &mut paths)
            else {
                return false;
            };
            if mapper
                .map_to_original(&raw.virtual_path, raw.line, raw.column)
                .is_none()
            {
                return false;
            }
            found += 1;
            follows_warning = true;
        }
    }
    found > 0
}

// Generated reachability assertions only produce assignability diagnostics.
// Do not accept arbitrary indented output: JS/Go stack traces are indented too.
pub(super) fn is_warning_continuation(line: &str) -> bool {
    line.starts_with("  ")
        && line.trim_start().starts_with("Type '")
        && line.contains(" is not assignable to type ")
        && line.ends_with('.')
}

pub(super) fn warning_summary_count(line: &str) -> Option<usize> {
    let count = line.strip_prefix("Found ")?;
    count
        .strip_suffix(" error.")
        .or_else(|| count.strip_suffix(" errors."))?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::super::parse_output_diagnostics;
    use super::only_pattern_warnings;
    use crate::batch::VirtualProject;
    use std::process::{ExitStatus, Output};
    use vize_carton::cstr;

    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;

    fn warning_output() -> (tempfile::TempDir, VirtualProject, Output) {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("App.vue");
        std::fs::write(&source, r#"<script setup lang="ts">const value = 'a' as 'a' | 'b';</script><template v-match="value"><p v-when="'a'"/><p v-when="'a'"/><p v-when="'b'"/></template>"#).unwrap();
        let mut project = VirtualProject::new(root.path()).unwrap();
        project.set_experimental_patterned_template(true);
        project.register_path(&source).unwrap();
        let file = project
            .virtual_files_sorted()
            .into_iter()
            .find(|file| file.original_path == source)
            .unwrap();
        let (line_number, line) = file
            .content
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains("unreachable: __VizePatterns.Reachable<"))
            .unwrap();
        let column = line.find("const ").unwrap() + "const ".len();
        let stdout = cstr!("{}({},{}): error TS2322: Type 'boolean' is not assignable to type '{{ \"Unreachable v-when\": never; }}'.\n", file.virtual_path.display(), line_number + 1, column + 1).as_bytes().to_vec();
        #[cfg(unix)]
        let status = ExitStatus::from_raw(1 << 8);
        #[cfg(windows)]
        let status = ExitStatus::from_raw(1);
        (
            root,
            project,
            Output {
                status,
                stdout,
                stderr: Vec::new(),
            },
        )
    }

    #[test]
    fn warning_only_output_is_strict_about_unknown_lines_in_either_stream() {
        let (_root, project, output) = warning_output();
        assert!(only_pattern_warnings(&output, &project));
        for extra in [
            "fatal error: out of memory\n",
            "panic: checker failed\n",
            "  at checker.run (checker.js:1:1)\n",
            "goroutine 1 [running]:\n",
            "  unexpected backend output\n",
            "error TS2688: Missing type library.\n",
        ] {
            for stderr in [false, true] {
                let mut output = output.clone();
                if stderr {
                    output.stderr.extend_from_slice(extra.as_bytes());
                } else {
                    output.stdout.extend_from_slice(extra.as_bytes());
                }
                assert!(
                    !only_pattern_warnings(&output, &project),
                    "{extra} stderr={stderr}"
                );
            }
        }
    }

    #[test]
    fn known_continuations_and_summary_are_not_backend_failures() {
        let (_root, project, mut output) = warning_output();
        output.stdout.extend_from_slice(
            b"  Type 'boolean' is not assignable to type 'never'.\n\nFound 1 error.\n",
        );
        assert!(only_pattern_warnings(&output, &project));
        let diagnostics = parse_output_diagnostics(&output, &project, &|_| true);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, 2);
        assert_eq!(
            diagnostics[0].message,
            "Type 'boolean' is not assignable to type '{ \"Unreachable v-when\": never; }'.\nType 'boolean' is not assignable to type 'never'."
        );
    }

    #[test]
    fn fatal_output_is_not_appended_as_a_warning_continuation() {
        let (_root, project, mut output) = warning_output();
        output
            .stdout
            .extend_from_slice(b"fatal error: out of memory\n");
        let diagnostics = parse_output_diagnostics(&output, &project, &|_| true);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, 1);
        assert_eq!(
            diagnostics[0].message,
            "Type 'boolean' is not assignable to type '{ \"Unreachable v-when\": never; }'.\nfatal error: out of memory"
        );
    }
}
