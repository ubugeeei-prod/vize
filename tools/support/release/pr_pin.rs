//! Opt-in immutable source qualification with a separate queued version PR.
use super::{
    pr_contract::{self, Candidate},
    pr_github as github,
};
use serde_json::Value;
use std::{
    env, fs,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};
#[path = "pr_pin_candidate.rs"]
mod candidate;
#[path = "pr_pin_catalog.rs"]
mod catalog;
#[cfg(test)]
#[path = "pr_pin_catalog_tests.rs"]
mod catalog_tests;
#[cfg(test)]
#[path = "pr_pin_jsr_tests.rs"]
mod jsr_tests;
#[cfg(test)]
#[path = "pr_pin_budget_tests.rs"]
mod budget_tests;
pub use candidate::{check_candidate, verify_candidate};
#[path = "pr_pin_delivery.rs"]
mod delivery;
#[path = "pr_pin_dispatch.rs"]
pub(super) mod dispatch;
#[cfg(test)]
#[path = "pr_pin_gates_tests.rs"]
mod gates_tests;
#[path = "pr_pin_integration.rs"]
mod integration;
#[path = "pr_pin_lock.rs"]
pub(super) mod lock;
#[path = "pr_pin_metadata.rs"]
mod metadata;
pub use metadata::rewrite_guest_lock;
#[path = "pr_pin_retire.rs"]
mod retire;
#[path = "pr_pin_retire_absence.rs"]
mod retire_absence;
#[path = "pr_pin_retire_archive.rs"]
mod retire_archive;
#[path = "pr_pin_retire_evidence.rs"]
mod retire_evidence;
#[path = "pr_pin_retire_guard.rs"]
mod retire_guard;
#[path = "pr_pin_retire_release.rs"]
mod retire_release;
#[path = "pr_pin_retire_ledger.rs"]
mod retire_ledger;
#[path = "pr_pin_retire_source.rs"]
mod retire_source;
#[cfg(test)]
#[path = "pr_pin_retire_lifecycle_tests.rs"]
mod retire_lifecycle_tests;
#[cfg(test)]
#[path = "pr_pin_retire_tests.rs"]
mod retire_tests;
pub use retire::{retire, validate_preparation_target};
#[cfg(test)]
#[path = "pr_pin_recovery_tests.rs"]
mod recovery_tests;
#[path = "pr_pin_runs.rs"]
mod runs;
#[path = "pr_pin_start.rs"]
mod start;
#[path = "pr_pin_tags.rs"]
mod tags;
#[cfg(test)]
#[path = "pr_pin_target_tests.rs"]
mod target_tests;
#[cfg(test)]
#[path = "pr_pin_tests.rs"]
mod tests;
pub use start::{resume, start};
#[path = "pr_pin_watch.rs"]
mod watcher;
use watcher::watch;

#[derive(Clone, Debug)]
pub(super) struct Source {
    candidate: Candidate,
    cut: String,
    base_version: String,
    integration: u64,
    closed: bool,
}

pub(super) fn marker(body: &str, key: &str) -> Result<String, String> {
    let prefix = format!("<!-- {key}: ");
    let found: Vec<_> = body
        .lines()
        .filter_map(|line| {
            line.strip_prefix(&prefix)
                .and_then(|s| s.strip_suffix(" -->"))
        })
        .collect();
    match found.as_slice() {
        [value] if !value.is_empty() => Ok((*value).into()),
        _ => Err(format!(
            "Missing or ambiguous immutable release marker {key}."
        )),
    }
}

pub(super) fn source(
    repository: &str,
    number: u64,
    head: &str,
    tag: &str,
    allow_closed: bool,
    root: &Path,
) -> Result<Source, String> {
    pr_contract::sha(head)?;
    pr_contract::tag(tag)?;
    let pr = github::api(repository, &format!("pulls/{number}"), root)?;
    let body = pr
        .get("body")
        .and_then(Value::as_str)
        .ok_or("Missing pinned source receipt")?;
    if marker(body, "vize-release-pin")? != "immutable-v1"
        || marker(body, "vize-release-pin-head")? != head
    {
        return Err("The immutable source receipt changed.".into());
    }
    let cut = marker(body, "vize-release-pin-cut")?;
    pr_contract::sha(&cut)?;
    let author = pr_contract::field(&pr, "/user/login")?;
    pr_contract::maintainer(&github::author(repository, author, root)?)?;
    let branch = format!("release/{tag}");
    for pointer in ["/base/repo/full_name", "/head/repo/full_name"] {
        if pr_contract::field(&pr, pointer)? != repository {
            return Err("Pinned release receipts reject forks.".into());
        }
    }
    let closed = pr_contract::field(&pr, "/state")? == "closed";
    if pr.get("merged").and_then(Value::as_bool) != Some(false)
        || pr.get("draft").and_then(Value::as_bool) != Some(true)
        || (closed && !allow_closed)
        || (!closed && pr_contract::field(&pr, "/state")? != "open")
        || pr_contract::field(&pr, "/base/ref")? != "main"
        || pr_contract::field(&pr, "/head/ref")? != branch
        || pr_contract::field(&pr, "/head/sha")? != head
    {
        return Err("A pinned source must remain an unchanged, unmerged draft receipt.".into());
    }
    let remote = github::git(
        &[
            "ls-remote",
            "--heads",
            "origin",
            &format!("refs/heads/{branch}"),
        ],
        root,
    )?;
    if remote.split_whitespace().next() != Some(head) {
        return Err("Pinned source branch and PR head disagree.".into());
    }
    if github::parent(head, root)? != cut {
        return Err("Pinned source cut is not its single parent.".into());
    }
    let integration = marker(body, "vize-release-integration")?
        .parse::<u64>()
        .map_err(|_| "Invalid integration PR number")?;
    if integration == 0 || integration == number {
        return Err("Pinned delivery requires a separate integration PR.".into());
    }
    let base_version = marker(body, "vize-release-base-version")?;
    metadata::verify_delta(
        &cut,
        head,
        &base_version,
        tag.strip_prefix('v').unwrap(),
        root,
    )?;
    let source = Source {
        candidate: Candidate {
            number,
            repository: repository.into(),
            author: author.into(),
            branch,
            head: head.into(),
            base: cut.clone(),
            tag: tag.into(),
            merged: false,
        },
        cut,
        base_version,
        integration,
        closed,
    };
    lock::verify_pin(&source, root)?;
    Ok(source)
}

