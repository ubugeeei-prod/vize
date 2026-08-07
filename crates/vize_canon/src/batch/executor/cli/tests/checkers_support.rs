//! `--checkers` pinning behaviour of the CLI runner (#3905).

use super::unique_case_dir;
use crate::batch::VirtualProject;
use crate::batch::error::CorsaError;
use std::fs;

/// A runtime that rejects `--checkers` must fail the run. Retrying without the
/// option would check the project at Corsa's default checker width, whose
/// diagnostic set differs from the pinned one-checker oracle (#3905).
#[cfg(unix)]
#[test]
fn corsa_without_checkers_support_fails_instead_of_retrying() {
    use super::super::run_cli_for_config;
    use std::os::unix::fs::PermissionsExt;

    let case_dir = unique_case_dir("checkers-unsupported");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&case_dir).unwrap();
    // Stub Corsa: rejects `--checkers` the way an older runtime does, and
    // succeeds without it, so the failure can only come from an invocation that
    // actually passes the option.
    let stub = case_dir.join("corsa-stub.sh");
    fs::write(
        &stub,
        r#"#!/bin/sh
for arg in "$@"; do
  if [ "$arg" = "--checkers" ]; then
    echo "error TS5023: Unknown compiler option '--checkers'."
    exit 1
  fi
done
exit 0
"#,
    )
    .unwrap();
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).unwrap();

    let project = VirtualProject::new(&case_dir).unwrap();
    fs::create_dir_all(project.virtual_root()).unwrap();
    let config_path = project.virtual_root().join("tsconfig.json");
    fs::write(&config_path, "{}\n").unwrap();

    let error = run_cli_for_config(&stub, &project, &config_path, 1).unwrap_err();
    match error {
        CorsaError::CorsaExecution { exit_code, message } => {
            assert_eq!(exit_code, 1);
            assert!(
                message.contains("does not support `--checkers`"),
                "expected an unsupported-`--checkers` failure, got: {message}"
            );
        }
        other => panic!("expected a corsa execution failure, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&case_dir);
}
