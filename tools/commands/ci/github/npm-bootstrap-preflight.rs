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

use std::{env, process::ExitCode};

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let command = match args.as_slice() {
        [] => "preflight",
        [command] if matches!(command.as_str(), "artifact" | "preflight" | "registry-recheck") => {
            command
        }
        [command, rest @ ..] if command == "__contract" => return contract(rest),
        _ => {
            return Err(
                "Usage: rust-script tools/commands/ci/github/npm-bootstrap-preflight.rs [artifact|preflight|registry-recheck]"
                    .to_string(),
            )
        }
    };
    let env = npm_bootstrap::env_map();
    match command {
        "preflight" => npm_bootstrap::run_preflight(&env),
        "artifact" => npm_bootstrap::run_artifact_validation(&env),
        "registry-recheck" => npm_bootstrap::run_registry_recheck(&env),
        _ => unreachable!(),
    }
}

fn contract(args: &[String]) -> Result<(), String> {
    let [mode, payload] = args else {
        return Err("__contract requires <mode> <json>".to_string());
    };
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|error| format!("invalid contract JSON: {error}"))?;
    match mode.as_str() {
        "request" => {
            let request = npm_bootstrap::validate_bootstrap_request(
                value
                    .get("tagName")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("packagePath")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("releaseRunId")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("workflowRef")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("workflowSha")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
            )?;
            println!(
                "{}",
                serde_json::to_string(&request).map_err(|error| error.to_string())?
            );
        }
        "manifest" => {
            let version = npm_bootstrap::validate_bootstrap_manifest(
                value
                    .get("tagName")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("tagSha")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("packagePath")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("packageName")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("cargoToml")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("packageManifest")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
            )?;
            println!("{version}");
        }
        "release-commit" => npm_bootstrap::validate_release_commit(
            value
                .get("tagSha")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            value
                .get("workflowSha")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            value
                .get("mainSha")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            value
                .get("isOnFirstParent")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        )?,
        "downloaded-artifact" => npm_bootstrap::validate_downloaded_artifact(
            value
                .get("packageManifest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            value
                .get("expectedName")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            value
                .get("expectedVersion")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
        )?,
        "release-run" => npm_bootstrap::validate_release_run(
            value.get("run").unwrap_or(&serde_json::Value::Null),
            value
                .get("releaseRunId")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            value
                .get("repository")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            value
                .get("tagName")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            value
                .get("tagSha")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
        )?,
        "release-jobs" => {
            let jobs = value
                .get("jobs")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| "jobs must be an array".to_string())?;
            npm_bootstrap::validate_release_jobs(jobs)?;
        }
        "release-artifact" => {
            let artifacts = value
                .get("artifacts")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| "artifacts must be an array".to_string())?;
            npm_bootstrap::validate_release_artifact(
                artifacts,
                value
                    .get("artifactName")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("releaseRunId")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("tagName")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                value
                    .get("tagSha")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
            )?;
        }
        "registry-response" => npm_bootstrap::validate_registry_response(
            value
                .get("packageName")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            value
                .get("status")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| "status is required".to_string())? as u16,
        )?,
        mode => return Err(format!("unknown contract mode: {mode}")),
    }
    Ok(())
}
