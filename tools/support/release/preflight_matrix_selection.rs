use chrono::{DateTime, FixedOffset};
use serde_json::{Value, json};

use crate::matrix_evidence::REQUIRED_REAL_PROJECT_MATRIX_SHARD_COUNT;

pub struct MatrixArtifactSelection {
    pub artifacts: Vec<Value>,
    pub provenance: Vec<Value>,
}

pub fn select_current_matrix_artifacts<F>(
    run: &Value,
    artifacts: &[Value],
    current_jobs: &[Value],
    mut read_previous_jobs: F,
) -> Result<MatrixArtifactSelection, String>
where
    F: FnMut(u64) -> Result<Vec<Value>, String>,
{
    let attempt = positive_integer(run, "run_attempt")?;
    positive_integer(run, "id")?;
    let sha = text(run, "head_sha")?;
    if sha.len() != 40
        || !sha
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("Real Project Matrix selection requires an exact source SHA".into());
    }
    if text(run, "path")? != ".github/workflows/real-project-matrix.yml"
        || text(run, "status")? != "completed"
        || text(run, "conclusion")? != "success"
    {
        return Err("Real Project Matrix selection requires a successful selected run".into());
    }
    let mut history = Vec::new();
    for previous in 1..attempt {
        history.push((previous, read_previous_jobs(previous)?));
    }
    history.push((attempt, current_jobs.to_vec()));
    let mut selected = MatrixArtifactSelection {
        artifacts: Vec::new(),
        provenance: Vec::new(),
    };
    for shard in 0..REQUIRED_REAL_PROJECT_MATRIX_SHARD_COUNT {
        let name = format!("real-project-matrix-{shard}");
        let job_name =
            format!("real projects ({shard}/{REQUIRED_REAL_PROJECT_MATRIX_SHARD_COUNT})");
        let current = exactly_one_job(current_jobs, &job_name)?;
        assert_job(run, current, attempt)?;
        if text(current, "conclusion")? != "success" {
            return Err(format!("{name} current producing job is not successful"));
        }
        let mut executions = Vec::new();
        for (snapshot, jobs) in &history {
            for job in jobs
                .iter()
                .filter(|job| job.get("name").and_then(Value::as_str) == Some(&job_name))
            {
                // Skipped jobs produced no archive. Every completed execution is
                // retained, including failures that uploaded partial reports.
                if job.get("started_at").and_then(Value::as_str).is_none() {
                    continue;
                }
                assert_job(run, job, *snapshot)?;
                if let Some(old) = executions.iter().find(|old| same_execution(old, job)) {
                    if positive_integer(old, "run_attempt")? == *snapshot {
                        return Err(format!("{name} has ambiguous historical job identities"));
                    }
                } else {
                    executions.push(job.clone());
                }
            }
        }
        let producer = executions
            .iter()
            .find(|job| same_execution(job, current))
            .ok_or_else(|| format!("{name} has no authenticated producing execution"))?;
        let producer_attempt = positive_integer(producer, "run_attempt")?;
        let carried = producer_attempt != attempt;
        let started = timestamp(current, "started_at")?;
        let attempt_started = timestamp(run, "run_started_at")?;
        if carried != (started < attempt_started) {
            return Err(format!(
                "{name} carried-forward execution does not match selected attempt"
            ));
        }
        let mut matches = Vec::new();
        let mut historical_artifacts = Vec::new();
        for artifact in artifacts
            .iter()
            .filter(|artifact| artifact.get("name").and_then(Value::as_str) == Some(&name))
        {
            assert_artifact_source(run, artifact)?;
            let created = timestamp(artifact, "created_at")?;
            let owners = executions
                .iter()
                .filter(|job| {
                    timestamp(job, "started_at").is_ok_and(|start| start <= created)
                        && timestamp(job, "completed_at").is_ok_and(|end| created <= end)
                })
                .collect::<Vec<_>>();
            if owners.len() != 1 {
                return Err(format!(
                    "{name} artifact has unknown or ambiguous producing execution"
                ));
            }
            if same_execution(owners[0], producer) {
                if artifact.get("expired").and_then(Value::as_bool) != Some(false) {
                    return Err(format!(
                        "{name} selected artifact is expired or has no expiry state"
                    ));
                }
                matches.push(artifact);
            }
            historical_artifacts.push(artifact.clone());
        }
        if matches.len() != 1 {
            return Err(format!(
                "{name} must have exactly one artifact from the successful current execution; found {}",
                matches.len()
            ));
        }
        selected.artifacts.push(matches[0].clone());
        selected.provenance.push(json!({
            "artifact": matches[0], "producingJob": producer, "observedJob": current,
            "selectedAttempt": attempt, "carriedForward": carried,
            "historicalArtifacts": historical_artifacts,
        }));
    }
    Ok(selected)
}

