use super::{
    pr_checks,
    pr_contract::{self, Candidate},
    pr_github as github, pr_start,
};
use serde_json::Value;
use std::{path::Path, process::Command};

fn pull_request_checks(
    candidate: &Candidate,
    root: &Path,
    required_only: bool,
) -> Result<Value, String> {
    let number = candidate.number.to_string();
    let mut args = vec![
        "pr",
        "checks",
        number.as_str(),
        "--repo",
        &candidate.repository,
    ];
    if required_only {
        args.push("--required");
    }
    args.extend(["--json", "name,bucket,link"]);
    let result = Command::new("gh")
        .args(&args)
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    if result.status.success() {
        return serde_json::from_slice(&result.stdout)
            .map_err(|_| "Could not read PR checks".to_string());
    }
    let stderr = String::from_utf8_lossy(&result.stderr);
    if required_only && stderr.contains("no required checks reported") {
        return pull_request_checks(candidate, root, false);
    }
    Err(format!("Could not read required PR checks: {stderr}"))
}

fn main_required_names(repository: &str, root: &Path) -> Result<Vec<String>, String> {
    let rules = github::api(repository, "rules/branches/main", root)?;
    let rules = rules.as_array().ok_or("Missing main branch rules")?;
    let mut names = Vec::new();
    for rule in rules {
        if rule.get("type").and_then(Value::as_str) != Some("required_status_checks") {
            continue;
        }
        let checks = rule
            .pointer("/parameters/required_status_checks")
            .and_then(Value::as_array)
            .ok_or("Invalid required status check rule")?;
        for check in checks {
            names.push(pr_contract::field(check, "/context")?.to_string());
        }
    }
    if names.is_empty() {
        return Err(
            "No required status checks are configured in main's rulesets; refusing to promote."
                .into(),
        );
    }
    Ok(names)
}

pub fn checks_pass(candidate: &Candidate, root: &Path) -> Result<bool, String> {
    let checks = pull_request_checks(candidate, root, true)?;
    let checks = checks.as_array().ok_or("Missing required PR checks")?;
    let names = main_required_names(&candidate.repository, root)?;
    let checks: Vec<Value> = checks
        .iter()
        .filter(|check| {
            check
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| names.iter().any(|required| required == name))
        })
        .cloned()
        .collect();
    if checks.is_empty() {
        return Ok(false);
    }
    if checks.iter().any(|c| {
        matches!(
            c.get("bucket").and_then(Value::as_str),
            Some("fail" | "cancel")
        )
    }) {
        return Err("A required PR check failed; no tag was created.".into());
    }
    let reported_pass = checks
        .iter()
        .all(|c| c.get("bucket").and_then(Value::as_str) == Some("pass"));
    Ok(reported_pass && pr_checks::ready(candidate, &checks, root)?)
}

pub fn refresh(
    candidate: &Candidate,
    bump: &str,
    base_version: &str,
    root: &Path,
) -> Result<(), String> {
    github::clean(root)?;
    let main_version = github::git(&["show", "origin/main:Cargo.toml"], root)?;
    if github::version_text(&main_version)? != base_version {
        return Err(
            "Another release changed main's version. Preserve this PR and start a new release."
                .into(),
        );
    }
    let message = github::git(&["log", "-1", "--format=%s"], root)?;
    if message != format!("chore: release {}", candidate.tag)
        || github::git(&["rev-parse", "HEAD"], root)? != candidate.head
    {
        return Err(
            "Release commit was edited; preserving it instead of regenerating over those changes."
                .into(),
        );
    }
    // Only the isolated, generated version commit is replaced. The lease
    // below protects any concurrent edit to the PR branch.
    github::git(&["reset", "--hard", "origin/main"], root)?;
    pr_start::prepare(bump, root)?;
    if format!("v{}", github::version(root)?) != candidate.tag {
        return Err("Refreshing changed the proposed version; refusing to replace this PR.".into());
    }
    github::git(
        &[
            "push",
            &format!(
                "--force-with-lease=refs/heads/{}:{}",
                candidate.branch, candidate.head
            ),
            "origin",
            &format!("HEAD:refs/heads/{}", candidate.branch),
        ],
        root,
    )?;
    Ok(())
}

pub fn promote(candidate: &Candidate, run_id: u64, root: &Path) -> Result<bool, String> {
    github::clean(root)?;
    let fresh = github::candidate(
        &candidate.repository,
        candidate.number,
        &candidate.head,
        &candidate.tag,
        false,
        root,
    )?;
    let main = github::fetch_main(root)?;
    if github::parent(&fresh.head, root)? != main {
        return Ok(false);
    }
    if github::tag_target(&fresh.tag, root)?.is_some() {
        return Err("The release tag appeared concurrently; refusing to replace it.".into());
    }
    pr_contract::current_parent(&github::parent(&fresh.head, root)?, &main)?;
    if !checks_pass(&fresh, root)? {
        return Ok(false);
    }
    let tag_ref = format!("refs/tags/{}", fresh.tag);
    github::git(
        &[
            "tag",
            "-a",
            &fresh.tag,
            &fresh.head,
            "-m",
            &format!(
                "Release {}\n\nRelease-PR: #{}\nValidated-run: {run_id}\nValidated-base: {main}",
                fresh.tag, fresh.number
            ),
        ],
        root,
    )?;
    // This is deliberately not a force push. Git's atomic ref transaction
    // rejects an unseen main advance and creates neither ref on failure.
    let pushed = push_atomic(root, &fresh.head, &tag_ref);
    match pushed {
        Ok(_) => {
            println!(
                "Promoted validated PR #{} and {} atomically.",
                fresh.number, fresh.tag
            );
            Ok(true)
        }
        Err(error) => {
            let remote = github::tag_target(&fresh.tag, root)?;
            if remote.as_deref() == Some(&fresh.head) {
                // The network can fail after the server commits the transaction.
                // Verify both refs before treating the promotion as complete.
                let history = github::git(&["fetch", "--no-tags", "origin", "main"], root)
                    .and_then(|_| {
                        github::git(&["rev-list", "--first-parent", "FETCH_HEAD"], root)
                    })?;
                if history.lines().any(|sha| sha == fresh.head) {
                    return Ok(true);
                }
                return Err("Tag exists but the validated commit is absent from main; preserving refs for inspection.".into());
            }
            github::git(&["tag", "-d", &fresh.tag], root)?;
            if github::fetch_main(root)? != main {
                Ok(false)
            } else {
                Err(error)
            }
        }
    }
}

/// The same non-force transaction is exercised against a local bare remote in tests.
pub fn push_atomic(root: &Path, head: &str, tag_ref: &str) -> Result<String, String> {
    github::git(
        &[
            "push",
            "--atomic",
            "origin",
            &format!("{head}:refs/heads/main"),
            tag_ref,
        ],
        root,
    )
}
