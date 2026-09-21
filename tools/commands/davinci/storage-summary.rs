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

const USAGE: &str =
    "usage: rust-script tools/commands/davinci/storage-summary.rs --write | --check";
const LEGACY_GENERATOR: &str = "legacy-tools/davinci/storage-summary.mjs";

fn main() -> ExitCode {
    artifact_command::run_node_generator(env::args().nth(1).as_deref(), LEGACY_GENERATOR, USAGE)
}
