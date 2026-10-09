//! Reconcile retained immutable guards offline; never infer authority from a bare archive ref.
use super::super::{pr_contract, pr_github as github};
use super::{
    Candidate, Source, first_parent, marker, metadata, retire_archive as archive,
    retire_evidence as evidence, retire_guard as guard, retire_ledger as ledger,
};
use ledger::{inventory, own, positive, text};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

fn pr(
    value: &Value,
    number: u64,
    repository: &str,
    branch: &str,
    head: &str,
) -> Result<(), String> {
    if positive(value, "/number")? != number
        || text(value, "/head/sha")? != head
        || text(value, "/head/ref")? != branch
        || text(value, "/base/ref")? != "main"
        || text(value, "/head/repo/full_name")? != repository
        || text(value, "/base/repo/full_name")? != repository
        || value["merged"] != false
        || !matches!(text(value, "/state")?, "open" | "closed")
    {
        return Err("Archived original PR identity/state differs".into());
    }
    text(value, "/user/login")?;
    Ok(())
}

fn markers(pr: &Value, fields: &[(&str, String)]) -> Result<(), String> {
    let body = text(pr, "/body")?;
    for (key, expected) in fields {
        if marker(body, key)? != *expected {
            return Err(format!("Archived original PR marker {key} differs"));
        }
    }
    Ok(())
}

fn releases(guards: &Value, repository: &str, tag: &str) -> Result<(), String> {
    let value = &guards["githubReleaseInventory"];
    if value["publishedTagLookup"] != "authenticated typed HTTP 404" {
        return Err("Original authenticated published Release absence missing".into());
    }
    let pages = value["pages"]
        .as_array()
        .filter(|p| !p.is_empty() && p.len() <= 100)
        .ok_or("Missing bounded original full Release pages")?;
    let mut ids = BTreeSet::new();
    for (index, page) in pages.iter().enumerate() {
        let rows = page
            .as_array()
            .filter(|r| r.len() <= 100)
            .ok_or("Invalid Release page")?;
        if (index + 1 < pages.len() && rows.len() != 100)
            || (index + 1 == pages.len() && rows.len() == 100)
        {
            return Err("Incomplete original full Release pages".into());
        }
        for row in rows {
            let id = positive(row, "/id")?;
            if !ids.insert(id)
                || text(row, "/tag_name")? == tag
                || row["draft"].as_bool().is_none()
                || row["prerelease"].as_bool().is_none()
                || row
                    .get("published_at")
                    .is_none_or(|v| !v.is_null() && v.as_str().is_none_or(str::is_empty))
                || text(row, "/url")?
                    != format!("https://api.github.com/repos/{repository}/releases/{id}")
            {
                return Err("Ambiguous, foreign or exact-tag original Release inventory".into());
            }
        }
    }
    Ok(())
}

fn operators(guards: &Value, source: &Source, identity: &Value) -> Result<(), String> {
    let inventory_value = &guards["operatorInventory"];
    let runs = inventory(&inventory_value["pages"], "workflow_runs")?;
    let context = inventory_value
        .get("currentOperatorContext")
        .ok_or("Missing operator context")?;
    let current = if context.is_null() {
        None
    } else {
        pr_contract::sha(text(context, "/sha")?)?;
        text(context, "/actor")?;
        Some(positive(context, "/id")?)
    };
    let original = positive(identity, "/operatorRun")?;
    let mut found_original = false;
    let mut found_current = false;
    for run in runs {
        own(run, &source.candidate.repository)?;
        if text(run, "/path")? != ".github/workflows/release-operator.yml" {
            return Err("Foreign archived operator workflow".into());
        }
        let id = positive(run, "/id")?;
        if Some(id) == current {
            evidence::current_operator(run, context, &source.candidate.repository)?;
            found_current = true;
        } else if run["status"] != "completed" {
            return Err("Unbound active original operator forbids retirement".into());
        }
        if id == original {
            evidence::original_operator(run, source, original)?;
            for field in ["run_attempt", "status", "conclusion", "display_title"] {
                if run[field] != guards["operatorRun"][field] {
                    return Err("Original operator inventory/guard differs".into());
                }
            }
            found_original = true;
        }
    }
    if !found_original || (current.is_some() && !found_current) {
        return Err("Original/current operator missing from complete inventory".into());
    }
    Ok(())
}

fn failure_jobs(receipt: &Value, workflow: &str) -> Result<(), String> {
    let guards = &receipt["guards"];
    let observations = receipt["originalEvidence"]["observations"]
        .as_array()
        .ok_or("Missing original run observations")?;
    let mut inventories = Vec::new();
    for (key, run) in [
        ("release", &guards["releaseRun"]),
        ("operator", &guards["operatorRun"]),
    ] {
        own(run, text(&receipt["identity"], "/repository")?)?;
        let jobs = inventory(&guards["originalFailureJobPages"][key], "jobs")?;
        let id = positive(run, "/id")?;
        let mut matching = observations
            .iter()
            .filter(|row| row["run"]["id"].as_u64() == Some(id));
        let observation = matching.next().ok_or("Missing original run observation")?;
        if matching.next().is_some() || observation["run"] != *run {
            return Err("Original guard/observation run identity differs".into());
        }
        let actual = inventory(&observation["jobPages"], "jobs")?;
        if jobs.len() != actual.len() || jobs.iter().any(|row| !actual.contains(row)) {
            return Err("Original failure guard jobs differ from complete run evidence".into());
        }
        inventories.push(jobs.into_iter().cloned().collect::<Vec<_>>());
    }
    let publication = guards["publicationJobs"]
        .as_array()
        .ok_or("Missing original guard publication jobs")?;
    if publication.len() != inventories[0].len()
        || publication.iter().any(|row| !inventories[0].contains(row))
        || inventories[0].iter().any(|row| !publication.contains(row))
    {
        return Err("Guard publication jobs differ from all-attempt R inventory".into());
    }
    evidence::original_failure_jobs(
        &guards["releaseRun"],
        &inventories[0],
        &guards["operatorRun"],
        &inventories[1],
    )?;
    evidence::publication_jobs(publication, workflow)
}

