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
    let diagnostics = parse_output_diagnostics(&output, &project);
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
    let diagnostics = parse_output_diagnostics(&output, &project);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, 1);
    assert_eq!(
        diagnostics[0].message,
        "Type 'boolean' is not assignable to type '{ \"Unreachable v-when\": never; }'.\nfatal error: out of memory"
    );
}
