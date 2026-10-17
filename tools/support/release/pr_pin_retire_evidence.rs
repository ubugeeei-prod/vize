//! Complete bounded API snapshots and relevant failure logs, without copying artifact bodies.
use super::super::{
    pr_budget::{self, Budget},
    pr_github as github,
};
use super::{Source, run_identity};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
#[path = "pr_pin_retire_logs.rs"]
mod logs;

pub(super) fn terminal_failed(run: &Value) -> Result<(), String> {
    if run["status"] != "completed"
        || !matches!(
            run["conclusion"].as_str(),
            Some("failure" | "cancelled" | "timed_out")
        )
    {
        return Err("Retirement requires the original run terminal failed/cancelled, never success or pending".into());
    }
    if run["run_attempt"].as_u64().is_none_or(|n| n == 0) {
        return Err("Missing original run attempt".into());
    }
    Ok(())
}

pub(super) fn original_failure_jobs(
    release: &Value,
    release_jobs: &[Value],
    operator: &Value,
    operator_jobs: &[Value],
) -> Result<(), String> {
    super::retire_ledger::original_failure_jobs(release, release_jobs, operator, operator_jobs)
}

pub(super) fn original_operator(run: &Value, source: &Source, id: u64) -> Result<(), String> {
    terminal_failed(run)?;
    if run["id"].as_u64() != Some(id)
        || run["head_sha"].as_str() != Some(super::stable::operator_cut(source))
        || run["head_branch"] != "main"
        || run["event"] != "workflow_dispatch"
        || run["path"] != ".github/workflows/release-operator.yml"
        || run
            .pointer("/head_repository/full_name")
            .and_then(Value::as_str)
            != Some(&source.candidate.repository)
        || run.pointer("/actor/login").and_then(Value::as_str) != Some(&source.candidate.author)
        || run
            .pointer("/triggering_actor/login")
            .and_then(Value::as_str)
            != Some(&source.candidate.author)
    {
        return Err(
            "Original operator is not the authenticated own-repo/main/C/maintainer execution"
                .into(),
        );
    }
    Ok(())
}

