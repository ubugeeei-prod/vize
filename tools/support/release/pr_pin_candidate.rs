//! Read-only, pre-merge qualification of the actual version delivery candidate.
use super::super::{pr_contract, pr_github as github};
use super::{Source, first_parent, marker, metadata, source};
use serde_json::{Value, json};
use std::{env, fs, path::Path};

fn integration_fields(
    pr: &Value,
    repository: &str,
    number: u64,
    head: &str,
) -> Result<(u64, String, String, String), String> {
    pr_contract::sha(head)?;
    if number == 0
        || pr.get("number").and_then(Value::as_u64) != Some(number)
        || pr.get("merged").and_then(Value::as_bool) != Some(false)
        || pr.get("draft").and_then(Value::as_bool) != Some(false)
        || pr_contract::field(pr, "/state")? != "open"
        || pr_contract::field(pr, "/base/ref")? != "main"
        || pr_contract::field(pr, "/head/sha")? != head
        || pr_contract::field(pr, "/base/repo/full_name")? != repository
        || pr_contract::field(pr, "/head/repo/full_name")? != repository
    {
        return Err(
            "Pre-merge qualification requires the exact open official integration PR.".into(),
        );
    }
    let body = pr_contract::field(pr, "/body")?;
    let source_number = marker(body, "vize-release-pin-source")?
        .parse::<u64>()
        .map_err(|_| "Invalid integration source PR number")?;
    if source_number == 0 || source_number == number {
        return Err("Integration requires a separate immutable source PR.".into());
    }
    let head = marker(body, "vize-release-pin-head")?;
    let cut = marker(body, "vize-release-pin-cut")?;
    let tag = marker(body, "vize-release-pin-tag")?;
    pr_contract::sha(&head)?;
    pr_contract::sha(&cut)?;
    pr_contract::tag(&tag)?;
    if pr_contract::field(pr, "/head/ref")? != format!("release-integration/{tag}") {
        return Err("Integration branch and pinned body tag disagree.".into());
    }
    Ok((source_number, head, cut, tag))
}

fn candidate_parents(
    candidate: &str,
    base: &str,
    head: &str,
    queue: bool,
    root: &Path,
) -> Result<(), String> {
    let row = github::git(&["rev-list", "--parents", "-n", "1", candidate], root)?;
    let expected = if queue {
        format!("{candidate} {base}")
    } else {
        format!("{candidate} {base} {head}")
    };
    if row != expected {
        return Err("Candidate parents do not bind the exact PR merge or own queue base.".into());
    }
    Ok(())
}

fn projection(source: &Source, candidate: &str, base: &str, root: &Path) -> Result<(), String> {
    first_parent(&source.cut, base, root)?;
    let version = source.candidate.tag.trim_start_matches('v');
    metadata::verify_delta(base, candidate, &source.base_version, version, root)?;
    // This is the same complete catalog used after actual signed delivery.
    // Manifest scripts and every other shipping field stay authoritative.
    if metadata::catalog(&source.candidate.head, version, root)?
        != metadata::catalog(candidate, version, root)?
    {
        return Err("The pre-merge candidate registry closure differs from the qualified H catalog; prepare a new source cut.".into());
    }
    Ok(())
}

fn current_pr_snapshot(
    candidate: &str,
    current: &str,
    base: &str,
    head: &str,
    pr_event: bool,
    root: &Path,
) -> Result<(), String> {
    pr_contract::sha(current)?;
    if current == candidate {
        return Ok(());
    }
    if !pr_event {
        return Err("The local receipt is not the current GitHub synthetic merge SHA.".into());
    }
    // GitHub can regenerate a PR merge commit after reopening without changing
    // either parent or any checked-out byte. The event SHA remains the tested
    // candidate; accept the refresh only after authenticating both complete
    // parent vectors and trees. Merge-group identity never uses this path.
    github::git(&["fetch", "--no-tags", "origin", current], root)?;
    candidate_parents(current, base, head, false, root)?;
    let tree = |sha: &str| github::git(&["rev-parse", &format!("{sha}^{{tree}}")], root);
    if tree(candidate)? != tree(current)? {
        return Err("The current synthetic PR merge changed its complete tree.".into());
    }
    Ok(())
}

