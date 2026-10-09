//! Abandon an unpublished failed immutable cut without rewriting its source/history.
use super::super::{pr_budget, pr_contract, pr_github as github, pr_start};
use super::{lock, retire_archive as archive, retire_evidence as evidence, retire_guard as guard};
use serde_json::{Value, json};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub(super) struct Request {
    pub(super) source: u64,
    pub(super) head: String,
    pub(super) tag: String,
    pub(super) run: u64,
    pub(super) operator: u64,
}

impl Request {
    pub(super) fn from_identity(fields: &Value) -> Result<Self, String> {
        let positive = |key: &str| {
            fields[key]
                .as_u64()
                .filter(|n| *n > 0)
                .ok_or(format!("Missing original {key}"))
        };
        Ok(Self {
            source: positive("sourcePr")?,
            head: fields["head"].as_str().ok_or("Missing original H")?.into(),
            tag: fields["tag"].as_str().ok_or("Missing original tag")?.into(),
            run: positive("releaseRun")?,
            operator: positive("operatorRun")?,
        })
    }
}

fn same_archive(receipt: &Value, guards: &Value, absence: &Value) -> Result<(), String> {
    if archive::identity(receipt)? != &guards["identity"] {
        return Err("Original C/H/pin/M/R/operator identities or attempts differ from the retirement archive".into());
    }
    for key in ["sourcePr", "integrationPr"] {
        for field in ["body", "title"] {
            if receipt.pointer(&format!("/guards/{key}/{field}"))
                != guards.pointer(&format!("/{key}/{field}"))
            {
                return Err(
                    "Original source/integration receipt body or title changed after retirement"
                        .into(),
                );
            }
        }
    }
    if receipt.pointer("/registryAbsence/plan") != absence.get("plan")
        || receipt
            .pointer("/originalEvidence/observations")
            .and_then(Value::as_array)
            .is_none()
        || receipt["artifactCustody"]
            != "Original Actions metadata/digests/expiry only; no permanent binary artifact body custody or reuse qualification"
    {
        return Err("Incomplete or changed immutable retirement source/evidence ledger".into());
    }
    Ok(())
}

pub fn retire(
    number: u64,
    head: &str,
    tag: &str,
    run: u64,
    operator_run: u64,
    root: &Path,
) -> Result<(), String> {
    let budget = pr_budget::configured()?;
    pr_budget::check(budget.as_ref())?;
    if [number, run, operator_run].contains(&0) || run == operator_run {
        return Err("Positive distinct original retirement identities required".into());
    }
    pr_contract::sha(head)?;
    pr_contract::tag(tag)?;
    let repository = guard::authorize(root)?;
    let request = Request {
        source: number,
        head: head.into(),
        tag: tag.into(),
        run,
        operator: operator_run,
    };
    let (source, initial) =
        guard::capture(&repository, &request, false, false, budget.as_ref(), root)?;
    let absence = guard::absence(&source, budget.as_ref(), root)?;
    if let Some((_, receipt)) = archive::read(tag, root)? {
        same_archive(&receipt, &initial, &absence)?;
    }
    // Never remove a predecessor's lease. Acquire only after its terminal/no-lease proof.
    pr_budget::check(budget.as_ref())?;
    let operator = lock::acquire(tag, head, root)?;
    let (_, fresh) = guard::capture(&repository, &request, false, true, budget.as_ref(), root)?;
    if initial["identity"] != fresh["identity"] {
        return Err("Original cut changed while acquiring retirement authority".into());
    }
    operator.verify()?;
    let archive_head = if let Some((archived, receipt)) = archive::read(tag, root)? {
        same_archive(&receipt, &fresh, &absence)?;
        archived
    } else {
        let (original_evidence, logs) =
            evidence::collect(&source, run, operator_run, budget.as_ref(), root)?;
        operator.verify()?;
        let (_, final_guards) =
            guard::capture(&repository, &request, false, true, budget.as_ref(), root)?;
        if final_guards["identity"] != fresh["identity"] {
            return Err("Original cut changed during evidence collection".into());
        }
        let final_absence = guard::absence(&source, budget.as_ref(), root)?;
        let receipt = json!({"schema":"vize-unpublished-retirement-v1","identity":fresh["identity"],"guards":final_guards,
            "createdAtUnixSeconds":SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_secs(),
            "registryAbsence":final_absence,"originalEvidence":original_evidence,
            "artifactCustody":"Original Actions metadata/digests/expiry only; no permanent binary artifact body custody or reuse qualification",
            "replacementQualification":"A new C/H/R must run fresh full gates; archived evidence never authorizes publication."});
        archive::install_authorized(receipt, &logs, root, &|| {
            pr_budget::check(budget.as_ref())?;
            operator.verify()?;
            let (_, guards) =
                guard::capture(&repository, &request, false, true, budget.as_ref(), root)?;
            if guards["identity"] != initial["identity"] {
                return Err("Original cut changed before archive ref mutation".into());
            }
            pr_budget::check(budget.as_ref())?;
            operator.verify()
        })?
    };
    // PR state updates cannot be an atomic Git transaction. Recheck each exact
    // identity under the cooperative lease, preserve partial progress and require
    // both closed/live guards before the archive can reserve the next version.
    for number in [source.integration, source.candidate.number] {
        operator.verify()?;
        let (_, before) =
            guard::capture(&repository, &request, false, true, budget.as_ref(), root)?;
        if before["identity"] != initial["identity"] {
            return Err(
                "Receipt changed before closure; preserved archive and partial progress".into(),
            );
        }
        let pr = github::api(&repository, &format!("pulls/{number}"), root)?;
        if pr["state"] == "open" {
            let expected_head = if number == source.integration {
                initial["identity"]["integrationHead"].as_str()
            } else {
                Some(head)
            };
            if pr.pointer("/head/sha").and_then(Value::as_str) != expected_head {
                return Err("PR head changed immediately before closure".into());
            }
            pr_budget::check(budget.as_ref())?;
            operator.verify()?;
            github::output(
                "gh",
                &[
                    "api",
                    "--method",
                    "PATCH",
                    &format!("repos/{repository}/pulls/{number}"),
                    "-f",
                    "state=closed",
                ],
                root,
            )?;
        }
    }
    operator.verify()?;
    let (_, closed) = guard::capture(&repository, &request, true, true, budget.as_ref(), root)?;
    let absent_now = guard::absence(&source, budget.as_ref(), root)?;
    let (observed, receipt) =
        archive::read(tag, root)?.ok_or("Archive disappeared after receipt closure")?;
    if observed != archive_head {
        return Err("Retirement archive changed during partial recovery".into());
    }
    same_archive(&receipt, &closed, &absent_now)?;
    pr_budget::check(budget.as_ref())?;
    operator.verify()?;
    println!(
        "Retired unpublished {tag}: source #{number}, H={head}, R={run}, archive={archive_head}. Original source/pin/M refs and objects were preserved; next official minor skips this reserved version. No tag or public artifacts were changed."
    );
    Ok(())
}

