use super::super::{pr_budget::Budget, pr_contract, pr_github as github, pr_promote};
use super::{Source, first_parent, marker, metadata, run_identity};
use serde_json::Value;
use std::path::Path;
#[path = "pr_pin_gates.rs"]
pub(super) mod gates;

#[derive(Clone, Debug)]
pub(super) struct Receipt {
    pub head: String,
    pub merge: String,
    pub parent: String,
    pub gate: String,
}

pub(super) fn integration(
    source: &Source,
    expected_gate: Option<&str>,
    root: &Path,
) -> Result<Option<Receipt>, String> {
    let candidate = &source.candidate;
    let pr = github::api(
        &candidate.repository,
        &format!("pulls/{}", source.integration),
        root,
    )?;
    let body = pr
        .get("body")
        .and_then(Value::as_str)
        .ok_or("Missing integration receipt")?;
    for (key, expected) in [
        ("vize-release-pin-source", candidate.number.to_string()),
        ("vize-release-pin-head", candidate.head.clone()),
        ("vize-release-pin-cut", source.cut.clone()),
        ("vize-release-pin-tag", candidate.tag.clone()),
    ] {
        if marker(body, key)? != expected {
            return Err("The integration PR does not authorize this immutable source cut.".into());
        }
    }
    for pointer in ["/base/repo/full_name", "/head/repo/full_name"] {
        if pr_contract::field(&pr, pointer)? != candidate.repository {
            return Err("Integration PRs reject forks.".into());
        }
    }
    if pr_contract::field(&pr, "/base/ref")? != "main"
        || pr_contract::field(&pr, "/head/ref")? != format!("release-integration/{}", candidate.tag)
        || pr.get("draft").and_then(Value::as_bool) != Some(false)
    {
        return Err("Delivery must use its normal main-targeted integration PR.".into());
    }
    pr_contract::maintainer(&github::author(
        &candidate.repository,
        pr_contract::field(&pr, "/user/login")?,
        root,
    )?)?;
    if pr.get("merged").and_then(Value::as_bool) != Some(true) {
        if pr_contract::field(&pr, "/state")? != "open" {
            return Err("The integration PR closed without actual delivery.".into());
        }
        return Ok(None);
    }
    pr_contract::field(&pr, "/merged_at")?;
    let head = pr_contract::field(&pr, "/head/sha")?.to_string();
    let merge = pr_contract::field(&pr, "/merge_commit_sha")?.to_string();
    for sha in [&head, &merge] {
        pr_contract::sha(sha)?;
    }
    let commit = github::api(&candidate.repository, &format!("commits/{merge}"), root)?;
    signed_merge(&commit, &merge)?;
    github::git(&["fetch", "--no-tags", "origin", &merge, &head], root)?;
    let parent = github::parent(&merge, root)?;
    let main = github::fetch_main(root)?;
    first_parent(&source.cut, &parent, root)?;
    first_parent(&merge, &main, root)?;
    let version = candidate.tag.trim_start_matches('v');
    metadata::verify_delta(&parent, &merge, &source.base_version, version, root)?;
    // The exact source registry closure remains authoritative. A newly added
    // package, changed publishing plan, or changed local dependency topology
    // requires a new cut rather than borrowing H's artifacts for moving main.
    let catalog = metadata::catalog(&candidate.head, version, root)?;
    if metadata::catalog(&merge, version, root)? != catalog
        || metadata::catalog(&main, version, root)? != catalog
    {
        return Err(
            "The integration/current main registry closure differs from the qualified H catalog."
                .into(),
        );
    }
    let gate = match expected_gate {
        Some(gate) => {
            if !gates::verify(
                &candidate.repository,
                gate,
                source.integration,
                &merge,
                &parent,
                root,
            )? {
                return Err("Protected integration qualification is incomplete.".into());
            }
            gate.to_string()
        }
        None => match gates::find(
            &candidate.repository,
            source.integration,
            &merge,
            &parent,
            root,
        )? {
            Some(gate) => gate,
            None => return Ok(None),
        },
    };
    Ok(Some(Receipt {
        head,
        merge,
        parent,
        gate,
    }))
}