fn event_fields(
    event: Option<(&str, &Value)>,
    pr: &Value,
    repository: &str,
    number: u64,
    candidate: &str,
    base: &str,
    head: &str,
) -> Result<bool, String> {
    match event {
        Some(("merge_group", event)) => {
            if pr_contract::field(event, "/repository/full_name")? != repository
                || pr_contract::field(event, "/merge_group/head_sha")? != candidate
                || pr_contract::field(event, "/merge_group/base_sha")? != base
                || pr_contract::field(event, "/merge_group/base_ref")? != "refs/heads/main"
                || pr_contract::field(event, "/merge_group/head_ref")?
                    != format!("refs/heads/gh-readonly-queue/main/pr-{number}-{base}")
            {
                return Err(
                    "Integration must qualify its own exact main merge-group candidate.".into(),
                );
            }
            Ok(true)
        }
        Some(("pull_request", event)) => {
            integration_fields(
                event
                    .get("pull_request")
                    .ok_or("Missing integration PR event")?,
                repository,
                number,
                head,
            )?;
            if event.get("number").and_then(Value::as_u64) != Some(number)
                || pr_contract::field(event, "/repository/full_name")? != repository
                || event.pointer("/pull_request/body") != pr.get("body")
                || pr_contract::field(event, "/pull_request/base/sha")? != base
            {
                return Err("Integration PR body/head/base changed since this event.".into());
            }
            Ok(false)
        }
        Some(_) => Err("Unsupported integration qualification event.".into()),
        None => Ok(false),
    }
}

fn verify(
    number: u64,
    candidate: &str,
    base: &str,
    head: &str,
    event: Option<(&str, &Value)>,
    root: &Path,
) -> Result<(), String> {
    for sha in [candidate, base, head] {
        pr_contract::sha(sha)?;
    }
    if github::git(&["rev-parse", "HEAD"], root)? != candidate
        || env::var("GITHUB_SHA").is_ok_and(|sha| sha != candidate)
    {
        return Err("Pre-merge qualification must check out its exact candidate SHA.".into());
    }
    let repository = github::repository(root)?;
    let pr = github::api(&repository, &format!("pulls/{number}"), root)?;
    let (source_number, source_head, cut, tag) =
        integration_fields(&pr, &repository, number, head)?;
    let remote = github::git(
        &[
            "ls-remote",
            "--heads",
            "origin",
            &format!("refs/heads/release-integration/{tag}"),
        ],
        root,
    )?;
    if remote.split_whitespace().next() != Some(head) {
        return Err("Integration branch and PR head disagree.".into());
    }
    pr_contract::maintainer(&github::author(
        &repository,
        pr_contract::field(&pr, "/user/login")?,
        root,
    )?)?;
    let queue = event_fields(event, &pr, &repository, number, candidate, base, head)?;
    github::git(
        &[
            "fetch",
            "--no-tags",
            "origin",
            &source_head,
            &cut,
            head,
            base,
        ],
        root,
    )?;
    candidate_parents(candidate, base, head, queue, root)?;
    if !queue {
        current_pr_snapshot(
            candidate,
            pr_contract::field(&pr, "/merge_commit_sha")?,
            base,
            head,
            matches!(event, Some(("pull_request", _))),
            root,
        )?;
    }
    let source = source(&repository, source_number, &source_head, &tag, false, root)?;
    if source.integration != number || source.cut != cut {
        return Err("Source and integration bodies do not bind the same immutable cut.".into());
    }
    let head_parent = github::parent(head, root)?;
    first_parent(&source.cut, &head_parent, root)?;
    metadata::verify_delta(
        &head_parent,
        head,
        &source.base_version,
        tag.trim_start_matches('v'),
        root,
    )?;
    projection(&source, candidate, base, root)?;
    println!(
        "{}",
        json!({"integration":number,"source":source_number,"H":source_head,"C":cut,"tag":tag,"candidate":candidate,"base":base,"integrationHead":head,"event":if queue {"merge_group"} else {"pull_request"},"catalogEqual":true,"metadataExact":true})
    );
    Ok(())
}

/// A local receipt qualifies only the current synthetic PR merge snapshot.
pub fn verify_candidate(
    number: u64,
    candidate: &str,
    base: &str,
    head: &str,
    root: &Path,
) -> Result<(), String> {
    verify(number, candidate, base, head, None, root)
}

pub fn check_candidate(number: u64, root: &Path) -> Result<(), String> {
    let event_name = env::var("GITHUB_EVENT_NAME").map_err(|_| "Missing integration event name")?;
    let event: Value = serde_json::from_slice(
        &fs::read(env::var("GITHUB_EVENT_PATH").map_err(|_| "Missing integration event payload")?)
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let candidate = env::var("GITHUB_SHA").map_err(|_| "Missing integration candidate SHA")?;
    let (base, head) = match event_name.as_str() {
        "pull_request" => (
            pr_contract::field(&event, "/pull_request/base/sha")?.to_string(),
            pr_contract::field(&event, "/pull_request/head/sha")?.to_string(),
        ),
        "merge_group" => {
            let repository = github::repository(root)?;
            let pr = github::api(&repository, &format!("pulls/{number}"), root)?;
            (
                pr_contract::field(&event, "/merge_group/base_sha")?.to_string(),
                pr_contract::field(&pr, "/head/sha")?.to_string(),
            )
        }
        _ => return Err("Unsupported integration qualification event.".into()),
    };
    verify(
        number,
        &candidate,
        &base,
        &head,
        Some((&event_name, &event)),
        root,
    )
}

#[cfg(test)]
#[path = "pr_pin_candidate_tests.rs"]
mod tests;
