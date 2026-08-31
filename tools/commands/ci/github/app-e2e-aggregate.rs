#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{env, process::ExitCode};

fn main() -> ExitCode {
    match aggregate(env::args().skip(1).collect()) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn aggregate(args: Vec<String>) -> Result<String, String> {
    let [
        profile,
        suite,
        run_required,
        plan_result,
        producer_result,
        planned_count,
    ] = args.as_slice()
    else {
        return Err("Usage: rust-script tools/commands/ci/github/app-e2e-aggregate.rs <profile> <suite> <run-required> <plan-result> <producer-result> <planned-count>".to_string());
    };
    let run_required = match run_required.as_str() {
        "true" => true,
        "false" => false,
        value => return Err(format!("run-required must be true or false, got {value}")),
    };
    let planned_count = if planned_count.is_empty() {
        0
    } else {
        planned_count.parse::<usize>().map_err(|_| {
            format!("planned-count must be a non-negative integer, got {planned_count}")
        })?
    };

    if plan_result != "success" {
        return Err(format!("App E2E planner is {plan_result}"));
    }
    if !run_required {
        if profile != "readiness" {
            return Err("Only readiness may be a successful no-op".to_string());
        }
        if producer_result != "skipped" {
            return Err(format!(
                "Irrelevant readiness producers must be skipped, got {producer_result}"
            ));
        }
        return Ok("App E2E readiness aggregate: 0 row(s) succeeded".to_string());
    }

    let expected = expected_count(profile, suite)?;
    if planned_count != expected {
        return Err(format!(
            "App E2E planner emitted {planned_count} rows; expected {expected}"
        ));
    }
    if producer_result != "success" {
        return Err(format!(
            "App E2E producers are {producer_result}; expected success"
        ));
    }
    Ok(format!(
        "App E2E {profile} aggregate: {expected} row(s) succeeded"
    ))
}

fn expected_count(profile: &str, suite: &str) -> Result<usize, String> {
    match (profile, suite) {
        ("readiness", "all" | "readiness") => Ok(6),
        ("readiness", other) => Err(format!("Unknown readiness suite: {other}")),
        ("full", "dev" | "vrt") => Ok(5),
        ("full", "preview") => Ok(4),
        ("full", "check" | "lint" | "build") => Ok(1),
        ("full", "all") => Ok(17),
        ("full", other) => Err(format!("Unknown full App E2E suite: {other}")),
        (other, _) => Err(format!("Unknown App E2E profile: {other}")),
    }
}
