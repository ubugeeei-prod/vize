//! Inert all-attempt API rows; archive checks use genuine private Git objects and refs.
use super::super::super::retire_source;
use super::super::archive;
use super::{Repo, github};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn release_jobs(receipt: &mut Value, jobs: Vec<Value>) {
    let pages = json!([{"total_count":jobs.len(),"jobs":jobs}]);
    receipt["guards"]["publicationJobs"] = json!(jobs);
    receipt["guards"]["originalFailureJobPages"]["release"] = pages.clone();
    receipt["originalEvidence"]["observations"][0]["jobPages"] = pages;
}

fn attempt_two(receipt: &mut Value) {
    receipt["identity"]["releaseAttempt"] = json!(2);
    receipt["guards"]["identity"] = receipt["identity"].clone();
    receipt["guards"]["releaseRun"]["run_attempt"] = json!(2);
    receipt["guards"]["releaseRun"]["conclusion"] = json!("cancelled");
    let run = receipt["guards"]["releaseRun"].clone();
    receipt["originalEvidence"]["HRunPages"][0]["workflow_runs"][0] = run.clone();
    receipt["originalEvidence"]["observations"][0]["run"] = run.clone();
    let mut jobs = receipt["guards"]["publicationJobs"]
        .as_array()
        .unwrap()
        .clone();
    for (id, name) in [
        (128, "Release law to npm"),
        (129, "Create GitHub Release"),
        (130, "Second build"),
    ] {
        jobs.push(
            json!({"id":id,"run_id":run["id"],"run_attempt":2,"head_sha":run["head_sha"],
            "name":name,"status":"completed","conclusion":"cancelled","steps":[]}),
        );
    }
    release_jobs(receipt, jobs);
}

fn refuses(receipt: Value, logs: &BTreeMap<String, Vec<u8>>, repo: &Repo, before: &str) {
    assert!(retire_source::validate(&receipt, &repo.work).is_err());
    assert!(archive::install(receipt, logs, &repo.work).is_err());
    assert_eq!(
        github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
        before
    );
}

pub(super) fn controls(receipt: &Value, logs: &BTreeMap<String, Vec<u8>>, repo: &Repo) {
    let before = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    for (pointer, value) in [
        ("/guards/originalFailureJobPages", Value::Null),
        ("/guards/originalFailureJobPages/release", json!([])),
        ("/guards/originalFailureJobPages/operator", json!([])),
        (
            "/guards/originalFailureJobPages/release/0/total_count",
            json!(4),
        ),
        (
            "/guards/originalFailureJobPages/operator/0/total_count",
            json!(2),
        ),
        (
            "/guards/originalFailureJobPages/release/0/jobs/0/run_id",
            json!(6),
        ),
        (
            "/guards/originalFailureJobPages/release/0/jobs/0/head_sha",
            json!("a".repeat(40)),
        ),
        (
            "/guards/originalFailureJobPages/release/0/jobs/0/run_attempt",
            json!(0),
        ),
        (
            "/guards/originalFailureJobPages/release/0/jobs/0/run_attempt",
            json!(2),
        ),
        (
            "/guards/originalFailureJobPages/operator/0/jobs/0/id",
            json!(123),
        ),
        ("/guards/publicationJobs/0/conclusion", json!("cancelled")),
    ] {
        let mut bad = receipt.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        refuses(bad, logs, repo, &before);
    }
    for (field, value) in [
        ("run_id", json!(6)),
        ("head_sha", json!("a".repeat(40))),
        ("run_attempt", json!(0)),
        ("run_attempt", json!(2)),
    ] {
        let mut bad = receipt.clone();
        let mut jobs = bad["guards"]["publicationJobs"].as_array().unwrap().clone();
        jobs[0][field] = value;
        release_jobs(&mut bad, jobs);
        refuses(bad, logs, repo, &before);
    }
    let mut collision = receipt.clone();
    collision["guards"]["originalFailureJobPages"]["operator"][0]["jobs"][0]["id"] = json!(123);
    collision["originalEvidence"]["observations"][2]["jobPages"][0]["jobs"][0]["id"] = json!(123);
    refuses(collision, logs, repo, &before);
    let mut cancelled = receipt.clone();
    cancelled["guards"]["releaseRun"]["conclusion"] = json!("cancelled");
    let run = cancelled["guards"]["releaseRun"].clone();
    cancelled["originalEvidence"]["HRunPages"][0]["workflow_runs"][0] = run.clone();
    cancelled["originalEvidence"]["observations"][0]["run"] = run;
    let mut jobs = cancelled["guards"]["publicationJobs"]
        .as_array()
        .unwrap()
        .clone();
    jobs[0]["conclusion"] = json!("cancelled");
    release_jobs(&mut cancelled, jobs);
    refuses(cancelled, logs, repo, &before);

    let mut historical = receipt.clone();
    attempt_two(&mut historical);
    retire_source::validate(&historical, &repo.work).unwrap();
    let mut jobs = historical["guards"]["publicationJobs"]
        .as_array()
        .unwrap()
        .clone();
    jobs[1]["steps"] = json!([{"name":"Publish","status":"completed","conclusion":"cancelled"}]);
    release_jobs(&mut historical, jobs);
    refuses(historical, logs, repo, &before);

    let mut duplicate = receipt.clone();
    attempt_two(&mut duplicate);
    let mut jobs = duplicate["guards"]["publicationJobs"]
        .as_array()
        .unwrap()
        .clone();
    let mut phase = jobs[1].clone();
    phase["id"] = json!(131);
    jobs.push(phase);
    release_jobs(&mut duplicate, jobs);
    refuses(duplicate, logs, repo, &before);

    let attempt_repo = Repo::new();
    let (mut valid, _, _, _) = super::fixture(&attempt_repo);
    let originals = github::git(&["ls-remote", "--refs", "origin"], &attempt_repo.work).unwrap();
    attempt_two(&mut valid);
    let archived = archive::install(valid, logs, &attempt_repo.work).unwrap();
    let (_, saved) = archive::read("v0.8.0", &attempt_repo.work)
        .unwrap()
        .unwrap();
    assert_eq!(saved["identity"]["releaseAttempt"], 2);
    let phases = saved["guards"]["publicationJobs"].as_array().unwrap();
    for name in ["Release law to npm", "Create GitHub Release"] {
        let attempts: Vec<_> = phases
            .iter()
            .filter(|job| job["name"] == name)
            .map(|job| job["run_attempt"].as_u64().unwrap())
            .collect();
        assert_eq!(attempts, [1, 2]);
    }
    assert_eq!(
        archive::remote(&archive::reference("v0.8.0"), &attempt_repo.work).unwrap(),
        Some(archived)
    );
    let after = github::git(&["ls-remote", "--refs", "origin"], &attempt_repo.work).unwrap();
    for original in originals.lines() {
        assert!(after.lines().any(|row| row == original));
    }
}
