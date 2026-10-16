use super::{REQUIRED_SEMVER_CRATES, flatten_collection, matrix_selection, repo_root};
use serde_json::Value;

fn required_workflow_job_names(workflow_name: &str, run: &Value) -> Result<Vec<String>, String> {
    if workflow_name == "Real Project Matrix" {
        return matrix_selection::matrix_job_names(run, &repo_root()?);
    }
    Ok(match workflow_name {
        "Check" => [
            "test-scripts",
            "test-js-packages",
            "build-js-packages",
            "full-js-report",
        ]
        .into_iter()
        .map(str::to_string)
        .chain(
            REQUIRED_SEMVER_CRATES
                .iter()
                .map(|crate_name| format!("cargo-semver-checks ({crate_name})")),
        )
        .collect(),
        "Benchmark" => vec!["pr-benchmark-budget".to_string()],
        "Fuzz" => vec![
            "Fuzz sfc_parse".to_string(),
            "Fuzz template_lexer".to_string(),
            "Fuzz js_ts_expression".to_string(),
            "Fuzz css_parse".to_string(),
            "Fuzz template_compile".to_string(),
        ],
        _ => Vec::new(),
    })
}

pub(super) fn assert_required_workflow_jobs(
    workflow_name: &str,
    jobs_response: &[Value],
    run: &Value,
) -> Result<(), String> {
    let job_names = required_workflow_job_names(workflow_name, run)?;
    let mut jobs = flatten_collection(jobs_response, "jobs");
    if jobs.is_empty() && jobs_response.iter().all(Value::is_object) {
        jobs.extend(jobs_response);
    }
    for name in job_names {
        let matching = jobs
            .iter()
            .filter(|job| job.get("name").and_then(Value::as_str) == Some(name.as_str()))
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            return Err(format!(
                "{workflow_name} must contain exactly one successful {name} job; found {}",
                matching.len()
            ));
        }
        let job = matching[0];
        if job.get("status").and_then(Value::as_str) != Some("completed")
            || job.get("conclusion").and_then(Value::as_str) != Some("success")
        {
            return Err(format!(
                "{workflow_name} required job {} is {}/{}",
                name,
                job.get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                job.get("conclusion")
                    .and_then(Value::as_str)
                    .unwrap_or("no conclusion")
            ));
        }
    }
    Ok(())
}

pub(super) fn summarize_required_workflow_job_failures(
    workflow_name: &str,
    jobs_response: &[Value],
    run: &Value,
) -> Option<String> {
    let job_names = match required_workflow_job_names(workflow_name, run) {
        Ok(names) => names,
        Err(error) => return Some(error),
    };
    if job_names.is_empty() {
        return None;
    }
    let mut jobs = flatten_collection(jobs_response, "jobs");
    if jobs.is_empty() && jobs_response.iter().all(Value::is_object) {
        jobs.extend(jobs_response);
    }
    let mut failures = Vec::new();
    for name in job_names {
        let matching = jobs
            .iter()
            .filter(|job| job.get("name").and_then(Value::as_str) == Some(name.as_str()))
            .collect::<Vec<_>>();
        match matching.as_slice() {
            [] => failures.push(format!("{name}=missing")),
            [job] => {
                let status = job
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let conclusion = job
                    .get("conclusion")
                    .and_then(Value::as_str)
                    .unwrap_or("no conclusion");
                if status != "completed" || conclusion != "success" {
                    failures.push(format!("{name}={status}/{conclusion}"));
                }
            }
            _ => failures.push(format!("{name}=duplicate({})", matching.len())),
        }
    }
    if failures.is_empty() {
        return None;
    }
    const MAX_REPORTED_JOBS: usize = 8;
    let omitted = failures.len().saturating_sub(MAX_REPORTED_JOBS);
    let mut reported = failures
        .into_iter()
        .take(MAX_REPORTED_JOBS)
        .collect::<Vec<_>>();
    if omitted > 0 {
        reported.push(format!("and {omitted} more"));
    }
    Some(format!("required jobs: {}", reported.join(", ")))
}

pub(super) fn required_jobs_contain_terminal_failure(
    workflow_name: &str,
    jobs_response: &[Value],
    run: &Value,
) -> bool {
    let Ok(job_names) = required_workflow_job_names(workflow_name, run) else {
        return true;
    };
    if job_names.is_empty() {
        return false;
    }
    let mut jobs = flatten_collection(jobs_response, "jobs");
    if jobs.is_empty() && jobs_response.iter().all(Value::is_object) {
        jobs.extend(jobs_response);
    }
    job_names.into_iter().any(|name| {
        jobs.iter()
            .filter(|job| job.get("name").and_then(Value::as_str) == Some(name.as_str()))
            .any(|job| {
                job.get("status").and_then(Value::as_str) == Some("completed")
                    && job.get("conclusion").and_then(Value::as_str) != Some("success")
            })
    })
}
