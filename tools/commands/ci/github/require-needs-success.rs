#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

use serde_json::Value;
use std::{collections::BTreeMap, env, process::ExitCode};

const SKIPPABLE_JOBS: &[(&str, &str)] = &[
    ("nix-flake", "runs on push and schedule only"),
    ("source-coverage", "runs on push and schedule only"),
];

fn main() -> ExitCode {
    let Some(raw) = env::var_os("NEEDS_JSON") else {
        eprintln!(
            "NEEDS_JSON is required: pass ${{{{ toJSON(needs) }}}} to tools/commands/ci/github/require-needs-success.rs"
        );
        return ExitCode::from(1);
    };
    let raw = raw.to_string_lossy();
    match aggregate(&raw) {
        Ok((0, message)) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Ok((_, message)) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn aggregate(raw: &str) -> Result<(u8, String), String> {
    let json: Value = serde_json::from_str(raw).map_err(|error| error.to_string())?;
    let needs = json
        .as_object()
        .ok_or_else(|| "The needs context must be an object of job results".to_string())?;
    if needs.is_empty() {
        return Err(
            "The needs context is empty: test-report must depend on the jobs it gates".to_string(),
        );
    }

    let skippable = SKIPPABLE_JOBS.iter().copied().collect::<BTreeMap<_, _>>();
    let mut succeeded = Vec::new();
    let mut skipped_by_design = Vec::new();
    let mut unresolved = Vec::new();

    for (job, value) in needs.iter().collect::<BTreeMap<_, _>>() {
        let result = value
            .get("result")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("Job {job} reported no result in the needs context"))?;
        if result == "success" {
            succeeded.push(job.as_str());
        } else if result == "skipped" && skippable.contains_key(job.as_str()) {
            skipped_by_design.push(job.as_str());
        } else {
            unresolved.push(format!("  - {job}: {result}"));
        }
    }

    let allowed = skippable.keys().copied().collect::<Vec<_>>().join(", ");
    if !unresolved.is_empty() {
        let mut lines = vec![format!(
            "test-report gate: {} of {} needed jobs did not succeed.",
            unresolved.len(),
            needs.len()
        )];
        lines.extend(unresolved);
        lines.push(
            "test-report is a required status check, so it must not pass while a job it aggregates is red."
                .to_string(),
        );
        lines.push(format!(
            "Only these jobs may skip on a pull request: {allowed}."
        ));
        return Ok((1, lines.join("\n")));
    }

    let skipped = if skipped_by_design.is_empty() {
        "0 skipped on a pull request by design.".to_string()
    } else {
        format!(
            "{} skipped on a pull request by design: {}.",
            skipped_by_design.len(),
            skipped_by_design.join(", ")
        )
    };
    Ok((
        0,
        format!(
            "test-report gate: all {} needed jobs are accounted for. {} succeeded; {skipped}",
            needs.len(),
            succeeded.len()
        ),
    ))
}
