#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env,
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    if let Some(extra) = env::args_os().nth(1) {
        eprintln!(
            "Usage: rust-script tools/commands/ci/github/check-maestro-feature-contract.rs; unexpected argument {}",
            extra.to_string_lossy()
        );
        return ExitCode::from(1);
    }

    for args in [
        &["check", "-p", "vize_maestro", "--no-default-features"][..],
        &[
            "test",
            "-p",
            "vize_maestro",
            "--no-default-features",
            "--test",
            "non_native_structural",
        ][..],
        &[
            "check",
            "-p",
            "vize_maestro",
            "--no-default-features",
            "--features",
            "glyph",
        ][..],
        &[
            "test",
            "-p",
            "vize_maestro",
            "--no-default-features",
            "--features",
            "glyph",
            "--test",
            "non_native_structural",
        ][..],
    ] {
        let status = Command::new("cargo")
            .args(args)
            .env("RUSTFLAGS", "-D warnings")
            .status();
        let Ok(status) = status else {
            eprintln!(
                "failed to run cargo {}: {}",
                args.join(" "),
                status.unwrap_err()
            );
            return ExitCode::from(1);
        };
        if !status.success() {
            return ExitCode::from(status.code().unwrap_or(1).clamp(0, 255) as u8);
        }
    }

    ExitCode::SUCCESS
}
