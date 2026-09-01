#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env,
    ffi::OsString,
    fs,
    fs::OpenOptions,
    io::{self, Write},
    process::{Command, ExitCode},
};

const COVERAGE_ARGS: &[&str] = &[
    "run",
    "--profile",
    "ci",
    "-p",
    "vize_test_runner",
    "--bin",
    "coverage",
];

fn main() -> ExitCode {
    ExitCode::from(run_app() as u8)
}

fn run_app() -> i32 {
    let Some(summary_path) = env::var("GITHUB_STEP_SUMMARY")
        .ok()
        .filter(|path| !path.is_empty())
    else {
        println!("GITHUB_STEP_SUMMARY is not set");
        return 1;
    };

    let output = match Command::new(coverage_cargo()).args(COVERAGE_ARGS).output() {
        Ok(output) => output,
        Err(error) => {
            eprintln!("failed to run cargo: {error}");
            return 1;
        }
    };

    if !output.stdout.is_empty() {
        let _ = io::stdout().write_all(&output.stdout);
    }
    if !output.stderr.is_empty() {
        let _ = io::stderr().write_all(&output.stderr);
    }

    if let Err(error) = fs::write("coverage-report.txt", &output.stdout) {
        eprintln!("failed to write coverage-report.txt: {error}");
        return 1;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Err(error) = append_summary(&summary_path, &coverage_summary(&stdout)) {
        eprintln!("failed to append {summary_path}: {error}");
        return 1;
    }

    output.status.code().unwrap_or(1)
}

fn coverage_cargo() -> OsString {
    env::var_os("VIZE_COVERAGE_CARGO").unwrap_or_else(|| "cargo".into())
}

fn coverage_summary(stdout: &str) -> String {
    let summary = last_lines(stdout, 7);
    if summary.is_empty() {
        "### Coverage Summary\n\n".to_string()
    } else {
        format!("### Coverage Summary\n{summary}\n")
    }
}

fn append_summary(path: &str, text: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(text.as_bytes())
}

fn last_lines(text: &str, count: usize) -> String {
    let trimmed = text.trim_end_matches('\n');
    if trimmed.is_empty() {
        return String::new();
    }
    let lines = trimmed.split('\n').collect::<Vec<_>>();
    let start = lines.len().saturating_sub(count);
    lines[start..].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_last_requested_lines() {
        assert_eq!(last_lines("", 7), "");
        assert_eq!(last_lines("one\ntwo\n", 7), "one\ntwo");
        assert_eq!(
            last_lines(
                "head\nline-1\nline-2\nline-3\nline-4\nline-5\nline-6\nline-7\n",
                7
            ),
            "line-1\nline-2\nline-3\nline-4\nline-5\nline-6\nline-7"
        );
    }

    #[test]
    fn renders_empty_and_populated_summaries() {
        assert_eq!(coverage_summary(""), "### Coverage Summary\n\n");
        assert_eq!(
            coverage_summary("Coverage report\nline-1\nline-2\n"),
            "### Coverage Summary\nCoverage report\nline-1\nline-2\n"
        );
    }
}