pub(super) fn first_parent(ancestor: &str, head: &str, root: &Path) -> Result<(), String> {
    pr_contract::sha(ancestor)?;
    pr_contract::sha(head)?;
    if github::git(&["rev-list", "--first-parent", head], root)?
        .lines()
        .any(|sha| sha == ancestor)
    {
        Ok(())
    } else {
        Err(format!(
            "{ancestor} is absent from {head}'s first-parent history."
        ))
    }
}

pub(super) fn run_identity(run: &Value, source: &Source, id: u64) -> Result<(), String> {
    let candidate = &source.candidate;
    let title = format!(
        "Pinned Release {} PR #{} @ {}",
        candidate.tag, candidate.number, candidate.head
    );
    if run.get("id").and_then(Value::as_u64) != Some(id)
        || run.get("head_sha").and_then(Value::as_str) != Some(&candidate.head)
        || run.get("head_branch").and_then(Value::as_str) != Some(&candidate.branch)
        || run.get("display_title").and_then(Value::as_str) != Some(&title)
        || run.get("event").and_then(Value::as_str) != Some("workflow_dispatch")
        || run.get("path").and_then(Value::as_str) != Some(".github/workflows/release.yml")
        || run
            .pointer("/head_repository/full_name")
            .and_then(Value::as_str)
            != Some(&candidate.repository)
    {
        return Err("Release evidence is not the exact pinned H/R workflow identity.".into());
    }
    Ok(())
}

pub fn validate(number: u64, head: &str, tag: &str, root: &Path) -> Result<(), String> {
    retire::reject_resume(tag, root)?;
    let repository = github::repository(root)?;
    let source = source(&repository, number, head, tag, false, root)?;
    if github::git(&["rev-parse", "HEAD"], root)? != head
        || env::var("GITHUB_SHA").is_ok_and(|sha| sha != head)
        || format!("v{}", github::version(root)?) != tag
    {
        return Err("The pinned workflow checkout/tag must be the exact immutable H.".into());
    }
    let main = github::fetch_main(root)?;
    first_parent(&source.cut, &main, root)?;
    let main_version = github::version_text(
        &String::from_utf8(metadata::bytes(
            &["show", &format!("{main}:Cargo.toml")],
            root,
        )?)
        .map_err(|e| e.to_string())?,
    )?;
    if main_version != source.base_version {
        if main_version != tag.trim_start_matches('v')
            || delivery::integration(&source, None, root)?.is_none()
        {
            return Err("Another release owns main; the pinned candidate cannot publish.".into());
        }
    }
    if github::tag_target(tag, root)?.is_some() {
        return Err(
            "A candidate cannot requalify after its immutable tag exists; resume its original R."
                .into(),
        );
    }
    if let Ok(path) = env::var("GITHUB_OUTPUT") {
        use std::io::Write;
        writeln!(
            fs::OpenOptions::new()
                .append(true)
                .open(path)
                .map_err(|e| e.to_string())?,
            "tag={tag}\nhead={head}\nbase={}\npr={number}",
            source.cut
        )
        .map_err(|e| e.to_string())?;
    }
    println!(
        "Immutable source {head} is authorized; moving main does not substitute source or artifacts."
    );
    Ok(())
}

/// Read-only custody verification for preflight and external consumers.
pub fn verify_published(
    number: u64,
    head: &str,
    tag: &str,
    run_id: u64,
    root: &Path,
) -> Result<(), String> {
    let repository = github::repository(root)?;
    let source = source(&repository, number, head, tag, true, root)?;
    delivery::published_identity(&source, run_id, root)
}

pub fn wait_for_promotion(number: u64, head: &str, tag: &str, root: &Path) -> Result<(), String> {
    let repository = github::repository(root)?;
    let run_id = env::var("GITHUB_RUN_ID")
        .map_err(|_| "Pinned hosted promotion requires GITHUB_RUN_ID")?
        .parse::<u64>()
        .map_err(|_| "Invalid hosted run ID")?;
    let started = Instant::now();
    loop {
        let source = source(&repository, number, head, tag, false, root)?;
        let main = github::fetch_main(root)?;
        first_parent(&source.cut, &main, root)?;
        if github::tag_target(tag, root)?.is_some() {
            delivery::published_identity(&source, run_id, root)?;
            println!(
                "Queued integration and immutable {tag} authorize the existing {head} artifacts from R={run_id}."
            );
            return Ok(());
        }
        let main_version = github::version_text(
            &String::from_utf8(metadata::bytes(
                &["show", &format!("{main}:Cargo.toml")],
                root,
            )?)
            .map_err(|e| e.to_string())?,
        )?;
        if main_version != source.base_version && main_version != tag.trim_start_matches('v') {
            return Err("A newer version owns main; pinned publication is forbidden.".into());
        }
        if started.elapsed() > Duration::from_secs(20_400) {
            return Err(
                "Timed out waiting for pinned integration and tag; no publication attempted."
                    .into(),
            );
        }
        println!("Pinned H={head}: waiting for queued version integration and tag.");
        sleep(Duration::from_secs(20));
    }
}
