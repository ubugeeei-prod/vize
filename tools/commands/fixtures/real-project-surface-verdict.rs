#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../rust/common.rs"]
mod common;

use serde::Serialize;
use std::{collections::BTreeSet, env, path::PathBuf, process::ExitCode};

const SURFACE_NAMES: &[&str] = &[
    "waiver-audit",
    "typecheck-dependencies",
    "core-tools",
    "lsp",
    "lint-divergence",
    "syntax-highlighter",
    "glyph",
    "typecheck-divergence",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
struct SurfaceResult {
    name: String,
    outcome: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Verdict {
    schema: &'static str,
    version: u8,
    source_commit: Option<String>,
    shard_index: Option<String>,
    status: &'static str,
    surfaces: Vec<SurfaceResult>,
    failed_surface_names: Vec<String>,
}

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args().skip(1))?;
    let surfaces = if args.from_workflow_env {
        results_from_workflow_env()
    } else {
        args.results
    };
    let verdict = create_verdict(surfaces)?;
    common::write_json_pretty(&args.output, &verdict)?;
    if verdict.status != "success" {
        return Err(format!(
            "real-project surfaces failed: {}",
            verdict.failed_surface_names.join(", ")
        ));
    }
    println!(
        "all {} real-project surfaces succeeded",
        verdict.surfaces.len()
    );
    Ok(())
}

struct Args {
    output: PathBuf,
    from_workflow_env: bool,
    results: Vec<SurfaceResult>,
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut output = None;
    let mut from_workflow_env = false;
    let mut results = Vec::new();
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--from-workflow-env" => from_workflow_env = true,
            "--output" => {
                let value = args
                    .next()
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(|| "--output requires a value".to_string())?;
                output = Some(PathBuf::from(value));
            }
            "--surface" => {
                let value = args
                    .next()
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(|| "--surface requires a value".to_string())?;
                let Some((name, outcome)) = value.split_once('=') else {
                    return Err(format!("invalid --surface value: {value}"));
                };
                if name.is_empty() {
                    return Err(format!("invalid --surface value: {value}"));
                }
                results.push(SurfaceResult {
                    name: name.to_string(),
                    outcome: outcome.to_string(),
                });
            }
            _ => return Err(format!("unknown or incomplete argument: {argument}")),
        }
    }
    if output.is_none() {
        return Err("--output is required".to_string());
    }
    if from_workflow_env && !results.is_empty() {
        return Err("--from-workflow-env cannot be combined with --surface".to_string());
    }
    Ok(Args {
        output: output.unwrap(),
        from_workflow_env,
        results,
    })
}

fn results_from_workflow_env() -> Vec<SurfaceResult> {
    vec![
        result("waiver-audit", env_value("VIZE_WAIVER_AUDIT_OUTCOME")),
        result(
            "typecheck-dependencies",
            record_only_verdict(
                env_value("VIZE_TYPECHECK_DEPENDENCIES_OUTCOME"),
                env_value("TYPECHECK_DEPENDENCIES_MODE"),
            ),
        ),
        result(
            "core-tools",
            record_only_verdict(
                env_value("VIZE_CORE_TOOLS_OUTCOME"),
                env_value("CORE_TOOLS_MODE"),
            ),
        ),
        result(
            "lsp",
            record_only_verdict(env_value("VIZE_LSP_OUTCOME"), env_value("LSP_MODE")),
        ),
        result(
            "lint-divergence",
            record_only_verdict(
                env_value("VIZE_LINT_DIVERGENCE_OUTCOME"),
                env_value("LINT_DIVERGENCE_MODE"),
            ),
        ),
        result(
            "syntax-highlighter",
            env_value("VIZE_SYNTAX_HIGHLIGHTER_OUTCOME"),
        ),
        result("glyph", env_value("VIZE_GLYPH_OUTCOME")),
        result(
            "typecheck-divergence",
            record_only_verdict(
                env_value("VIZE_TYPECHECK_DIVERGENCE_OUTCOME"),
                env_value("TYPECHECK_DIVERGENCE_MODE"),
            ),
        ),
    ]
}

fn result(name: &str, outcome: String) -> SurfaceResult {
    SurfaceResult {
        name: name.to_string(),
        outcome,
    }
}

fn env_value(name: &str) -> String {
    env::var(name).unwrap_or_default()
}

fn record_only_verdict(outcome: String, mode: String) -> String {
    if mode == "record-only" && outcome == "failure" {
        "success".to_string()
    } else {
        outcome
    }
}

fn create_verdict(results: Vec<SurfaceResult>) -> Result<Verdict, String> {
    let expected = SURFACE_NAMES.iter().copied().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for result in &results {
        if !expected.contains(result.name.as_str()) {
            return Err(format!("unknown real-project surface: {}", result.name));
        }
        if !seen.insert(result.name.as_str()) {
            return Err(format!("duplicate real-project surface: {}", result.name));
        }
        if !matches!(
            result.outcome.as_str(),
            "success" | "failure" | "cancelled" | "skipped"
        ) {
            return Err(format!(
                "invalid outcome for {}: {}",
                result.name, result.outcome
            ));
        }
    }
    let missing = SURFACE_NAMES
        .iter()
        .copied()
        .filter(|name| !seen.contains(name))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "missing real-project surface verdict(s): {}",
            missing.join(", ")
        ));
    }
    let failed = results
        .iter()
        .filter(|result| result.outcome != "success")
        .map(|result| result.name.clone())
        .collect::<Vec<_>>();
    let mut failed_sorted = failed;
    failed_sorted.sort();
    Ok(Verdict {
        schema: "vize.realProjectSurfaceVerdict",
        version: 1,
        source_commit: env::var("GITHUB_SHA").ok(),
        shard_index: env::var("FIXTURE_SHARD_INDEX").ok(),
        status: if failed_sorted.is_empty() {
            "success"
        } else {
            "failure"
        },
        surfaces: SURFACE_NAMES
            .iter()
            .filter_map(|name| results.iter().find(|result| result.name == *name).cloned())
            .collect(),
        failed_surface_names: failed_sorted,
    })
}
