//! Exact actual-merge custody, without an invalid PR-head ancestry assumption.
use super::super::super::{pr_checks, pr_contract, pr_github as github};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};
const WORKFLOWS: [&str; 5] = [
    "check.yml",
    "n8n-adoption.yml",
    "musea-browser.yml",
    "nuxt-style-build.yml",
    "nuxt3-module-build.yml",
];

pub(super) fn find(
    repository: &str,
    integration: u64,
    merge: &str,
    parent: &str,
    root: &Path,
) -> Result<Option<String>, String> {
    // G must be actual signed V, not an unrelated equal tree or stale PR head.
    Ok(verify(repository, merge, integration, merge, parent, root)?.then(|| merge.to_string()))
}

pub(super) fn verify(
    repository: &str,
    gate: &str,
    integration: u64,
    merge: &str,
    parent: &str,
    root: &Path,
) -> Result<bool, String> {
    pr_contract::sha(gate)?;
    if gate != merge {
        return Err(
            "The qualified protected candidate G must be the actual signed delivery V.".into(),
        );
    }
    let response = github::api(
        repository,
        &format!("actions/runs?head_sha={gate}&per_page=100"),
        root,
    )?;
    let group_runs = response
        .get("workflow_runs")
        .and_then(Value::as_array)
        .ok_or("Missing protected workflow runs")?;
    if !full_workflows(group_runs, repository, gate, integration, parent)? {
        return Ok(false);
    }
    let mut checks = Vec::new();
    for page in 1..=100 {
        let response = github::api(
            repository,
            &format!("commits/{gate}/check-runs?filter=latest&per_page=100&page={page}"),
            root,
        )?;
        let rows = response
            .get("check_runs")
            .and_then(Value::as_array)
            .ok_or("Missing protected commit checks")?;
        checks.extend(rows.iter().cloned());
        if rows.len() < 100 {
            break;
        }
        if page == 100 {
            return Err("Protected check pagination exceeded its bound.".into());
        }
    }
    let selected:Vec<Value>=checks.iter().map(|check|json!({"name":check["name"],"link":check["details_url"],"bucket":if check.get("status").and_then(Value::as_str)==Some("completed") && check.get("conclusion").and_then(Value::as_str)==Some("success") {"pass"} else {"fail"}})).collect();
    let rules = github::api(repository, "rules/branches/main", root)?;
    pr_checks::required_checks(&rules, &checks, &selected, gate)
}

pub(crate) fn full_workflows(
    runs: &[Value],
    repository: &str,
    gate: &str,
    integration: u64,
    parent: &str,
) -> Result<bool, String> {
    let branch = format!("gh-readonly-queue/main/pr-{integration}-{parent}");
    let mut latest: BTreeMap<&str, &Value> = BTreeMap::new();
    for run in runs {
        if run.get("event").and_then(Value::as_str) != Some("merge_group")
            || run.get("head_sha").and_then(Value::as_str) != Some(gate)
            || run
                .pointer("/head_repository/full_name")
                .and_then(Value::as_str)
                != Some(repository)
            || run.get("head_branch").and_then(Value::as_str) != Some(&branch)
        {
            continue;
        }
        let Some(path) = run.get("path").and_then(Value::as_str) else {
            continue;
        };
        if !WORKFLOWS
            .iter()
            .any(|file| path == format!(".github/workflows/{file}"))
        {
            continue;
        }
        if run
            .get("id")
            .and_then(Value::as_u64)
            .is_none_or(|id| id == 0)
        {
            return Err("Protected workflow lacks an authenticated run ID.".into());
        }
        if latest.get(path).is_none_or(|old| {
            run.get("id").and_then(Value::as_u64) > old.get("id").and_then(Value::as_u64)
        }) {
            latest.insert(path, run);
        }
    }
    let mut complete = true;
    for file in WORKFLOWS {
        let path = format!(".github/workflows/{file}");
        let Some(run) = latest.get(path.as_str()) else {
            complete = false;
            continue;
        };
        if run.get("status").and_then(Value::as_str) != Some("completed") {
            complete = false;
            continue;
        }
        if run.get("conclusion").and_then(Value::as_str) != Some("success") {
            return Err(format!(
                "Protected {file} did not succeed at exact G=V={gate}."
            ));
        }
    }
    Ok(complete)
}
