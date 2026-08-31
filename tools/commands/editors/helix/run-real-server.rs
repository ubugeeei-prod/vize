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

#[path = "../../../rust/common.rs"]
mod common;
#[path = "../../../rust/lsp_smoke.rs"]
mod lsp_smoke;

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
        "[language-server.vize.config]",
        "editor = true",
        "ecosystem = true",
        "lint = true",
        "typecheck = true",
    ] {
        if !languages.contains(needle) {
            return Err(format!("Helix languages.toml missing {needle}"));
        }
    }
    lsp_smoke::run_editor_contract(&repo, "helix", false)
}
