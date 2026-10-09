//! Offline completeness and actual failure-blob custody for immutable retirement archives.
use super::{retire_archive as archive, retire_evidence as evidence};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) fn text<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or(format!("Missing retirement ledger string {pointer}"))
}
pub(super) fn positive(value: &Value, pointer: &str) -> Result<u64, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .filter(|n| *n > 0)
        .ok_or(format!(
            "Missing positive retirement ledger field {pointer}"
        ))
}
pub(super) fn inventory<'a>(pages: &'a Value, key: &str) -> Result<Vec<&'a Value>, String> {
    let pages = pages
        .as_array()
        .filter(|p| !p.is_empty() && p.len() <= 100)
        .ok_or("Missing bounded complete retirement pages")?;
    let mut rows = Vec::new();
    let mut total = None;
    let mut ids = BTreeSet::new();
    for (index, page) in pages.iter().enumerate() {
        let count = page["total_count"]
            .as_u64()
            .filter(|n| *n <= 10_000)
            .ok_or("Missing bounded retirement inventory total")?;
        if total.is_some_and(|n| n != count) {
            return Err("Retirement page totals differ".into());
        }
        total = Some(count);
        let entries = page[key]
            .as_array()
            .filter(|r| r.len() <= 100)
            .ok_or("Missing retirement page rows")?;
        if index + 1 < pages.len() && entries.len() != 100 {
            return Err("Incomplete intermediate retirement page".into());
        }
        for row in entries {
            if !ids.insert(positive(row, "/id")?) {
                return Err("Duplicate retirement inventory ID".into());
            }
            rows.push(row);
        }
    }
    if total != Some(rows.len() as u64) {
        return Err("Incomplete retirement inventory total".into());
    }
    Ok(rows)
}
fn state(value: &Value) -> Result<(), String> {
    let status = text(value, "/status")?;
    if !matches!(
        status,
        "queued" | "in_progress" | "waiting" | "pending" | "requested" | "completed"
    ) {
        return Err("Unknown retirement diagnostic status".into());
    }
    let conclusion = value
        .get("conclusion")
        .ok_or("Missing diagnostic conclusion")?;
    if (status != "completed" && !conclusion.is_null())
        || (status == "completed"
            && !matches!(
                conclusion.as_str(),
                Some(
                    "success"
                        | "failure"
                        | "cancelled"
                        | "timed_out"
                        | "action_required"
                        | "neutral"
                        | "skipped"
                        | "stale"
                        | "startup_failure"
                )
            ))
    {
        return Err("Diagnostic status/conclusion are inconsistent".into());
    }
    Ok(())
}
pub(super) fn own(run: &Value, repository: &str) -> Result<(), String> {
    if text(run, "/repository/full_name")? != repository
        || text(run, "/head_repository/full_name")? != repository
    {
        return Err("Foreign retirement run repository".into());
    }
    positive(run, "/repository/id")?;
    positive(run, "/head_repository/id")?;
    positive(run, "/run_attempt")?;
    state(run)?;
    Ok(())
}
pub(super) fn job(job: &Value, run: &Value) -> Result<(), String> {
    if positive(job, "/run_id")? != positive(run, "/id")?
        || text(job, "/head_sha")? != text(run, "/head_sha")?
        || positive(job, "/run_attempt")? > positive(run, "/run_attempt")?
    {
        return Err("Retirement job run/head/attempt differs".into());
    }
    text(job, "/name")?;
    state(job)?;
    if matches!(job["conclusion"].as_str(), Some("failure" | "timed_out"))
        && job["status"] != "completed"
    {
        return Err("Nonterminal retirement failure job".into());
    }
    if !job["steps"].is_array() {
        return Err("Retirement job steps missing".into());
    }
    Ok(())
}
pub(super) fn original_failure_jobs(
    release: &Value,
    release_jobs: &[Value],
    operator: &Value,
    operator_jobs: &[Value],
) -> Result<(), String> {
    evidence::terminal_failed(release)?;
    evidence::terminal_failed(operator)?;
    if positive(release, "/id")? == positive(operator, "/id")? {
        return Err("Original failure runs must differ".into());
    }
    let mut ids = BTreeSet::new();
    let mut failed = false;
    for (run, jobs) in [(release, release_jobs), (operator, operator_jobs)] {
        for row in jobs {
            job(row, run)?;
            if !ids.insert(positive(row, "/id")?) {
                return Err("Original failure job IDs collide".into());
            }
            failed |= matches!(row["conclusion"].as_str(), Some("failure" | "timed_out"));
        }
    }
    if !failed {
        return Err("Original R/operator actual failed or timed-out job required".into());
    }
    Ok(())
}
fn artifact(value: &Value, run: &Value, repository: &str) -> Result<(), String> {
    let id = positive(value, "/id")?;
    let digest = text(value, "/digest")?;
    let hash = digest
        .strip_prefix("sha256:")
        .ok_or("Artifact SHA256 provider digest missing")?;
    if hash.len() != 64
        || !hash.bytes().all(|b| b.is_ascii_hexdigit())
        || positive(value, "/size_in_bytes").is_err()
        || value["expired"].as_bool().is_none()
        || !text(value, "/expires_at")?.contains('T')
        || !text(value, "/expires_at")?.ends_with('Z')
        || positive(value, "/workflow_run/id")? != positive(run, "/id")?
        || positive(value, "/workflow_run/repository_id")? != positive(run, "/repository/id")?
        || positive(value, "/workflow_run/head_repository_id")?
            != positive(run, "/head_repository/id")?
        || text(value, "/workflow_run/head_sha")? != text(run, "/head_sha")?
        || text(value, "/workflow_run/head_branch")? != text(run, "/head_branch")?
    {
        return Err("Incomplete/foreign retirement artifact metadata".into());
    }
    text(value, "/name")?;
    if text(value, "/archive_download_url")?
        != format!("https://api.github.com/repos/{repository}/actions/artifacts/{id}/zip")
    {
        return Err("Foreign original artifact URL".into());
    }
    Ok(())
}
pub(super) fn workflow(head: &str, root: &Path) -> Result<String, String> {
    let raw = super::super::pr_github::git(
        &[
            "--no-replace-objects",
            "ls-tree",
            head,
            "--",
            ".github/workflows/release.yml",
        ],
        root,
    )?;
    let row: Vec<_> = raw.split_whitespace().collect();
    if row.len() != 4
        || !matches!(row[0], "100644" | "100755")
        || row[1] != "blob"
        || row[3] != ".github/workflows/release.yml"
    {
        return Err("Normal raw H publication workflow blob required".into());
    }
    super::super::pr_contract::sha(row[2])?;
    super::super::pr_github::git(&["--no-replace-objects", "cat-file", "blob", row[2]], root)
}
fn failure_objects(
    receipt: &Value,
    expected: &BTreeSet<String>,
    root: &Path,
) -> Result<(), String> {
    let objects = receipt["failureLogObjects"]
        .as_object()
        .ok_or("Missing actual failure objects")?;
    if objects.keys().cloned().collect::<BTreeSet<_>>() != *expected || expected.is_empty() {
        return Err("Actual failed jobs and failure-log objects differ or are empty".into());
    }
    let mut total = 0usize;
    for object in objects.values() {
        let oid = text(object, "/oid")?;
        super::super::pr_contract::sha(oid)?;
        let bytes = usize::try_from(positive(object, "/bytes")?)
            .map_err(|_| "Failure blob size overflow")?;
        if super::super::pr_github::git(&["--no-replace-objects", "cat-file", "-t", oid], root)?
            != "blob"
            || super::super::pr_github::git(&["--no-replace-objects", "cat-file", "-s", oid], root)?
                .parse::<usize>()
                .map_err(|_| "Invalid actual failure blob size")?
                != bytes
        {
            return Err("Actual failure blob type/size differs".into());
        }
        total = total
            .checked_add(bytes)
            .ok_or("Failure blob total overflow")?;
        if total > archive::MAX_LOG_BYTES {
            return Err("Failure logs exceed bounded 8 MiB aggregate".into());
        }
    }
    let encoded = serde_json::to_vec_pretty(receipt).map_err(|e| e.to_string())?;
    archive::check_size(encoded.len(), total)?;
    Ok(())
}

