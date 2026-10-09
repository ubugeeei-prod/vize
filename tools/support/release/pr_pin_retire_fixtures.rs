//! Inert complete API snapshots backed by actual private Git objects, not public execution.
use super::{Repo, github};
use serde_json::{Value, json};
#[path = "pr_pin_retire_absence_fixtures.rs"]
mod absence;
#[path = "pr_pin_retire_guard_fixtures.rs"]
mod guards;
pub(super) fn guard_controls(
    receipt: &Value,
    logs: &std::collections::BTreeMap<String, Vec<u8>>,
    repo: &Repo,
) {
    guards::guard_controls(receipt, logs, repo);
}

const WORKFLOW: &str = "jobs:\n  release-npm-law:\n    name: Release law to npm\n  create-github-release:\n    name: Create GitHub Release\n";

fn run(id: u64, sha: &str, branch: &str, workflow: &str, title: &str) -> Value {
    json!({"id":id,"head_sha":sha,"head_branch":branch,"path":workflow,"display_title":title,
        "event":"workflow_dispatch","status":"completed","conclusion":"failure","run_attempt":1,
        "repository":{"id":10,"full_name":"owner/repo"},"head_repository":{"id":10,"full_name":"owner/repo"},
        "actor":{"login":"maintainer"},"triggering_actor":{"login":"maintainer"}})
}
fn job(id: u64, run: &Value, name: &str, status: &str, conclusion: Value) -> Value {
    json!({"id":id,"run_id":run["id"],"run_attempt":1,"head_sha":run["head_sha"],
        "name":name,"status":status,"conclusion":conclusion,"steps":[]})
}
fn observation(run: &Value, jobs: Vec<Value>, artifacts: Vec<Value>) -> Value {
    json!({"run":run,"jobPages":[{"total_count":jobs.len(),"jobs":jobs}],
        "artifactPages":[{"total_count":artifacts.len(),"artifacts":artifacts}]})
}

pub(super) fn fixture(repo: &Repo) -> (Value, String, String, String) {
    let cut = repo.commit(&[
        ("Cargo.toml", "[workspace.package]\nversion = \"0.7.0\"\n"),
        (".github/workflows/release.yml", WORKFLOW),
        (
            ".github/workflows/release-operator.yml",
            "name: Release Operator\non: workflow_dispatch\n",
        ),
    ]);
    let head = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"0.8.0\"\n")]);
    let pin = github::git(
        &[
            "commit-tree",
            &github::git(&["rev-parse", &format!("{head}^{{tree}}")], &repo.work).unwrap(),
            "-p",
            &head,
            "-m",
            &format!("Vize immutable release cut\n\nSource-PR: #42\nSource-head: {head}\nSource-cut: {cut}\nTag: v0.8.0\nBase-version: 0.7.0\nIntegration-PR: #99"),
        ],
        &repo.work,
    )
    .unwrap();
    let integration = github::git(
        &[
            "commit-tree",
            &github::git(&["rev-parse", &format!("{head}^{{tree}}")], &repo.work).unwrap(),
            "-p",
            &cut,
            "-m",
            "original M",
        ],
        &repo.work,
    )
    .unwrap();
    for (branch, sha) in [
        ("main", cut.as_str()),
        ("release/v0.8.0", head.as_str()),
        ("release-pin/v0.8.0", pin.as_str()),
        ("release-integration/v0.8.0", integration.as_str()),
    ] {
        github::git(
            &["push", "origin", &format!("{sha}:refs/heads/{branch}")],
            &repo.work,
        )
        .unwrap();
    }
    let identity = json!({"repository":"owner/repo","tag":"v0.8.0","sourcePr":42,"head":head,
        "cut":cut,"baseVersion":"0.7.0","pin":pin,"integrationPr":99,"integrationHead":integration,
        "releaseRun":7,"releaseAttempt":1,"operatorRun":6,"operatorAttempt":1});
    let release = run(
        7,
        &head,
        "release/v0.8.0",
        ".github/workflows/release.yml",
        &format!("Pinned Release v0.8.0 PR #42 @ {head}"),
    );
    let mut operator = run(
        6,
        &cut,
        "main",
        ".github/workflows/release-operator.yml",
        "Release Operator",
    );
    operator["conclusion"] = json!("cancelled");
    let mut pending = run(
        8,
        &head,
        "release/v0.8.0",
        ".github/workflows/check.yml",
        "Check full",
    );
    pending["status"] = json!("in_progress");
    pending["conclusion"] = Value::Null;
    let artifact = json!({"id":1001,"name":"diagnostic metadata fixture","digest":format!("sha256:{}", "a".repeat(64)),
        "size_in_bytes":123,"expires_at":"2026-11-09T00:00:00Z","expired":false,
        "archive_download_url":"https://api.github.com/repos/owner/repo/actions/artifacts/1001/zip",
        "workflow_run":{"id":7,"repository_id":10,"head_repository_id":10,"head_sha":head,"head_branch":"release/v0.8.0"}});
    let source_pr = json!({"number":42,"user":{"login":"maintainer"},"state":"open","draft":true,"merged":false,
        "base":{"ref":"main","repo":{"full_name":"owner/repo"}},"head":{"sha":head,"ref":"release/v0.8.0","repo":{"full_name":"owner/repo"}},
        "body":format!("<!-- vize-release-pin: immutable-v1 -->\n<!-- vize-release-pin-cut: {cut} -->\n<!-- vize-release-pin-head: {head} -->\n<!-- vize-release-integration: 99 -->\n<!-- vize-release-base-version: 0.7.0 -->\n")});
    let integration_pr = json!({"number":99,"user":{"login":"maintainer"},"state":"open","draft":false,"merged":false,
        "base":{"ref":"main","sha":head,"repo":{"full_name":"owner/repo"}},"head":{"sha":integration,"ref":"release-integration/v0.8.0","repo":{"full_name":"owner/repo"}},
        "body":format!("<!-- vize-release-pin-source: 42 -->\n<!-- vize-release-pin-cut: {cut} -->\n<!-- vize-release-pin-head: {head} -->\n<!-- vize-release-pin-tag: v0.8.0 -->\n")});
    let jobs = vec![
        job(
            123,
            &release,
            "Original build",
            "completed",
            json!("failure"),
        ),
        job(
            124,
            &release,
            "Release law to npm",
            "completed",
            json!("cancelled"),
        ),
        job(
            125,
            &release,
            "Create GitHub Release",
            "completed",
            json!("skipped"),
        ),
    ];
    let receipt = json!({"schema":"vize-unpublished-retirement-v1","identity":identity,
    "guards":{"identity":identity,"sourcePr":source_pr,"integrationPr":integration_pr,"releaseRun":release,"operatorRun":operator,
        "publicationJobs":jobs,"mainAtGuard":cut,"tagAbsent":true,"githubReleaseAbsent":true,
        "githubReleaseInventory":{"publishedTagLookup":"authenticated typed HTTP 404","pages":[[]]},
        "operatorInventory":{"pages":[{"total_count":1,"workflow_runs":[operator]}],"currentOperatorContext":null}},
    "originalEvidence":{"HRunPages":[{"total_count":2,"workflow_runs":[release,pending]}],
        "observations":[
            observation(&release,jobs,vec![artifact]),
            observation(&pending,vec![job(126,&pending,"Pending Check","in_progress",Value::Null)],vec![]),
            observation(&operator,vec![job(127,&operator,"Original operator","completed",json!("cancelled"))],vec![])
        ]}});
    let mut receipt = receipt;
    receipt["registryAbsence"] = absence::fixture(repo, &head, &cut);
    (receipt, head, pin, integration)
}

