#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! vize_tool_support = { path = "../../../support" }
//! ```

use std::{
    env,
    process::{Command, ExitCode, Stdio},
};
use vize_tool_support::common;

fn main() -> ExitCode {
    common::main_result(run())
}
fn run() -> Result<(), String> {
    let root = common::repo_root()?;
    let status = Command::new("node")
        .arg(root.join("tools/support/editors/typescript-provider/package.ts"))
        .args(env::args_os().skip(1))
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("cannot start TypeScript provider packager: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("TypeScript provider packaging failed: {status}"))
    }
}
