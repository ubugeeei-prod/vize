#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```
//! Driver source pin: pinned-raw-parent-custody-v13.

#[path = "../../support/release/pr_budget.rs"]
mod pr_budget;

#[path = "../../support/release/pr_checks.rs"]
mod pr_checks;
#[path = "../../support/release/pr_ci.rs"]
mod pr_ci;
#[path = "../../support/release/pr_contract.rs"]
mod pr_contract;
#[path = "../../support/release/pr_github.rs"]
mod pr_github;
#[path = "../../support/release/pr_pin.rs"]
mod pr_pin;
#[path = "../../support/release/pr_promote.rs"]
mod pr_promote;
#[path = "../../support/release/pr_start.rs"]
mod pr_start;
#[path = "../../support/release/pr_watch.rs"]
mod pr_watch;
#[cfg(test)]
#[path = "../../support/release/pr_tests.rs"]
mod tests;

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["rewrite-guest-lock", path, old, new] => pr_pin::rewrite_guest_lock(path, old, new, &root),
        ["start", bump] => pr_start::start(bump, &root),
        ["start", bump, "--pin"] => pr_start::start_pinned(bump, &root),
        ["retire", number, head, tag, run, operator] => pr_pin::retire(
            number.parse().map_err(|_| "Invalid original source PR number")?, head, tag,
            run.parse().map_err(|_| "Invalid original Release run number")?,
            operator.parse().map_err(|_| "Invalid original operator run number")?, &root),
        ["validate-preparation-target", bump, base, target] => pr_pin::validate_preparation_target(bump, base, target, &root),
        ["resume", number] => {
            pr_start::resume(number.parse().map_err(|_| "Invalid PR number")?, &root)
        }
        ["resume", number, "--pin"] => {
            pr_start::resume_pinned(number.parse().map_err(|_| "Invalid PR number")?, &root)
        }
        [command @ ("validate-pinned" | "wait-promotion-pinned"), number, head, tag] => {
            let number = number.parse().map_err(|_| "Invalid PR number")?;
            if *command == "validate-pinned" {
                pr_ci::validate_pinned(number, head, tag, &root)
            } else {
                pr_ci::wait_for_promotion_pinned(number, head, tag, &root)
            }
        }
        ["verify-pinned", number, head, tag, run_id] => pr_pin::verify_published(
            number.parse().map_err(|_| "Invalid PR number")?,
            head,
            tag,
            run_id.parse().map_err(|_| "Invalid release run number")?,
            &root,
        ),
        ["verify-integration-candidate", number, candidate, base, head] => pr_pin::verify_candidate(
            number.parse().map_err(|_| "Invalid integration PR number")?, candidate, base, head, &root,
        ),
        ["check-integration-candidate", number] => pr_pin::check_candidate(
            number.parse().map_err(|_| "Invalid integration PR number")?, &root,
        ),
        [command @ ("validate" | "wait-promotion"), number, head, tag] => {
            let number = number.parse().map_err(|_| "Invalid PR number")?;
            if *command == "validate" {
                pr_ci::validate(number, head, tag, &root)
            } else {
                pr_ci::wait_for_promotion(number, head, tag, &root)
            }
        }
        _ => Err(
            "Usage: pr.rs start <bump> [--pin] | resume <PR> [--pin] | retire <source PR> <H> <tag> <R> <original operator run> | validate|wait-promotion|validate-pinned|wait-promotion-pinned <PR> <SHA> <tag> | verify-pinned <PR> <SHA> <tag> <run> | verify-integration-candidate <integration PR> <candidate SHA> <base SHA> <integration head SHA> | check-integration-candidate <integration PR>"
                .into(),
        ),
    }
}
