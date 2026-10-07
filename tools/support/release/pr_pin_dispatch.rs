use super::super::pr_github as github;
use super::{Source, lock, run_identity, source};
use serde_json::Value;
use std::path::Path;

pub(crate) fn obsolete_legacy(run: &Value, source: &Source) -> bool {
    let candidate = &source.candidate;
    run.get("id")
        .and_then(Value::as_u64)
        .is_some_and(|id| id > 0)
        && run.get("event").and_then(Value::as_str) == Some("workflow_dispatch")
        && run.get("head_sha").and_then(Value::as_str) == Some(&candidate.head)
        && run.get("head_branch").and_then(Value::as_str) == Some(&candidate.branch)
        && run
            .pointer("/head_repository/full_name")
            .and_then(Value::as_str)
            == Some(&candidate.repository)
        && run.get("path").and_then(Value::as_str) == Some(".github/workflows/release.yml")
        && run.get("display_title").and_then(Value::as_str)
            == Some(&format!(
                "Release {} PR #{} @ {}",
                candidate.tag, candidate.number, candidate.head
            ))
        && matches!(
            run.get("status").and_then(Value::as_str),
            Some("queued" | "in_progress" | "waiting" | "requested" | "pending")
        )
}

/// Retire only the exact obsolete mode execution after durable pin revocation.
/// No pin execution, full-H gate workflow, or other source/head is canceled.
pub(super) fn retire_legacy(
    pinned: &Source,
    operator: &lock::Operator,
    root: &Path,
) -> Result<(), String> {
    let candidate = &pinned.candidate;
    let response = github::api(
        &candidate.repository,
        &format!(
            "actions/workflows/release.yml/runs?head_sha={}&per_page=100",
            candidate.head
        ),
        root,
    )?;
    let runs = response
        .get("workflow_runs")
        .and_then(Value::as_array)
        .ok_or("Missing mode-switch workflow inventory")?;
    for run in runs.iter().filter(|run| obsolete_legacy(run, pinned)) {
        operator.verify()?;
        source(
            &candidate.repository,
            candidate.number,
            &candidate.head,
            &candidate.tag,
            false,
            root,
        )?;
        if github::tag_target(&candidate.tag, root)?.is_some() {
            return Err(
                "A tag appeared during mode switch; preserve every execution for inspection."
                    .into(),
            );
        }
        let id = run.get("id").and_then(Value::as_u64).unwrap();
        let fresh = github::api(&candidate.repository, &format!("actions/runs/{id}"), root)?;
        if !obsolete_legacy(&fresh, pinned) {
            continue;
        }
        if run_identity(&fresh, pinned, id).is_ok() {
            return Err("Refusing to retire a pinned execution.".into());
        }
        github::output(
            "gh",
            &[
                "run",
                "cancel",
                &id.to_string(),
                "--repo",
                &candidate.repository,
            ],
            root,
        )?;
        println!(
            "Official mode switch retired obsolete legacy R={id} at unchanged H={}.",
            candidate.head
        );
    }
    Ok(())
}

/// Open source receipts hold the next version until external verification.
pub(super) fn version_owner(repository: &str, tag: &str, root: &Path) -> Result<(), String> {
    let raw = github::output(
        "gh",
        &[
            "pr",
            "list",
            "--repo",
            repository,
            "--state",
            "open",
            "--limit",
            "1000",
            "--json",
            "number,headRefName",
        ],
        root,
    )?;
    let prs: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    for pr in prs
        .as_array()
        .ok_or("Missing open release receipt inventory")?
    {
        if let Some(branch) = pr.get("headRefName").and_then(Value::as_str) {
            if branch.starts_with("release/v") && branch != format!("release/{tag}") {
                return Err(format!(
                    "Source receipt #{} still owns {branch}; finish its external publication before another version.",
                    pr["number"]
                ));
            }
        }
    }
    if prs.as_array().unwrap().len() == 1000 {
        return Err(
            "Release receipt inventory reached its bound; refusing another version.".into(),
        );
    }
    Ok(())
}
