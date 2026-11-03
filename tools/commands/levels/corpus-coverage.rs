#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! vize_tool_support = { path = "../../support" }
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::{env, process::ExitCode};

mod artifact_command;
use vize_tool_support::common;

fn main() -> ExitCode {
    common::main_result(artifact_command::run_single(
        env::args().nth(1).as_deref(),
        "docs/davinci/plan/corpus-coverage.md",
        "usage: rust-script tools/commands/davinci/corpus-coverage.rs --write | --check",
        "rust-script tools/commands/davinci/corpus-coverage.rs --write",
    ))
}
