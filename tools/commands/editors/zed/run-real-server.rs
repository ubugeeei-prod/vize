#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! vize_tool_support = { path = "../../../support" }
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::process::ExitCode;

use vize_tool_support::common;
use vize_tool_support::editors::lsp_smoke;
mod workspace_profile;

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    lsp_smoke::run_editor_contract(&common::repo_root()?, "zed")?;
    workspace_profile::run(&common::repo_root()?)
}