/// No live calls or artifact-body claims: all inventories must reconcile to actual archived blobs.
pub(super) fn validate(receipt: &Value, root: &Path) -> Result<(), String> {
    let identity = archive::identity(receipt)?;
    let repository = text(identity, "/repository")?;
    let head = text(identity, "/head")?;
    let release = positive(identity, "/releaseRun")?;
    let operator = positive(identity, "/operatorRun")?;
    if release == operator {
        return Err("Original R and operator must differ".into());
    }
    let guards = &receipt["guards"];
    if guards["identity"] != *identity {
        return Err("Retirement guard identity differs".into());
    }
    let source = super::retire_source::validate(receipt, root)?;
    let author = source.candidate.author.as_str();
    let evidence_value = &receipt["originalEvidence"];
    let hrows = inventory(&evidence_value["HRunPages"], "workflow_runs")?;
    let mut expected = BTreeMap::new();
    for run in &hrows {
        own(run, repository)?;
        if text(run, "/head_sha")? != head {
            return Err("Foreign H inventory source".into());
        }
        expected.insert(positive(run, "/id")?, *run);
    }
    if !expected.contains_key(&release) || expected.contains_key(&operator) {
        return Err("Original R/operator inventory binding missing or ambiguous".into());
    }
    let observations = evidence_value["observations"]
        .as_array()
        .filter(|r| r.len() == hrows.len() + 1)
        .ok_or("Incomplete H/operator observations")?;
    let mut seen = BTreeSet::new();
    let mut jobs_seen = BTreeSet::new();
    let mut artifacts_seen = BTreeSet::new();
    let mut failures = BTreeSet::new();
    for observation in observations {
        let run = &observation["run"];
        let id = positive(run, "/id")?;
        if !seen.insert(id) {
            return Err("Duplicate diagnostic observation".into());
        }
        own(run, repository)?;
        if id == operator {
            evidence::original_operator(run, &source, operator)?;
            if run != &guards["operatorRun"]
                || positive(run, "/run_attempt")? != positive(identity, "/operatorAttempt")?
            {
                return Err("Original operator/attempt differs from guard".into());
            }
        } else if expected.get(&id) != Some(&run) {
            return Err("Diagnostic observation differs from full H inventory".into());
        }
        if id == release {
            super::run_identity(run, &source, release)?;
            evidence::terminal_failed(run)?;
            if run != &guards["releaseRun"]
                || positive(run, "/run_attempt")? != positive(identity, "/releaseAttempt")?
            {
                return Err("Original R/attempt differs from guard".into());
            }
        }
        if (id == release || id == operator)
            && (text(run, "/actor/login")? != author
                || text(run, "/triggering_actor/login")? != author)
        {
            return Err("Original R/operator actor custody differs".into());
        }
        let jobs = inventory(&observation["jobPages"], "jobs")?;
        for row in &jobs {
            job(row, run)?;
            let job_id = positive(row, "/id")?;
            if !jobs_seen.insert(job_id) {
                return Err("Cross-run diagnostic job ID collision".into());
            }
            if matches!(row["conclusion"].as_str(), Some("failure" | "timed_out")) {
                failures.insert(format!("failure/{job_id}.log"));
            }
        }
        if id == release {
            evidence::publication_jobs(
                &jobs.iter().map(|row| (*row).clone()).collect::<Vec<_>>(),
                &workflow(head, root)?,
            )?;
        }
        for row in inventory(&observation["artifactPages"], "artifacts")? {
            artifact(row, run, repository)?;
            if !artifacts_seen.insert(positive(row, "/id")?) {
                return Err("Cross-run artifact ID collision".into());
            }
        }
    }
    if !seen.contains(&operator) || !expected.keys().all(|id| seen.contains(id)) {
        return Err("Original complete H/operator observations required".into());
    }
    failure_objects(receipt, &failures, root)
}
