//! Typed native CLI outcomes distinguish required semantic completion from failure.

use super::{CorsaError, TypeCheckResult, is_cli_diagnostic_line};
use std::{io::Write, process::Output};

pub(in crate::batch::executor) type CliCheckResult = Result<TypeCheckResult, CliCheckFailure>;

#[derive(Debug)]
pub(in crate::batch::executor) enum CliCheckFailure {
    /// The native CLI diagnosed syntax and stopped before semantic checking.
    SemanticCompletionRequired(CorsaError),
    /// An unsupported, unparseable or unavailable native runtime path.
    Runtime(CorsaError),
}

impl CliCheckFailure {
    pub(in crate::batch::executor) fn into_error(self) -> CorsaError {
        match self {
            Self::SemanticCompletionRequired(error) | Self::Runtime(error) => error,
        }
    }
}

/// Only the ordinary complete parser packet is a required semantic handoff.
/// A crash, extra output or undecodable byte remains observable degradation.
pub(super) fn classify_parser_abort(error: CorsaError, output: &Output) -> CliCheckFailure {
    let expected = output.status.code() == Some(1)
        && output.stderr.is_empty()
        && std::str::from_utf8(&output.stdout).is_ok_and(|text| {
            text.lines()
                .filter(|line| !line.trim().is_empty())
                .all(|line| {
                    is_cli_diagnostic_line(line)
                        && line
                            .split_once("): ")
                            .is_some_and(|(_, message)| message.starts_with("error TS"))
                })
        });
    if expected {
        CliCheckFailure::SemanticCompletionRequired(error)
    } else {
        let mut stderr = std::io::stderr().lock();
        let replay = stderr
            .write_all(&output.stdout)
            .and_then(|()| stderr.write_all(&output.stderr));
        if let Err(replay_error) = replay {
            tracing::warn!(error = %error, replay_error = %replay_error, "failed to replay unexpected native parser packet");
        }
        CliCheckFailure::Runtime(error)
    }
}

impl From<CorsaError> for CliCheckFailure {
    fn from(error: CorsaError) -> Self {
        Self::Runtime(error)
    }
}