pub(super) fn validate(receipt: &Value, root: &Path) -> Result<Source, String> {
    guard::no_replacements(root)?;
    let identity = archive::identity(receipt)?;
    let guards = &receipt["guards"];
    if guards["identity"] != *identity
        || guards["tagAbsent"] != true
        || guards["githubReleaseAbsent"] != true
    {
        return Err("Original immutable guard identity/absence differs".into());
    }
    let repository = text(identity, "/repository")?;
    let tag = text(identity, "/tag")?;
    let head = text(identity, "/head")?;
    let cut = text(identity, "/cut")?;
    let source_pr = &guards["sourcePr"];
    let source = Source {
        candidate: Candidate {
            number: positive(identity, "/sourcePr")?,
            repository: repository.into(),
            author: text(source_pr, "/user/login")?.into(),
            branch: format!("release/{tag}"),
            head: head.into(),
            base: cut.into(),
            tag: tag.into(),
            merged: false,
        },
        cut: cut.into(),
        base_version: text(identity, "/baseVersion")?.into(),
        integration: positive(identity, "/integrationPr")?,
        closed: source_pr["state"] == "closed",
    };
    if source.integration == source.candidate.number || source_pr["draft"] != true {
        return Err("Original source must be separate unmerged draft receipt".into());
    }
    pr(
        source_pr,
        source.candidate.number,
        repository,
        &source.candidate.branch,
        head,
    )?;
    markers(
        source_pr,
        &[
            ("vize-release-pin", "immutable-v1".into()),
            ("vize-release-pin-cut", cut.into()),
            ("vize-release-pin-head", head.into()),
            ("vize-release-integration", source.integration.to_string()),
            ("vize-release-base-version", source.base_version.clone()),
        ],
    )?;
    let integration = text(identity, "/integrationHead")?;
    pr(
        &guards["integrationPr"],
        source.integration,
        repository,
        &format!("release-integration/{tag}"),
        integration,
    )?;
    markers(
        &guards["integrationPr"],
        &[
            (
                "vize-release-pin-source",
                source.candidate.number.to_string(),
            ),
            ("vize-release-pin-cut", cut.into()),
            ("vize-release-pin-head", head.into()),
            ("vize-release-pin-tag", tag.into()),
        ],
    )?;
    if github::parent(head, root)? != cut {
        return Err("Original raw H sole parent differs from C".into());
    }
    metadata::verify_delta(cut, head, &source.base_version, &tag[1..], root)?;
    let pin = text(identity, "/pin")?;
    let pin_raw = github::git(&["--no-replace-objects", "cat-file", "commit", pin], root)?;
    let message = format!(
        "Vize immutable release cut\n\nSource-PR: #{}\nSource-head: {head}\nSource-cut: {cut}\nTag: {tag}\nBase-version: {}\nIntegration-PR: #{}",
        source.candidate.number, source.base_version, source.integration
    );
    if github::parent(pin, root)? != head
        || github::git(&["rev-parse", &format!("{pin}^{{tree}}")], root)?
            != github::git(&["rev-parse", &format!("{head}^{{tree}}")], root)?
        || pin_raw.split_once("\n\n").map(|(_, body)| body.trim_end()) != Some(message.as_str())
    {
        return Err("Original raw pin parent/tree/message differs".into());
    }
    let parent = github::parent(integration, root)?;
    first_parent(cut, &parent, root)?;
    metadata::verify_delta(&parent, integration, &source.base_version, &tag[1..], root)?;
    first_parent(cut, text(guards, "/mainAtGuard")?, root)?;
    let release = positive(identity, "/releaseRun")?;
    super::run_identity(&guards["releaseRun"], &source, release)?;
    evidence::terminal_failed(&guards["releaseRun"])?;
    evidence::original_operator(
        &guards["operatorRun"],
        &source,
        positive(identity, "/operatorRun")?,
    )?;
    let operator_workflow = metadata::text(cut, ".github/workflows/release-operator.yml", root)?;
    let name = operator_workflow
        .lines()
        .find_map(|line| line.strip_prefix("name: "))
        .ok_or("Missing original operator workflow name")?;
    if operator_workflow
        .lines()
        .any(|line| line.starts_with("run-name:"))
        || text(&guards["operatorRun"], "/display_title")? != name
    {
        return Err("Original operator title does not match its raw C workflow".into());
    }
    failure_jobs(receipt, &ledger::workflow(head, root)?)?;
    operators(guards, &source, identity)?;
    releases(guards, repository, tag)?;
    super::retire_absence::validate(receipt)?;
    Ok(source)
}
