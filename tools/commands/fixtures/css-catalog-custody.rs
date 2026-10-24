#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! vize_tool_support = { path = "../../support" }
//! ```
//! Run the observer beside the actual static catalog it compiles.

use std::{
    env,
    process::{Command, ExitCode},
};
use vize_tool_support::common;

fn main() -> ExitCode {
    let result = (|| {
        let root = common::repo_root()?;
        let status = Command::new("rust-script")
            .arg("--force")
            .arg(root.join("crates/vize_maestro/src/ide/style/css_catalog_custody.rs"))
            .args(env::args_os().skip(1))
            .status()
            .map_err(|error| error.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("catalog observer exited with {status}"))
        }
    })();
    common::main_result(result)
}
