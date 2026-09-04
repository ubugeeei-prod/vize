#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env,
    process::{Command, ExitCode, ExitStatus},
    thread,
    time::Duration,
};

const DEFAULT_ATTEMPTS: u32 = 3;
const DEFAULT_RETRY_DELAY_MS: u64 = 20_000;

fn main() -> ExitCode {
    let attempts = env_u32("VIZE_NPM_AUDIT_ATTEMPTS", DEFAULT_ATTEMPTS).max(1);
    let retry_delay_ms = env_u64("VIZE_NPM_AUDIT_RETRY_DELAY_MS", DEFAULT_RETRY_DELAY_MS);

    for attempt in 1..=attempts {
        let status = match run_audit() {
            Ok(status) if status.success() => return ExitCode::SUCCESS,
            Ok(status) => status,
            Err(error) => {
                eprintln!("npm-audit: failed to run vp: {error}");
                return ExitCode::from(1);
            }
        };

        if attempt == attempts {
            return exit_code(status);
        }

        let delay = retry_delay_ms * u64::from(attempt);
        eprintln!(
            "npm-audit: attempt {attempt} failed; retrying in {}s...",
            delay / 1_000
        );
        thread::sleep(Duration::from_millis(delay));
    }

    ExitCode::from(1)
}

fn run_audit() -> Result<ExitStatus, std::io::Error> {
    Command::new("vp")
        .args([
            "exec",
            "pnpm",
            "audit",
            "--prod",
            "--audit-level",
            "moderate",
        ])
        .status()
}

fn env_u32(name: &str, default_value: u32) -> u32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default_value)
}

fn env_u64(name: &str, default_value: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default_value)
}

fn exit_code(status: ExitStatus) -> ExitCode {
    match status.code().and_then(|code| u8::try_from(code).ok()) {
        Some(code) => ExitCode::from(code),
        None => ExitCode::from(1),
    }
}
