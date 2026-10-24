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

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let repo = common::repo_root()?;
    let languages = common::read_text(repo.join("editors/helix/languages.toml"))?;
    for needle in [
        "[language-server.vize]",
        "command = \"vize\"",
        "args = [\"lsp\"]",
    ] {
        if !languages.contains(needle) {
            return Err(format!("Helix languages.toml missing {needle}"));
        }
    }
    lsp_smoke::run_editor_contract(&repo, "helix")
}