impl From<std::io::Error> for CliCheckFailure {
    fn from(error: std::io::Error) -> Self {
        Self::Runtime(CorsaError::Io(error))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::batch::{VirtualProject, executor::cli::check_with_cli_classified};
    use std::{fs, os::unix::fs::PermissionsExt};
    use vize_carton::cstr;

    fn fixture(
        source: &str,
        output: &str,
        exit: i32,
    ) -> (tempfile::TempDir, VirtualProject, std::path::PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fs::write(root.join("source.ts"), source).unwrap();
        let mut project = VirtualProject::new(&root).unwrap();
        project.register_path(&root.join("source.ts")).unwrap();
        project.materialize().unwrap();
        let executable = root.join("native-control.sh");
        fs::write(
            &executable,
            cstr!("#!/bin/sh\nprintf '%s\\n' '{output}'\nexit {exit}\n"),
        )
        .unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        (temp, project, executable)
    }

    #[test]
    fn parser_abort_is_typed_completion_without_changing_the_whole_refusal() {
        let (_temp, project, runtime) = fixture(
            "const value = ;\n",
            "source.ts(1,15): error TS1109: Expression expected.",
            1,
        );
        let error = check_with_cli_classified(&runtime, &project, 1).unwrap_err();
        let CliCheckFailure::SemanticCompletionRequired(error) = error else {
            panic!("parser abort must require same-config semantics: {error:?}");
        };
        let CorsaError::CorsaExecution { exit_code, message } = error else {
            panic!("the complete execution refusal must remain unchanged");
        };
        assert_eq!(exit_code, 1);
        assert_eq!(
            message,
            "Corsa CLI semantic diagnostics are incomplete after a syntax error:\nsource.ts(1,15): error TS1109: Expression expected."
        );
    }

    #[test]
    fn parser_packets_with_abnormal_status_or_extra_bytes_remain_runtime_failures() {
        const DIAGNOSTIC: &str = "source.ts(1,15): error TS1109: Expression expected.";
        for (exit, extra_stdout, stderr_script, expected_output) in [
            (17, "", "", DIAGNOSTIC),
            (
                1,
                "\nnative transport failed",
                "",
                "source.ts(1,15): error TS1109: Expression expected.\nnative transport failed",
            ),
            (
                1,
                "",
                "printf 'unexpected native stderr\\n' >&2",
                "unexpected native stderr\nsource.ts(1,15): error TS1109: Expression expected.",
            ),
            (
                1,
                "",
                "printf '\\000\\377' >&2",
                "\0�\nsource.ts(1,15): error TS1109: Expression expected.",
            ),
        ] {
            let (_temp, project, runtime) = fixture(
                "const value = ;\n",
                &cstr!("{DIAGNOSTIC}{extra_stdout}"),
                exit,
            );
            let script = fs::read_to_string(&runtime).unwrap();
            fs::write(
                &runtime,
                script.replace(
                    cstr!("exit {exit}").as_str(),
                    &cstr!("{stderr_script}\nexit {exit}"),
                ),
            )
            .unwrap();
            let error = check_with_cli_classified(&runtime, &project, 1).unwrap_err();
            let CliCheckFailure::Runtime(error) = error else {
                panic!("an abnormal parser packet cannot silently hand off: {error:?}");
            };
            let CorsaError::CorsaExecution { exit_code, message } = error else {
                panic!("original complete parser refusal must remain unchanged");
            };
            assert_eq!(exit_code, exit);
            assert_eq!(
                message,
                cstr!(
                    "Corsa CLI semantic diagnostics are incomplete after a syntax error:\n{expected_output}"
                )
            );
        }
    }

    #[test]
    fn parser_abort_in_one_shard_cannot_hide_another_shards_runtime_failure() {
        let (_temp, mut project, runtime) = fixture(
            "const value = ;\n",
            "source.ts(1,15): error TS1109: Expression expected.",
            1,
        );
        for name in ["A.vue", "B.vue"] {
            let path = project.project_root().join(name);
            fs::write(&path, "<script setup lang=\"ts\">const n = 1</script>").unwrap();
            project.register_path(&path).unwrap();
        }
        project.materialize().unwrap();
        let original = fs::read_to_string(&runtime).unwrap();
        fs::write(
            &runtime,
            original.replace(
                "#!/bin/sh\n",
                "#!/bin/sh\ncase \"$*\" in *tsconfig.shard1.json*) printf 'native unavailable'; exit 41;; esac\n",
            ),
        ).unwrap();
        let failure = super::super::check_with_cli_sharded(&runtime, &project, 2, 1).unwrap_err();
        let CliCheckFailure::Runtime(CorsaError::CorsaExecution { exit_code, message }) = failure
        else {
            panic!("a runtime failure must take precedence over a parser handoff: {failure:?}");
        };
        assert_eq!(exit_code, 41);
        assert_eq!(message, "native unavailable");
    }

    #[test]
    fn unexpected_runtime_failure_keeps_its_complete_error_and_degradation_kind() {
        let (_temp, project, runtime) =
            fixture("const value = 1;\n", "native transport unavailable", 41);
        let error = check_with_cli_classified(&runtime, &project, 1).unwrap_err();
        let CliCheckFailure::Runtime(error) = error else {
            panic!("unexpected failure must remain observable runtime degradation: {error:?}");
        };
        let CorsaError::CorsaExecution { exit_code, message } = error else {
            panic!("whole runtime refusal must retain its execution identity");
        };
        assert_eq!(exit_code, 41);
        assert_eq!(message, "native transport unavailable");
    }

    #[test]
    fn ordinary_semantic_error_stays_on_the_cli_path_and_clean_control_stays_clean() {
        let (_temp, project, runtime) = fixture(
            "const value: number = 'bad';\n",
            "source.ts(1,1): error TS2322: semantic control",
            1,
        );
        let result = check_with_cli_classified(&runtime, &project, 1).unwrap();
        let rows: Vec<_> = result
            .diagnostics
            .iter()
            .map(|row| {
                (
                    row.file.clone(),
                    row.line,
                    row.column,
                    row.code,
                    row.severity,
                    row.message.as_str(),
                    row.block_type,
                )
            })
            .collect();
        assert_eq!(
            rows,
            vec![(
                project.project_root().join("source.ts"),
                0,
                0,
                Some(2322),
                1,
                "semantic control",
                None
            )]
        );
        assert_eq!((result.success, result.exit_code), (false, 1));
        let (_clean_temp, clean_project, clean_runtime) = fixture("const value = 1;\n", "", 0);
        let clean = check_with_cli_classified(&clean_runtime, &clean_project, 1).unwrap();
        assert_eq!(
            (clean.success, clean.exit_code, clean.diagnostics.len()),
            (true, 0, 0)
        );
    }
}
