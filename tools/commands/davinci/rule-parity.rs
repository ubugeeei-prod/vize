#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::{env, process::ExitCode};

#[path = "../../support/artifacts.rs"]
mod artifact_command;
#[path = "../../support/common.rs"]
mod common;

fn main() -> ExitCode {
    common::main_result(artifact_command::run_single(
        env::args().nth(1).as_deref(),
        "davinci-road/plan/rule-parity.md",
        "usage: rust-script tools/commands/davinci/rule-parity.rs --write | --check",
        "rust-script tools/commands/davinci/rule-parity.rs --write",
    ))
}
