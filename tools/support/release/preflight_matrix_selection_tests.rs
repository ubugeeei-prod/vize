use super::matrix_selection::{MatrixArtifactSelection, select_current_matrix_artifacts};
use serde_json::{Value, json};

struct Retry {
    run: Value,
    previous: Vec<Value>,
    current: Vec<Value>,
    artifacts: Vec<Value>,
}

fn retry() -> Retry {
    let run = json!({
        "id": 900, "run_attempt": 2, "head_sha": "a".repeat(40),
        "head_branch": "release/v0.435.1", "path": ".github/workflows/real-project-matrix.yml",
        "status": "completed", "conclusion": "success",
        "created_at": "2026-10-08T00:00:00Z", "run_started_at": "2026-10-08T00:10:00Z",
        "updated_at": "2026-10-08T00:20:00Z"
    });
    let previous = (0..22).map(|shard| json!({
        "id": 1_000 + shard, "name": format!("real projects ({shard}/22)"),
        "run_id": 900, "run_attempt": 1, "head_sha": "a".repeat(40), "status": "completed",
        "conclusion": if shard == 19 { "failure" } else { "success" },
        "started_at": "2026-10-08T00:00:01Z",
        "completed_at": if shard == 19 { "2026-10-08T00:00:20Z" } else { "2026-10-08T00:00:10Z" }
    })).collect::<Vec<_>>();
    let current = previous
        .iter()
        .enumerate()
        .map(|(shard, job)| {
            let mut job = job.clone();
            job["id"] = json!(2_000 + shard);
            job["run_attempt"] = json!(2);
            job["conclusion"] = json!("success");
            if shard == 19 {
                job["started_at"] = json!("2026-10-08T00:10:01Z");
                job["completed_at"] = json!("2026-10-08T00:10:20Z");
            }
            job
        })
        .collect();
    let mut artifacts = (0..22).map(|shard| json!({
        "id": if shard == 19 { 9_999 } else { 3_000 + shard },
        "name": format!("real-project-matrix-{shard}"), "expired": false,
        "created_at": if shard == 19 { "2026-10-08T00:00:15Z" } else { "2026-10-08T00:00:05Z" },
        "workflow_run": { "id": 900, "head_sha": "a".repeat(40), "head_branch": "release/v0.435.1" }
    })).collect::<Vec<_>>();
    let mut successful = artifacts[19].clone();
    successful["id"] = json!(3_019);
    successful["created_at"] = json!("2026-10-08T00:10:10Z");
    artifacts.push(successful);
    Retry {
        run,
        previous,
        current,
        artifacts,
    }
}

fn select(fixture: &Retry) -> Result<MatrixArtifactSelection, String> {
    select_current_matrix_artifacts(
        &fixture.run,
        &fixture.artifacts,
        &fixture.current,
        |attempt| {
            assert_eq!(attempt, 1);
            Ok(fixture.previous.clone())
        },
    )
}

fn reject(fixture: &Retry, message: &str) {
    let error = select(fixture)
        .err()
        .expect("malformed evidence must fail closed");
    assert!(error.contains(message), "{error}");
}

#[test]
fn matrix_retry_selects_successful_lower_id_and_records_original_carried_execution() {
    let fixture = retry();
    let original = fixture.artifacts.clone();
    let selection = select(&fixture).unwrap();
    assert_eq!(selection.artifacts.len(), 22);
    assert_eq!(selection.artifacts[19]["id"], 3_019);
    let fresh = &selection.provenance[19];
    assert_eq!(fresh["producingJob"]["id"], 2_019);
    assert_eq!(fresh["carriedForward"], false);
    assert_eq!(
        fresh["historicalArtifacts"],
        json!([fixture.artifacts[19], fixture.artifacts[22]])
    );
    let carried = &selection.provenance[0];
    assert_eq!(carried["producingJob"]["id"], 1_000);
    assert_eq!(carried["producingJob"]["run_attempt"], 1);
    assert_eq!(carried["observedJob"]["id"], 2_000);
    assert_eq!(carried["carriedForward"], true);
    assert_eq!(fixture.artifacts, original);
}

#[test]
fn matrix_retry_never_reuses_failed_archive_or_accepts_ambiguous_current_archives() {
    let mut missing = retry();
    missing.artifacts.pop();
    reject(&missing, "successful current execution; found 0");
    let mut duplicate = retry();
    let mut extra = duplicate.artifacts[22].clone();
    extra["id"] = json!(3_020);
    duplicate.artifacts.push(extra);
    reject(&duplicate, "successful current execution; found 2");
    let mut unknown = retry();
    unknown.artifacts[22]["created_at"] = json!("2026-10-08T00:05:00Z");
    reject(&unknown, "unknown or ambiguous producing execution");
}

#[test]
fn matrix_retry_rejects_foreign_artifact_and_job_source_run_and_attempt() {
    for (field, value) in [
        ("head_sha", json!("b".repeat(40))),
        ("id", json!(901)),
        ("head_branch", json!("foreign")),
    ] {
        let mut fixture = retry();
        fixture.artifacts[22]["workflow_run"][field] = value;
        reject(&fixture, "artifact is not bound");
    }
    for (field, value) in [
        ("head_sha", json!("b".repeat(40))),
        ("run_id", json!(901)),
        ("run_attempt", json!(1)),
    ] {
        let mut fixture = retry();
        fixture.current[19][field] = value;
        reject(&fixture, "job is not bound");
    }
}

#[test]
fn matrix_retry_requires_unambiguous_original_execution_for_carried_aliases() {
    let mut absent = retry();
    absent.previous.clear();
    reject(&absent, "carried-forward execution");
    let mut ambiguous = retry();
    let mut duplicate = ambiguous.previous[0].clone();
    duplicate["id"] = json!(4_000);
    ambiguous.previous.push(duplicate);
    reject(&ambiguous, "ambiguous historical job identities");
    let mut current = retry();
    current.current.push(current.current[0].clone());
    reject(&current, "exactly one current job; found 2");
}

#[test]
fn matrix_retry_rejects_missing_stamps_failed_current_jobs_and_invalid_times() {
    let mut missing = retry();
    missing.run.as_object_mut().unwrap().remove("run_attempt");
    reject(&missing, "positive run_attempt");
    let mut failed = retry();
    failed.current[19]["conclusion"] = json!("failure");
    reject(&failed, "current producing job is not successful");
    for invalid in ["2026-10-08T00:10:60Z", "2026-02-30T00:00:00Z", "invalid"] {
        let mut fixture = retry();
        fixture.current[19]["started_at"] = json!(invalid);
        reject(&fixture, "invalid started_at timestamp");
    }
    let mut outside = retry();
    outside.current[19]["completed_at"] = json!("2026-10-08T00:21:00Z");
    reject(&outside, "inconsistent execution timestamps");
    let mut expired = retry();
    expired.artifacts[22]["expired"] = json!(true);
    reject(&expired, "expired");
}
