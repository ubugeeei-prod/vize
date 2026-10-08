#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env,
    path::PathBuf,
    process::{Command, ExitCode},
};

const OVERRIDES: [&str; 11] = [
    "NODE_OPTIONS",
    "VIZE_PREFER_WORKSPACE_BINDING",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
    "CORSA_PATH",
    "CORSA_EXECUTABLE",
    "TSGO_PATH",
    "TSGO_EXECUTABLE",
    "VIZE_PUBLIC_NATIVE_CUSTODY",
    "VIZE_OXLINT_NATIVE_CUSTODY",
];

fn main() -> ExitCode {
    // This boundary executes before Python and, crucially, before any Node startup.
    for key in OVERRIDES {
        if env::var_os(key).is_some_and(|value| !value.is_empty()) {
            eprintln!("initial source/runtime override must be empty: {key}");
            return ExitCode::FAILURE;
        }
    }
    let script = match env::var_os("RUST_SCRIPT_PATH") {
        Some(value) => PathBuf::from(value),
        None => {
            eprintln!("run this entrypoint with rust-script");
            return ExitCode::FAILURE;
        }
    };
    let Some(root) = script.ancestors().find(|path| {
        path.join("Cargo.toml").is_file() && path.join("pnpm-workspace.yaml").is_file()
    }) else {
        eprintln!("cannot locate the repository-owned collector");
        return ExitCode::FAILURE;
    };
    match Command::new("python3")
        .arg("-I")
        .arg(root.join("tools/support/release/public_install/collect.py"))
        .args(env::args_os().skip(1))
        .status()
    {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(error) => {
            eprintln!("cannot launch the repository-owned collector: {error}");
            ExitCode::FAILURE
        }
    }
}