pub(super) fn signed_merge(commit: &Value, expected: &str) -> Result<(), String> {
    if commit.get("sha").and_then(Value::as_str) != Some(expected)
        || commit
            .pointer("/commit/verification/verified")
            .and_then(Value::as_bool)
            != Some(true)
        || commit
            .pointer("/commit/verification/reason")
            .and_then(Value::as_str)
            != Some("valid")
        || commit
            .get("parents")
            .and_then(Value::as_array)
            .map(Vec::len)
            != Some(1)
    {
        return Err(
            "Version delivery requires its actual signed single-parent merge commit.".into(),
        );
    }
    Ok(())
}

use super::tags;
#[cfg(test)]
pub(super) use tags::annotation;
pub(super) use tags::{push_tag, tag_receipt};

pub(super) fn published_identity(source: &Source, id: u64, root: &Path) -> Result<(), String> {
    let (recorded_id, recorded) = tag_receipt(source, root)?;
    if recorded_id != id {
        return Err(
            "The existing tag belongs to another R; no duplicate publication is allowed.".into(),
        );
    }
    let run = github::api(
        &source.candidate.repository,
        &format!("actions/runs/{id}"),
        root,
    )?;
    run_identity(&run, source, id)?;
    if !pr_contract::ready_job(&github::jobs(&source.candidate.repository, id, root)?)? {
        return Err("The tagged R has no successful exact readiness evidence.".into());
    }
    if !pr_promote::checks_pass(&source.candidate, root)? {
        return Err("The tagged H lacks its own configured exact-head required checks.".into());
    }
    let actual = integration(source, Some(&recorded.gate), root)?
        .ok_or("The tagged integration lacks terminal protected delivery proof")?;
    if actual.head != recorded.head
        || actual.merge != recorded.merge
        || actual.parent != recorded.parent
    {
        return Err("The annotated integration receipt changed.".into());
    }
    if source.closed
        && (run.get("status").and_then(Value::as_str) != Some("completed")
            || run.get("conclusion").and_then(Value::as_str) != Some("success"))
    {
        return Err("A closed source receipt requires successful publication.".into());
    }
    Ok(())
}

pub(super) fn promote(
    source: &Source,
    id: u64,
    receipt: &Receipt,
    operator: &super::lock::Operator,
    budget: &Budget,
    root: &Path,
) -> Result<(), String> {
    github::clean(root)?;
    let fresh = super::source(
        &source.candidate.repository,
        source.candidate.number,
        &source.candidate.head,
        &source.candidate.tag,
        false,
        root,
    )?;
    if !pr_promote::checks_pass(&fresh.candidate, root)?
        || !pr_contract::ready_job(&github::jobs(&fresh.candidate.repository, id, root)?)?
    {
        return Err("Pinned H/R lost its exact required checks or readiness.".into());
    }
    let run = github::api(
        &fresh.candidate.repository,
        &format!("actions/runs/{id}"),
        root,
    )?;
    run_identity(&run, &fresh, id)?;
    let actual =
        integration(&fresh, Some(&receipt.gate), root)?.ok_or("No actual protected integration")?;
    if actual.merge != receipt.merge
        || actual.head != receipt.head
        || actual.parent != receipt.parent
    {
        return Err("Integration changed before immutable tag promotion.".into());
    }
    if github::tag_target(&fresh.candidate.tag, root)?.is_some() {
        return published_identity(&fresh, id, root);
    }
    let tag_ref = format!("refs/tags/{}", fresh.candidate.tag);
    operator.verify()?;
    budget.check()?;
    tags::ensure_local_tag(&fresh, id, &actual, root)?;
    // Only the new tag is written. No main ref appears in this transaction.
    // A race/network ambiguity is resolved by authenticating the complete
    // receipt, never by replacing/deleting a remote immutable tag.
    budget.check()?;
    if let Err(error) = push_tag(root, &tag_ref) {
        if github::tag_target(&fresh.candidate.tag, root)?.is_none() {
            return Err(error);
        }
    }
    published_identity(&fresh, id, root)
}
