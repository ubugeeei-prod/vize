#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../../support/common.rs"]
mod common;
#[path = "../../../support/release/npm_publish.rs"]
mod npm_publish;

use std::{env, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return Err(
            "Usage: rust-script tools/commands/release/npm/prepare-publish-manifest.rs <package-dir>..."
                .to_string(),
        );
    }
    for package_dir in args {
        npm_publish::prepare_publish_manifest(&PathBuf::from(package_dir))?;
    }
    Ok(())
}