pub(super) fn reject_resume(tag: &str, root: &Path) -> Result<(), String> {
    if archive::remote(&archive::reference(tag), root)?.is_some() {
        return Err("This cut has a retirement archive; never resume/requalify it. Prepare a new official minor cut after the authenticated reservation completes.".into());
    }
    Ok(())
}

pub(super) fn ordinary_minor(version: &str) -> Result<String, String> {
    let pieces: Vec<_> = version.split('-').collect();
    if pieces.len() > 2
        || pieces.get(1).is_some_and(|value| {
            value.is_empty()
                || !value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.')
        })
    {
        return Err("Invalid canonical minor base version".into());
    }
    let parts: Vec<_> = pieces[0].split('.').collect();
    if parts.len() != 3
        || parts.iter().any(|part| {
            part.is_empty()
                || !part.bytes().all(|b| b.is_ascii_digit())
                || (part.len() > 1 && part.starts_with('0'))
                || part.parse::<u64>().is_err()
        })
    {
        return Err("Pinned minor requires a canonical supported semantic base version".into());
    }
    let minor = parts[1]
        .parse::<u64>()
        .map_err(|_| "Invalid minor version")?
        .checked_add(1)
        .ok_or("Minor version overflow")?;
    Ok(format!("{}.{minor}.0", parts[0]))
}

pub(super) fn next_minor(
    repository: &str,
    base: &str,
    budget: Option<&pr_budget::Budget>,
    root: &Path,
) -> Result<String, String> {
    let mut target = ordinary_minor(base)?;
    for _ in 0..64 {
        pr_budget::check(budget)?;
        let tag = format!("v{target}");
        let Some((_, receipt)) = archive::read(&tag, root)? else {
            return Ok(target);
        };
        let identity = archive::identity(&receipt)?;
        if identity["repository"].as_str() != Some(repository)
            || identity["baseVersion"].as_str() != Some(base)
        {
            return Err(
                "A retirement reservation belongs to a foreign repository/base version".into(),
            );
        }
        let request = Request::from_identity(identity)?;
        let (source, guards) = guard::capture(repository, &request, true, false, budget, root)?;
        let absence = guard::absence(&source, budget, root)?;
        same_archive(&receipt, &guards, &absence)?;
        println!(
            "Skipping authenticated retired unpublished {tag}; original H and evidence stay preserved."
        );
        target = ordinary_minor(&target)?;
    }
    Err("Retirement reservation chain exceeded its bound; no candidate prepared".into())
}

pub fn validate_preparation_target(
    bump: &str,
    base: &str,
    target: &str,
    root: &Path,
) -> Result<(), String> {
    if bump != "minor" {
        return Err("Reserved target is internal to pinned minor preparation".into());
    }
    let budget = pr_budget::configured()?;
    pr_budget::check(budget.as_ref())?;
    let repository = guard::authorize(root)?;
    guard::no_replacements(root)?;
    if github::version(root)? != base {
        return Err("Preparation base version changed".into());
    }
    if next_minor(&repository, base, budget.as_ref(), root)? != target {
        return Err("Preparation target is not the authenticated next unreserved minor".into());
    }
    Ok(())
}

pub(super) fn prepare(
    bump: &str,
    tag: &str,
    budget: Option<&pr_budget::Budget>,
    root: &Path,
) -> Result<(), String> {
    if bump == "minor" {
        pr_start::prepare_pinned_minor(tag.trim_start_matches('v'), budget, root)
    } else {
        pr_start::prepare(bump, root)
    }
}