pub(super) fn inventory(
    repository: &str,
    resource: &str,
    key: &str,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<(Vec<Value>, Vec<Value>), String> {
    let mut pages = Vec::new();
    let mut rows = Vec::new();
    let mut total = None;
    for page in 1..=100 {
        pr_budget::check(budget)?;
        let separator = if resource.contains('?') { '&' } else { '?' };
        let value = github::api(
            repository,
            &format!("{resource}{separator}per_page=100&page={page}"),
            root,
        )?;
        pr_budget::check(budget)?;
        let count = value["total_count"]
            .as_u64()
            .ok_or("Missing complete retirement inventory total")?;
        if total.is_some_and(|prior| prior != count) {
            return Err("Retirement inventory changed during pagination".into());
        }
        total = Some(count);
        let entries = value[key]
            .as_array()
            .ok_or("Missing retirement inventory entries")?;
        let last = entries.len() < 100;
        rows.extend(entries.iter().cloned());
        pages.push(value);
        if last {
            if rows.len() as u64 != count {
                return Err("Retirement inventory is incomplete".into());
            }
            let mut ids = std::collections::BTreeSet::new();
            if rows.iter().any(|row| {
                row["id"]
                    .as_u64()
                    .is_none_or(|id| id == 0 || !ids.insert(id))
            }) {
                return Err("Retirement inventory has missing or duplicate IDs".into());
            }
            return Ok((pages, rows));
        }
    }
    Err("Retirement inventory exceeded its bounded pagination".into())
}

pub(super) fn no_other_operator(
    repository: &str,
    context: Option<&Value>,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<Value, String> {
    let (pages, runs) = inventory(
        repository,
        "actions/workflows/release-operator.yml/runs",
        "workflow_runs",
        budget,
        root,
    )?;
    let current = context.and_then(|v| v["id"].as_u64());
    for run in &runs {
        if run["status"] != "completed" && run["id"].as_u64() != current {
            return Err("A running or ambiguous release operator forbids retirement".into());
        }
    }
    if let Some(current) = current {
        let context = context.unwrap();
        let run = runs
            .iter()
            .find(|run| run["id"].as_u64() == Some(current))
            .ok_or("Current hosted retirement is absent from operator inventory")?;
        current_operator(run, context, repository)?;
    }
    Ok(json!({"pages":pages,"currentOperatorContext":context}))
}

pub(super) fn current_operator(
    run: &Value,
    context: &Value,
    repository: &str,
) -> Result<(), String> {
    if run["id"] != context["id"]
        || context["id"].as_u64().is_none_or(|n| n == 0)
        || run["event"] != "workflow_dispatch"
        || run["head_branch"] != "main"
        || run["head_sha"] != context["sha"]
        || run.pointer("/actor/login") != context.get("actor")
        || run.pointer("/triggering_actor/login") != context.get("actor")
        || context["workflowRef"]
            != format!("{repository}/.github/workflows/release-operator.yml@refs/heads/main")
        || run["path"] != ".github/workflows/release-operator.yml"
        || run
            .pointer("/head_repository/full_name")
            .and_then(Value::as_str)
            != Some(repository)
    {
        return Err("Current retirement is not the supported main-only operator".into());
    }
    Ok(())
}

pub(super) fn publication_jobs(jobs: &[Value], workflow: &str) -> Result<(), String> {
    let mut active = false;
    let mut expected = Vec::new();
    for line in workflow.lines() {
        if line.starts_with("  ") && !line.starts_with("   ") && line.ends_with(':') {
            let id = line.trim().trim_end_matches(':');
            active = id.starts_with("release-npm-")
                || matches!(
                    id,
                    "release-vscode-extension" | "release-crates" | "create-github-release"
                );
        }
        if active && let Some(name) = line.strip_prefix("    name: ") {
            if !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b" @/.-".contains(&b))
            {
                return Err("Unsupported source publication name".into());
            }
            expected.push(name);
            active = false;
        }
    }
    if expected.is_empty()
        || expected
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != expected.len()
    {
        return Err("Original H lacks distinct identifiable publication phases".into());
    }
    for name in expected {
        let publication: Vec<_> = jobs
            .iter()
            .filter(|job| job["name"].as_str() == Some(name))
            .collect();
        if publication.is_empty() {
            return Err("Original run publication-phase inventory is incomplete/ambiguous".into());
        }
        let mut attempts = std::collections::BTreeSet::new();
        for job in publication {
            let attempt = job["run_attempt"]
                .as_u64()
                .filter(|n| *n > 0)
                .ok_or("Original publication attempt missing")?;
            if !attempts.insert(attempt)
                || job["status"] != "completed"
                || !matches!(job["conclusion"].as_str(), Some("cancelled" | "skipped"))
                || job["steps"]
                    .as_array()
                    .is_none_or(|steps| !steps.is_empty())
            {
                return Err(
                    "An original publication attempt started or has ambiguous evidence".into(),
                );
            }
        }
    }
    Ok(())
}

/// Execute the same pure TypeScript policy for live snapshots and archived replay.
pub(super) fn qualification_policy(value: &Value, root: &Path) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if bytes.len() > super::retire_archive::MAX_BYTES {
        return Err("Qualification evidence exceeds the existing archive bound".into());
    }
    let mut command = Command::new("node");
    command
        .arg(root.join("tools/support/release/retirement_qualification.ts"))
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let budget = pr_budget::configured()?;
    pr_budget::child_command(&mut command, budget.as_ref())?;
    let mut child = command
        .spawn()
        .map_err(|e| format!("Could not start qualification policy: {e}"))?;
    let write = child
        .stdin
        .take()
        .ok_or("Missing qualification policy stdin")?
        .write_all(&bytes);
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    write.map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "Pre-release qualification evidence refused: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !output.stdout.is_empty() {
        return Err("Qualification policy emitted unexpected output".into());
    }
    Ok(())
}

