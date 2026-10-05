//! Source-built invocation for the unchanged fallthrough CLI corpus.
#![cfg(test)]
use std::path::Path;
use std::process::Command;

pub(super) fn run_check_json(project_root: &Path, corsa_path: &Path) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success() || (output.status.code() == Some(1) && !stdout.trim().is_empty()),
        "check crashed\nstdout:\n{}\nstderr:\n{}",
        stdout,
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
    );
    serde_json::from_str(stdout).unwrap()
}
