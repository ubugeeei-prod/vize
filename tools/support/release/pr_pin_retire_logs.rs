//! Raw job logs are archived byte-for-byte, never rendered to a terminal.
use std::{path::Path, process::Command};

const ESCAPE_REFUSAL: &str = "the response contains terminal escape sequences; pass --allow-escape-sequences to output it anyway";

fn command(repository: &str, job: u64, root: &Path) -> Command {
    let mut command = Command::new("gh");
    command
        .args([
            "api",
            &format!("repos/{repository}/actions/jobs/{job}/logs"),
            "--allow-escape-sequences",
        ])
        .current_dir(root);
    command
}

pub(super) fn capture(repository: &str, job: u64, root: &Path) -> Result<Vec<u8>, String> {
    capture_command(command(repository, job, root), job)
}

fn capture_command(mut command: Command, job: u64) -> Result<Vec<u8>, String> {
    let output = command.output().map_err(|error| {
        format!(
            "Could not start complete failure log capture {job}: {:?}",
            error.kind()
        )
    })?;
    if !output.status.success() || output.stdout.is_empty() {
        // Only known static classifications are public. Raw CLI diagnostics may contain
        // credentials or short-lived signed download URLs and must never be emitted.
        let diagnostic = if output.stderr.trim_ascii() == ESCAPE_REFUSAL.as_bytes() {
            ESCAPE_REFUSAL
        } else if output.status.success() {
            "empty complete job log"
        } else {
            "unrecognized GitHub CLI failure; raw diagnostics withheld"
        };
        return Err(format!(
            "Could not preserve complete failure log {job}: {}; stdout={} bytes; stderr={} bytes; diagnostic={}",
            output.status,
            output.stdout.len(),
            output.stderr.len(),
            serde_json::to_string(diagnostic).map_err(|error| error.to_string())?,
        ));
    }
    Ok(output.stdout)
}

#[cfg(test)]
#[path = "pr_pin_retire_log_tests.rs"]
mod tests;
