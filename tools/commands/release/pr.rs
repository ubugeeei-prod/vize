#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../support/release/pr_checks.rs"]
mod pr_checks;
#[path = "../../support/release/pr_ci.rs"]
mod pr_ci;
#[path = "../../support/release/pr_contract.rs"]
mod pr_contract;
#[path = "../../support/release/pr_github.rs"]
mod pr_github;
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
        ["start", bump] => pr_start::start(bump, &root),
        ["resume", number] => {
            pr_start::resume(number.parse().map_err(|_| "Invalid PR number")?, &root)
        }
        [command @ ("validate" | "wait-promotion"), number, head, tag] => {
            let number = number.parse().map_err(|_| "Invalid PR number")?;
            if *command == "validate" {
                pr_ci::validate(number, head, tag, &root)
            } else {
                pr_ci::wait_for_promotion(number, head, tag, &root)
            }
        }
        _ => Err(
            "Usage: pr.rs start <bump> | resume <PR> | validate|wait-promotion <PR> <SHA> <tag>"
                .into(),
        ),
    }
}
