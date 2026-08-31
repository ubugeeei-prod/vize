#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::process::ExitCode;

#[path = "../../../support/common.rs"]
mod common;
#[path = "../../../support/editors/lsp_smoke.rs"]
mod lsp_smoke;

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    lsp_smoke::run_editor_contract(&common::repo_root()?, "zed", true)
}
