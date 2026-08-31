#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! base64 = "0.22"
//! regex = "1"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! sha2 = "0.10"
//! tempfile = "3"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../../support/common.rs"]
mod common;
#[path = "../../../support/release/npm_bootstrap.rs"]
mod npm_bootstrap;

use std::{env, path::Path, process::ExitCode};

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if let [mode, package_name, version] = args.as_slice() {
        if mode == "__names" {
            let (artifact_name, tarball_name) =
                npm_bootstrap::cli_handoff_names(package_name, version)?;
            println!(
                "{}",
                serde_json::json!({
                    "artifactName": artifact_name,
                    "tarballName": tarball_name,
                })
            );
            return Ok(());
        }
    }
    if !args.is_empty() {
        return Err(
            "Usage: rust-script tools/commands/ci/github/npm-bootstrap-handoff.rs".to_string(),
        );
    }
    let env = npm_bootstrap::env_map();
    if env.get("BOOTSTRAP_ARTIFACT_PATH").map(String::as_str) != Some("bootstrap-package") {
        return Err("BOOTSTRAP_ARTIFACT_PATH must be bootstrap-package".to_string());
    }
    if env.get("BOOTSTRAP_HANDOFF_PATH").map(String::as_str) != Some("npm-cli-first-publish") {
        return Err("BOOTSTRAP_HANDOFF_PATH must be npm-cli-first-publish".to_string());
    }
    let npm_bin = env.get("NPM_BIN").map(String::as_str).unwrap_or("npm");
    let handoff = npm_bootstrap::create_cli_publish_handoff(
        Path::new("bootstrap-package"),
        Path::new("npm-cli-first-publish"),
        env.get("EXPECTED_PACKAGE_NAME")
            .map(String::as_str)
            .unwrap_or(""),
        env.get("EXPECTED_PACKAGE_VERSION")
            .map(String::as_str)
            .unwrap_or(""),
        env.get("RELEASE_ARTIFACT_NAME")
            .map(String::as_str)
            .unwrap_or(""),
        env.get("RELEASE_RUN_ID").map(String::as_str).unwrap_or(""),
        env.get("RELEASE_TAG_NAME")
            .map(String::as_str)
            .unwrap_or(""),
        env.get("RELEASE_TAG_SHA").map(String::as_str).unwrap_or(""),
        npm_bin,
    )?;
    common::append_text(
        env.get("GITHUB_OUTPUT")
            .ok_or_else(|| "GITHUB_OUTPUT is required for npm CLI handoff".to_string())?,
        &format!(
            "artifact_name={}\ntarball_name={}\nsha512={}\n",
            npm_bootstrap::handoff_artifact_name(&handoff),
            npm_bootstrap::handoff_tarball_file(&handoff),
            npm_bootstrap::handoff_sha512(&handoff)
        ),
    )?;
    common::append_text(
        env.get("GITHUB_STEP_SUMMARY")
            .ok_or_else(|| "GITHUB_STEP_SUMMARY is required for npm CLI handoff".to_string())?,
        &npm_bootstrap::format_cli_handoff_summary(&handoff),
    )?;
    println!(
        "Prepared deterministic npm CLI handoff {}/{}.",
        npm_bootstrap::handoff_artifact_name(&handoff),
        npm_bootstrap::handoff_tarball_file(&handoff)
    );
    Ok(())
}
