#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde_json = "1"
//! flate2 = { version = "1", default-features = false, features = ["rust_backend"] }
//!
//! [package]
//! edition = "2024"
//! ```
//! Driver source pin: release-integration-qualification-overlap-v19-prepared-tree.
//! This entry change invalidates rust-script's cache of the changed path modules;
//! the v19 lease, queue and qualification policy is unchanged.

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

/// A bounded stdin adapter for the same readiness authority used by promotion.
/// The catalog authenticates R first; the final report rechecks the whole job.
fn readiness_receipt(input: impl std::io::Read) -> Result<serde_json::Value, String> {
    use std::io::Read;
    const LIMIT: u64 = 2_097_152;
    let mut bytes = Vec::new();
    input
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err("Release readiness inventory exceeds the bounded input".into());
    }
    let jobs: Vec<serde_json::Value> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if !pr_contract::ready_job(&jobs)? {
        return Ok(serde_json::json!({"ready":false}));
    }
    let ready = jobs
        .iter()
        .find(|job| job["name"] == "Release candidate ready")
        .ok_or("Accepted readiness omitted its actual job")?;
    Ok(serde_json::json!({"ready":true,"readyJob":ready}))
}

#[cfg(test)]
mod readiness_tests {
    use super::readiness_receipt;
    use serde_json::json;

    #[test]
    fn original_rust_readiness_adapter_preserves_pending_complete_packets_and_every_failure() {
        let ready = json!({"id":9,"run_id":7,"run_attempt":2,"head_sha":"1111111111111111111111111111111111111111","name":"Release candidate ready","status":"completed","conclusion":"success","steps":[{"name":"original","conclusion":"success"}],"future":{"field":"preserved"}});
        let evaluate = |jobs| readiness_receipt(serde_json::to_vec(&jobs).unwrap().as_slice());
        assert_eq!(
            evaluate(json!([ready.clone()])).unwrap(),
            json!({"ready":true,"readyJob":ready})
        );
        assert_eq!(evaluate(json!([])).unwrap(), json!({"ready":false}));
        for status in ["queued", "in_progress", "waiting"] {
            let mut pending = ready.clone();
            pending["status"] = json!(status);
            pending["conclusion"] = json!(null);
            assert_eq!(evaluate(json!([pending])).unwrap(), json!({"ready":false}));
        }
        for conclusion in [
            "failure",
            "cancelled",
            "skipped",
            "timed_out",
            "neutral",
            "unknown",
        ] {
            let mut bad = ready.clone();
            bad["conclusion"] = json!(conclusion);
            assert!(evaluate(json!([bad])).is_err(), "{conclusion}");
        }
        assert!(evaluate(json!([ready.clone(), ready])).is_err());
        for input in [b"".as_slice(), b"null", b"{}", b"[", b"[] trailing"] {
            assert!(readiness_receipt(input).is_err());
        }
        assert!(readiness_receipt(vec![b' '; 2_097_153].as_slice()).is_err());
    }
}

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
        ["check-release-readiness"] => {
            let receipt = readiness_receipt(std::io::stdin().lock())?;
            println!("{receipt}");
            Ok(())
        }
        ["rewrite-workspace-lock", old, new] => pr_pin::rewrite_workspace_lock(old, new, &root),
        ["verify-producer-metadata", frozen] => pr_pin::verify_producer_metadata(frozen, &root),
        ["start", bump] => pr_start::start(bump, &root),
        ["start", bump, "--pin"] => pr_start::start_pinned(bump, &root),
        ["start-stable", policy, cut] => pr_pin::start_stable(policy, cut, &root),
        ["retire", number, head, tag, run, operator] => pr_pin::retire(
            number.parse().map_err(|_| "Invalid original source PR number")?, head, tag,
            run.parse().map_err(|_| "Invalid original Release run number")?,
            operator.parse().map_err(|_| "Invalid original operator run number")?, &root),
        ["retire-before-release", number, head, tag, qualification, operator] => pr_pin::retire_before_release(
            number.parse().map_err(|_| "Invalid source PR number")?,
            head,
            tag,
            qualification.parse().map_err(|_| "Invalid qualification run")?,
            operator.parse().map_err(|_| "Invalid original operator run")?,
            &root,
        ),
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
            "Usage: pr.rs start <bump> [--pin] | resume <PR> [--pin] | retire <source PR> <H> <tag> <R> <original operator run> | retire-before-release <source PR> <H> <tag> <Q> <original operator run> | validate|wait-promotion|validate-pinned|wait-promotion-pinned <PR> <SHA> <tag> | verify-pinned <PR> <SHA> <tag> <run> | verify-integration-candidate <integration PR> <candidate SHA> <base SHA> <integration head SHA> | check-integration-candidate <integration PR> | check-release-readiness | verify-producer-metadata <frozen-directory>"
                .into(),
        ),
    }
}