/// One-field inert corruptions retain all unrelated source and private Git evidence.
pub(super) fn mutations(head: &str, bytes: usize) -> Vec<(String, Value)> {
    let root = "/originalEvidence/observations/0";
    vec![
        ("/originalEvidence/observations".into(), json!([])),
        ("/originalEvidence/HRunPages".into(), json!([])),
        (
            "/originalEvidence/HRunPages/0/workflow_runs/0/head_sha".into(),
            json!("a".repeat(40)),
        ),
        (
            "/originalEvidence/HRunPages/0/workflow_runs/0/repository/full_name".into(),
            json!("foreign/repo"),
        ),
        (
            "/originalEvidence/HRunPages/0/total_count".into(),
            json!(10001),
        ),
        (
            "/originalEvidence/HRunPages/0/workflow_runs/1/id".into(),
            json!(7),
        ),
        (format!("{root}/jobPages"), json!([])),
        (format!("{root}/jobPages/0/total_count"), json!(4)),
        (format!("{root}/jobPages/0/jobs/1/id"), json!(123)),
        (format!("{root}/jobPages/0/jobs/0/run_id"), json!(8)),
        (
            format!("{root}/jobPages/0/jobs/0/head_sha"),
            json!("a".repeat(40)),
        ),
        (format!("{root}/jobPages/0/jobs/0/run_attempt"), json!(2)),
        (format!("{root}/jobPages/0/jobs/0/status"), json!("unknown")),
        (
            format!("{root}/jobPages/0/jobs/0/conclusion"),
            json!("unknown"),
        ),
        (
            "/originalEvidence/observations/1/jobPages/0/jobs/0/status".into(),
            json!("completed"),
        ),
        (
            format!("{root}/jobPages/0/jobs/0/conclusion"),
            json!("cancelled"),
        ),
        (format!("{root}/artifactPages"), json!([])),
        (format!("{root}/artifactPages/0/total_count"), json!(2)),
        (
            format!("{root}/artifactPages/0/artifacts/0/size_in_bytes"),
            json!(0),
        ),
        (
            format!("{root}/artifactPages/0/artifacts/0/archive_download_url"),
            json!("https://api.github.com/repos/foreign/repo/actions/artifacts/1001/zip"),
        ),
        (
            format!("{root}/artifactPages/0/artifacts/0/workflow_run/repository_id"),
            json!(11),
        ),
        (
            format!("{root}/artifactPages/0/artifacts/0/workflow_run/head_repository_id"),
            json!(11),
        ),
        (
            "/originalEvidence/observations/1/jobPages/0/jobs/0/id".into(),
            json!(123),
        ),
        (
            format!("{root}/artifactPages/0/artifacts/0/digest"),
            Value::Null,
        ),
        (
            format!("{root}/artifactPages/0/artifacts/0/expires_at"),
            Value::Null,
        ),
        (
            format!("{root}/artifactPages/0/artifacts/0/expired"),
            json!("unknown"),
        ),
        (
            format!("{root}/artifactPages/0/artifacts/0/workflow_run/id"),
            json!(8),
        ),
        (
            format!("{root}/artifactPages/0/artifacts/0/workflow_run/head_sha"),
            json!("a".repeat(40)),
        ),
        ("/originalEvidence/observations/2/run/id".into(), json!(88)),
        ("/failureLogObjects".into(), json!({})),
        (
            "/failureLogObjects/failure~1123.log/oid".into(),
            json!(head),
        ),
        ("/failureLogObjects/failure~1123.log/bytes".into(), json!(0)),
        (
            "/failureLogObjects/failure~1123.log/bytes".into(),
            json!(bytes + 1),
        ),
    ]
}