pub(super) fn collect(
    source: &Source,
    request: &super::retire::Request,
    guards: &Value,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<(Value, BTreeMap<String, Vec<u8>>), String> {
    let repository = &source.candidate.repository;
    let run_id = request.failure.run();
    let operator_id = request.operator;
    let (run_pages, mut runs) = inventory(
        repository,
        &format!("actions/runs?head_sha={}", source.candidate.head),
        "workflow_runs",
        budget,
        root,
    )?;
    if runs.iter().any(|run| {
        run["head_sha"].as_str() != Some(&source.candidate.head)
            || run
                .pointer("/head_repository/full_name")
                .and_then(Value::as_str)
                != Some(repository)
    }) {
        return Err("H workflow inventory contains foreign source identity".into());
    }
    pr_budget::check(budget)?;
    let original_run = github::api(repository, &format!("actions/runs/{run_id}"), root)?;
    if request.failure.before_release() {
        qualification_policy(guards, root)?;
        if original_run != guards["qualificationRun"] {
            return Err("Original qualification changed during evidence collection".into());
        }
    } else {
        run_identity(&original_run, source, run_id)?;
        terminal_failed(&original_run)?;
    }
    if !runs.iter().any(|run| run["id"].as_u64() == Some(run_id)) {
        return Err("Original R is absent from H workflow inventory".into());
    }
    pr_budget::check(budget)?;
    let operator = github::api(repository, &format!("actions/runs/{operator_id}"), root)?;
    original_operator(&operator, source, operator_id)?;
    runs.push(operator);
    let mut observations = Vec::new();
    let mut logs = BTreeMap::new();
    let mut total_log_bytes = 0usize;
    let mut raw_log_bytes = 0usize;
    let mut raw_log_sources = BTreeMap::new();
    for run in runs {
        pr_budget::check(budget)?;
        let id = run["id"].as_u64().ok_or("Missing diagnostic run ID")?;
        let (job_pages, jobs) = inventory(
            repository,
            &format!("actions/runs/{id}/jobs?filter=all"),
            "jobs",
            budget,
            root,
        )?;
        if id == run_id && !request.failure.before_release() {
            publication_jobs(
                &jobs,
                &super::metadata::text(
                    &source.candidate.head,
                    ".github/workflows/release.yml",
                    root,
                )?,
            )?;
        }
        for job in &jobs {
            super::retire_ledger::job(job, &run)?;
            if job["conclusion"] == "failure" || job["conclusion"] == "timed_out" {
                let job_id = job["id"].as_u64().ok_or("Missing failure job ID")?;
                pr_budget::check(budget)?;
                let raw = logs::capture(repository, job_id, root)?;
                pr_budget::check(budget)?;
                raw_log_bytes = raw_log_bytes
                    .checked_add(raw.len())
                    .filter(|total| *total <= super::retire_archive::MAX_BYTES)
                    .ok_or("Complete decoded failure logs exceed bounded 64 MiB")?;
                let raw_oid = super::retire_log_storage::blob_identity(&raw, root)?;
                let bytes = super::retire_log_storage::gzip(&raw)?;
                total_log_bytes = total_log_bytes
                    .checked_add(bytes.len())
                    .ok_or("Failure log size overflow")?;
                if total_log_bytes > super::retire_archive::MAX_LOG_BYTES {
                    return Err("Complete compressed failure logs exceed bounded 8 MiB aggregate; preserve originals and stop".into());
                }
                let path = format!("failure/{job_id}.log.gz");
                raw_log_sources.insert(path.clone(), json!({"oid":raw_oid,"bytes":raw.len()}));
                logs.insert(path, bytes);
            }
        }
        let (artifact_pages, artifacts) = inventory(
            repository,
            &format!("actions/runs/{id}/artifacts"),
            "artifacts",
            budget,
            root,
        )?;
        for artifact in &artifacts {
            let digest = artifact["digest"]
                .as_str()
                .ok_or("Artifact provider digest missing; archive cannot claim complete custody")?;
            if !digest.strip_prefix("sha256:").is_some_and(|value| {
                value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
            }) || artifact["size_in_bytes"].as_u64().is_none()
                || artifact["expires_at"].as_str().is_none()
                || artifact["expired"].as_bool().is_none()
                || artifact.pointer("/workflow_run/id").and_then(Value::as_u64) != Some(id)
            {
                return Err("Incomplete original artifact digest/size/expiry/run metadata".into());
            }
        }
        observations.push(json!({"run":run,"jobPages":job_pages,"artifactPages":artifact_pages}));
    }
    pr_budget::check(budget)?;
    Ok((
        json!({"HRunPages":run_pages,"observations":observations,
        "failureLogEncoding":"gzip","failureLogSources":raw_log_sources,
        "artifactCustody":"Metadata/provider digests and original Actions URLs/expiry only; binary bodies were not copied, are not permanently held, and cannot qualify a replacement cut.",
        "pendingDiagnosticRows":"Old source/full-H diagnostics may finish independently; their current states are retained honestly and are not new retirement gates."}),
        logs,
    ))
}
