#!/usr/bin/env rust-script
//! Compatibility entrypoint. Release workflows execute the TypeScript source directly.
use std::{
    env,
    process::{Command, ExitCode},
};
fn main() -> ExitCode {
    match Command::new("node")
        .arg("tools/support/github/release-platforms.ts")
        .args(env::args().skip(1))
        .status()
    {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
