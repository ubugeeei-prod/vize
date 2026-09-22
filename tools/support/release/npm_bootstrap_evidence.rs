use super::*;

pub fn validate_release_run(
    run: &Value,
    release_run_id: &str,
    repository: &str,
    tag_name: &str,
    tag_sha: &str,
) -> Result<(), String> {
    let candidate = run.get("event").and_then(Value::as_str) == Some("workflow_dispatch");
    if candidate {
        let title = value_string(run.get("display_title"));
        let prefix = format!("Release {tag_name} PR #");
        let suffix = format!(" @ {tag_sha}");
        let number = title
            .strip_prefix(&prefix)
            .and_then(|s| s.strip_suffix(&suffix));
        if !number.is_some_and(|n| {
            !n.is_empty() && !n.starts_with('0') && n.bytes().all(|byte| byte.is_ascii_digit())
        }) {
            return Err("Release run does not match the failed exact-tag release contract: invalid candidate title".into());
        }
    }
    let expected = BTreeMap::from([
        ("conclusion", "failure".to_string()),
        (
            "event",
            if candidate {
                "workflow_dispatch"
            } else {
                "push"
            }
            .to_string(),
        ),
        (
            "head_branch",
            if candidate {
                format!("release/{tag_name}")
            } else {
                tag_name.to_string()
            },
        ),
        ("head_sha", tag_sha.to_string()),
        ("id", release_run_id.to_string()),
        ("name", "Release".to_string()),
        ("path", ".github/workflows/release.yml".to_string()),
        ("repository", repository.to_string()),
        ("status", "completed".to_string()),
    ]);
    let actual = BTreeMap::from([
        ("conclusion", value_string(run.get("conclusion"))),
        ("event", value_string(run.get("event"))),
        ("head_branch", value_string(run.get("head_branch"))),
        ("head_sha", value_string(run.get("head_sha"))),
        ("id", value_string(run.get("id"))),
        ("name", value_string(run.get("name"))),
        ("path", value_string(run.get("path"))),
        (
            "repository",
            value_string(
                run.get("head_repository")
                    .and_then(|repo| repo.get("full_name")),
            ),
        ),
        ("status", value_string(run.get("status"))),
    ]);
    let mismatches = expected
        .iter()
        .filter_map(|(key, expected)| {
            let actual = actual.get(key).cloned().unwrap_or_default();
            if &actual == expected {
                None
            } else {
                Some(format!("{key}={actual}"))
            }
        })
        .collect::<Vec<_>>();
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Release run {release_run_id} does not match the failed exact-tag release contract: {}",
            mismatches.join(", ")
        ))
    }
}

pub fn validate_release_jobs(jobs: &[Value]) -> Result<(), String> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for job in jobs {
        let name = value_string(job.get("name"));
        *counts.entry(name.clone()).or_default() += 1;
        if job.get("status").and_then(Value::as_str) != Some("completed") {
            return Err(format!(
                "Every Release job must be terminal; {name} is {}",
                value_string(job.get("status"))
            ));
        }
    }
    let duplicates = counts
        .iter()
        .filter(|(_, count)| **count != 1)
        .map(|(name, count)| format!("{name}={count}"))
        .collect::<Vec<_>>();
    if !duplicates.is_empty() {
        return Err(format!(
            "Release job names must be unique: {}",
            duplicates.join(", ")
        ));
    }

    let candidate = jobs.iter().any(|job| {
        matches!(
            job.get("name").and_then(Value::as_str),
            Some("Release candidate ready" | "Authorize release candidate")
        )
    });
    let required = if candidate {
        &[
            "Build release npm packages",
            "Smoke release npm package installs",
            "Authorize release candidate",
            "Release candidate ready",
            "candidate-preflight / Verify release safety contract",
            "candidate-preflight / Validate crates.io publish plan",
            "release-preflight / Wait for validated PR and tag promotion",
        ][..]
    } else {
        REQUIRED_SUCCESSFUL_RELEASE_JOBS
    };
    for name in required {
        validate_exact_job(jobs, name, "success")?;
    }
    for name in REQUIRED_FAILED_RELEASE_JOBS {
        validate_exact_job(jobs, name, "failure")?;
    }
    for name in REQUIRED_SKIPPED_RELEASE_JOBS {
        validate_exact_job(jobs, name, "skipped")?;
    }

    let allowed_non_success = REQUIRED_FAILED_RELEASE_JOBS
        .iter()
        .map(|name| (*name, "failure"))
        .chain(
            REQUIRED_SKIPPED_RELEASE_JOBS
                .iter()
                .map(|name| (*name, "skipped")),
        )
        .collect::<BTreeMap<_, _>>();
    for job in jobs {
        let name = value_string(job.get("name"));
        if candidate && name == "Release crates.io handoff crates" {
            validate_exact_job(jobs, &name, "skipped")?;
            continue;
        }
        let expected = allowed_non_success
            .get(name.as_str())
            .copied()
            .unwrap_or("success");
        let conclusion = value_string(job.get("conclusion"));
        if conclusion != expected {
            return Err(format!(
                "Unexpected Release job conclusion: {name}={conclusion}, expected {expected}"
            ));
        }
    }
    Ok(())
}

pub fn validate_release_artifact(
    artifacts: &[Value],
    artifact_name: &str,
    release_run_id: &str,
    tag_name: &str,
    tag_sha: &str,
) -> Result<(), String> {
    let matches = artifacts
        .iter()
        .filter(|artifact| artifact.get("name").and_then(Value::as_str) == Some(artifact_name))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(format!(
            "Release run must contain exactly one {artifact_name} artifact, found {}",
            matches.len()
        ));
    }
    let artifact = matches[0];
    if artifact.get("expired").and_then(Value::as_bool) == Some(true) {
        return Err(format!("Release artifact {artifact_name} has expired"));
    }
    let source = artifact.get("workflow_run");
    if value_string(source.and_then(|source| source.get("id"))) != release_run_id
        || !source
            .and_then(|source| source.get("head_branch"))
            .and_then(Value::as_str)
            .is_some_and(|branch| branch == tag_name || branch == format!("release/{tag_name}"))
        || source
            .and_then(|source| source.get("head_sha"))
            .and_then(Value::as_str)
            != Some(tag_sha)
    {
        return Err(format!(
            "Release artifact {artifact_name} is not bound to {tag_name} ({tag_sha})"
        ));
    }
    Ok(())
}