fn exactly_one_job<'a>(jobs: &'a [Value], name: &str) -> Result<&'a Value, String> {
    let matches = jobs
        .iter()
        .filter(|job| job.get("name").and_then(Value::as_str) == Some(name))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(format!(
            "{name} must have exactly one current job; found {}",
            matches.len()
        ));
    }
    Ok(matches[0])
}

fn assert_job(run: &Value, job: &Value, attempt: u64) -> Result<(), String> {
    positive_integer(job, "id")?;
    if positive_integer(job, "run_id")? != positive_integer(run, "id")?
        || positive_integer(job, "run_attempt")? != attempt
        || text(job, "head_sha")? != text(run, "head_sha")?
        || text(job, "status")? != "completed"
    {
        return Err(
            "Real Project Matrix job is not bound to the selected run/attempt/source".into(),
        );
    }
    let start = timestamp(job, "started_at")?;
    let end = timestamp(job, "completed_at")?;
    if start > end || start < timestamp(run, "created_at")? || end > timestamp(run, "updated_at")? {
        return Err("Real Project Matrix job has inconsistent execution timestamps".into());
    }
    text(job, "conclusion")?;
    Ok(())
}

fn same_execution(left: &Value, right: &Value) -> bool {
    [
        "name",
        "run_id",
        "head_sha",
        "status",
        "conclusion",
        "started_at",
        "completed_at",
    ]
    .iter()
    .all(|field| left.get(field) == right.get(field))
}

fn assert_artifact_source(run: &Value, artifact: &Value) -> Result<(), String> {
    positive_integer(artifact, "id")?;
    let source = artifact
        .get("workflow_run")
        .ok_or("Matrix artifact has no source run")?;
    if positive_integer(source, "id")? != positive_integer(run, "id")?
        || text(source, "head_sha")? != text(run, "head_sha")?
        || text(source, "head_branch")? != text(run, "head_branch")?
    {
        return Err("Real Project Matrix artifact is not bound to the selected run/source".into());
    }
    Ok(())
}

fn positive_integer(value: &Value, field: &str) -> Result<u64, String> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .filter(|id| *id > 0 && *id <= 9_007_199_254_740_991)
        .ok_or_else(|| format!("Matrix evidence requires positive {field}"))
}

fn text<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| format!("Matrix evidence requires {field}"))
}

fn timestamp(value: &Value, field: &str) -> Result<DateTime<FixedOffset>, String> {
    let raw = text(value, field)?;
    let parsed = DateTime::parse_from_rfc3339(raw)
        .map_err(|_| format!("Matrix evidence has invalid {field} timestamp"))?;
    if raw.len() != 20
        || raw.get(17..19).is_none_or(|seconds| seconds > "59")
        || parsed.format("%Y-%m-%dT%H:%M:%SZ").to_string() != raw
        || parsed.offset().local_minus_utc() != 0
    {
        return Err(format!("Matrix evidence has invalid {field} timestamp"));
    }
    Ok(parsed)
}
