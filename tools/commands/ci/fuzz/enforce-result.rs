#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [event_name, target, outcome] = args.as_slice() else {
        eprintln!(
            "Usage: rust-script tools/commands/ci/fuzz/enforce-result.rs <event-name> <target> <outcome>"
        );
        return ExitCode::from(1);
    };

    if !matches!(
        event_name.as_str(),
        "pull_request" | "schedule" | "workflow_dispatch"
    ) {
        eprintln!("Unsupported fuzz event: {}", empty_label(event_name));
        return ExitCode::from(1);
    }
    if !matches!(
        outcome.as_str(),
        "success" | "failure" | "cancelled" | "skipped"
    ) {
        eprintln!("Unsupported fuzz outcome: {}", empty_label(outcome));
        return ExitCode::from(1);
    }
    if outcome == "success" {
        println!("Fuzz target {target} completed successfully.");
        return ExitCode::SUCCESS;
    }

    let message = format!("Fuzz target {target} finished with {outcome} on {event_name}.");
    if matches!(event_name.as_str(), "schedule" | "workflow_dispatch") {
        eprintln!("::error::{message} Release evidence must be green.");
        ExitCode::from(1)
    } else {
        eprintln!("::warning::{message} Pull-request fuzzing is advisory.");
        ExitCode::SUCCESS
    }
}

fn empty_label(value: &str) -> &str {
    if value.is_empty() { "empty" } else { value }
}
