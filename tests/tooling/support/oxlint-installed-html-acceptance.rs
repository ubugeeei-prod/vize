#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    // Enforce the boundary before any Node preload can execute.
    for key in [
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
        "VIZE_OXLINT_PUBLIC_CUSTODY",
    ] {
        if env::var_os(key).is_some_and(|value| !value.is_empty()) {
            eprintln!("initial source/runtime override must be empty: {key}");
            return ExitCode::FAILURE;
        }
    }
    let mut arguments = env::args_os().skip(1);
    let Some(node) = arguments.next().map(PathBuf::from) else {
        eprintln!("explicit canonical collector Node executable required");
        return ExitCode::FAILURE;
    };
    if !node.is_absolute()
        || !node.is_file()
        || fs::canonicalize(&node).ok().as_ref() != Some(&node)
    {
        eprintln!("Node must be an absolute regular canonical executable path");
        return ExitCode::FAILURE;
    }
    let Some(script) = env::var_os("RUST_SCRIPT_PATH").map(PathBuf::from) else {
        eprintln!("run the owned campaign entrypoint with rust-script");
        return ExitCode::FAILURE;
    };
    let Some(root) = script.ancestors().find(|directory| {
        directory.join("Cargo.toml").is_file() && directory.join("pnpm-workspace.yaml").is_file()
    }) else {
        eprintln!("cannot locate the repository-owned campaign");
        return ExitCode::FAILURE;
    };
    launch(&node, root, arguments.collect())
}

fn launch(node: &Path, root: &Path, arguments: Vec<std::ffi::OsString>) -> ExitCode {
    match Command::new(node)
        .arg(root.join("tests/tooling/support/oxlint-installed-html-acceptance.ts"))
        .args(arguments)
        .status()
    {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(error) => {
            eprintln!("cannot launch the owned installed campaign: {error}");
            ExitCode::FAILURE
        }
    }
}
