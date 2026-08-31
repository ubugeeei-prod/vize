#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::{env, path::PathBuf, process::ExitCode};

#[path = "../../../support/common.rs"]
mod common;
#[path = "../../../support/editors/e2e.rs"]
mod editor_e2e;

const USAGE: &str = "Usage: rust-script tools/commands/editors/e2e/real-vue-workspace.rs <prepare|server-path|corsa-path|vue-path> [workspace]\n\nprepare <workspace> materializes the shared real-vue editor fixture.";

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let repo_root = common::repo_root()?;
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("prepare") => {
            let workspace = args
                .get(1)
                .map(PathBuf::from)
                .ok_or_else(|| format!("prepare requires a workspace path\n\n{USAGE}"))?;
            let prepared = editor_e2e::prepare_real_vue_workspace(&workspace, false)?;
            println!("{}", prepared.display());
            Ok(())
        }
        Some("server-path") => {
            println!(
                "{}",
                editor_e2e::resolve_real_server_path(&repo_root)?.display()
            );
            Ok(())
        }
        Some("corsa-path") => {
            println!("{}", editor_e2e::resolve_corsa_path(&repo_root)?.display());
            Ok(())
        }
        Some("vue-path") => {
            println!(
                "{}",
                editor_e2e::resolve_vue_package_path(&repo_root)?.display()
            );
            Ok(())
        }
        Some("--help" | "-h") => {
            println!("{USAGE}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command {other}\n\n{USAGE}")),
        None => Err(USAGE.to_string()),
    }
}
