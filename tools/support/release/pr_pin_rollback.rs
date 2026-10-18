//! Fresh, bounded authentication for withdrawal of an already-owned queue entry.
use super::super::pr_contract;
use super::watcher::retain_queue;
use super::{Source, body, candidate, lock, marker, queue};
use serde_json::Value;
use std::path::Path;

pub(super) struct Rollback<'a> {
    pub pinned: &'a Source,
    pub candidate: &'a pr_contract::Candidate,
    pub owner: &'a queue::QueueOwner,
    pub operator: &'a lock::Operator,
    pub actor: &'a str,
    pub root: &'a Path,
}

fn authenticate_compensation(
    context: &Rollback<'_>,
    adapter: &mut queue::Adapter<'_>,
) -> Result<queue::Authority, String> {
    let pinned = context.pinned;
    let expected = context.candidate;
    if expected.repository != pinned.candidate.repository
        || expected.number != pinned.integration
        || expected.tag != pinned.candidate.tag
        || context.owner.head != expected.head
    {
        return Err("Original rollback source/integration identity changed".into());
    }
    let repository = &expected.repository;
    let prs = adapter.read_group(
        repository,
        &[
            ("rollback-pr", format!("pulls/{}", pinned.candidate.number)),
            ("rollback-pr", format!("pulls/{}", expected.number)),
        ],
    )?;
    let mut reads = Vec::new();
    let mut references = Vec::new();
    for (pr, number, branch, head, draft) in [
        (
            &prs[0],
            pinned.candidate.number,
            pinned.candidate.branch.as_str(),
            pinned.candidate.head.as_str(),
            true,
        ),
        (
            &prs[1],
            expected.number,
            expected.branch.as_str(),
            expected.head.as_str(),
            false,
        ),
    ] {
        let consumed = !draft && pr["state"] == "closed" && pr["merged"] == true;
        if pr["number"].as_u64() != Some(number)
            || pr["draft"] != draft
            || (!consumed && (pr["state"] != "open" || pr["merged"] != false))
            || pr["head"]["sha"] != head
            || pr["head"]["ref"] != branch
            || pr["base"]["ref"] != "main"
            || pr["head"]["repo"]["full_name"] != *repository
            || pr["base"]["repo"]["full_name"] != *repository
            || pr["user"]["type"] != "User"
        {
            return Err("Original rollback PR changed".into());
        }
        let user = pr_contract::field(pr, "/user/login")?;
        if user.is_empty()
            || !user
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err("Invalid original PR author".into());
        }
        reads.push((
            "rollback-pr-permission",
            format!("collaborators/{user}/permission"),
        ));
        references.push(None);
        if !consumed {
            reads.push(("rollback-pr-ref", format!("git/ref/heads/{branch}")));
            references.push(Some((branch, head)));
        }
    }
    for (value, reference) in adapter
        .read_group(repository, &reads)?
        .iter()
        .zip(references)
    {
        if let Some((branch, head)) = reference {
            if value["ref"] != format!("refs/heads/{branch}")
                || value["object"]["type"] != "commit"
                || value["object"]["sha"] != head
            {
                return Err("Original rollback PR ref changed".into());
            }
        } else {
            pr_contract::maintainer(value)?;
        }
    }
    let receipt = &prs[0];
    let integration = &prs[1];
    let raw = pr_contract::field(receipt, "/body")?;
    for (key, value) in [
        ("vize-release-pin", "immutable-v1".into()),
        ("vize-release-bump", "minor".into()),
        ("vize-release-pin-head", pinned.candidate.head.clone()),
        ("vize-release-pin-cut", pinned.cut.clone()),
        ("vize-release-base-version", pinned.base_version.clone()),
        ("vize-release-integration", pinned.integration.to_string()),
    ] {
        if marker(raw, key)? != value {
            return Err("Original rollback source markers changed".into());
        }
    }
    let canonical = body::generated(
        pinned.candidate.number,
        &pinned.cut,
        &pinned.candidate.head,
        &pinned.candidate.tag,
    );
    body::qualify(
        pr_contract::field(integration, "/body")?,
        &canonical,
        repository,
        expected.number,
    )?;
    if integration["merged"] == false
        && candidate::integration_fields(integration, repository, expected.number, &expected.head)?
            != (
                pinned.candidate.number,
                pinned.candidate.head.clone(),
                pinned.cut.clone(),
                pinned.candidate.tag.clone(),
            )
    {
        return Err("Original rollback integration markers changed".into());
    }
    context
        .operator
        .compensation_authority(pinned, context.actor, adapter)
}

/// The actual watch error route, also exercised after a real lease acquisition.
/// Only bounded identity reads and withdrawal of its already-owned entry occur.
pub(super) fn finish_owned_watch(
    failure: &str,
    context: &Rollback<'_>,
    execute: impl Fn(&str, &[&str], &Path) -> Result<String, String>,
    batch: impl Fn(&str, &[&str], &Path) -> Result<Vec<Value>, String>,
) -> Result<(), String> {
    let mut verify = |candidate: &pr_contract::Candidate| {
        if candidate != context.candidate {
            return Err("Rollback candidate changed".into());
        }
        let mut bounded = |program: &str, args: &[&str], root: &Path| execute(program, args, root);
        let mut retain = retain_queue;
        let mut grouped =
            |repo: &str, endpoints: &[&str], root: &Path| batch(repo, endpoints, root);
        let mut adapter = queue::Adapter {
            root: context.root,
            execute: &mut bounded,
            retain: &mut retain,
            batch: Some(&mut grouped),
        };
        authenticate_compensation(context, &mut adapter)
    };
    let mut bounded = |program: &str, args: &[&str], root: &Path| execute(program, args, root);
    let mut retain = retain_queue;
    let mut grouped = |repo: &str, endpoints: &[&str], root: &Path| batch(repo, endpoints, root);
    let mut adapter = queue::Adapter {
        root: context.root,
        execute: &mut bounded,
        retain: &mut retain,
        batch: Some(&mut grouped),
    };
    match queue::cleanup(context.candidate, context.owner, &mut verify, &mut adapter) {
        Ok(()) => Err(failure.to_owned()),
        Err(error) => Err(format!(
            "{failure}\nOwned queue withdrawal refused; preserve its complete evidence: {error}"
        )),
    }
}
