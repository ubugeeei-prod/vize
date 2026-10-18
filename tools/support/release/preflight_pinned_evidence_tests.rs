use super::*;

#[test]
fn release_dispatch_requires_exact_full_check() {
    let target = target();
    let plans =
        create_release_gate_dispatch_plans("release/v0.435.1", &target.sha, &target.base_sha)
            .unwrap();
    assert_eq!(REQUIRED_RELEASE_WORKFLOWS, &["Check"]);
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].workflow_id, "check.yml");
    assert_eq!(plans[0].inputs, json!({ "release-qualification": true }));
    assert_eq!(
        plans[0].expected_run_name,
        format!("Check full @ {}", target.sha)
    );
    let selected = select_required_workflow_runs(
        &[],
        &target.sha,
        &release_evidence_shas(&target, true),
        &plans,
        None,
    )
    .unwrap_err();
    assert!(selected.contains("Check: missing"));
    assert!(create_release_gate_dispatch_plans("", &target.sha, &target.base_sha).is_err());
    assert!(create_release_gate_dispatch_plans("release/test", "HEAD", &target.base_sha).is_err());
    assert!(create_release_gate_dispatch_plans("release/test", &target.sha, &target.sha).is_err());
}

fn target() -> ReleaseTarget {
    ReleaseTarget {
        tag: "v0.435.1".into(),
        sha: "a".repeat(40),
        version: "0.435.1".into(),
        base_sha: "b".repeat(40),
        version_only: true,
    }
}

fn explicit_check_plan(target: &ReleaseTarget) -> DispatchPlan {
    DispatchPlan {
        workflow_name: "Check",
        workflow_id: "check.yml",
        ref_name: "release/v0.435.1".into(),
        inputs: json!({ "release-qualification": true }),
        expected_run_name: format!("Check full @ {}", target.sha),
        accepts_scheduled_evidence: false,
    }
}

fn green_run(target: &ReleaseTarget, plan: &DispatchPlan) -> Value {
    json!({
        "id": 1,
        "path": format!(".github/workflows/{}", plan.workflow_id),
        "display_title": plan.expected_run_name,
        "head_sha": target.sha,
        "head_branch": plan.ref_name,
        "event": "workflow_dispatch",
        "status": "completed",
        "conclusion": "success",
        "created_at": "2026-10-08T00:00:00Z"
    })
}

#[test]
fn explicit_check_inspection_selects_only_the_pinned_source_and_workflow() {
    let target = target();
    let plan = explicit_check_plan(&target);
    let mut runs = vec![green_run(&target, &plan)];
    for (field, value) in [
        ("head_sha", target.base_sha.as_str()),
        ("path", ".github/workflows/other.yml"),
        ("event", "pull_request"),
        ("display_title", "Check fast"),
        ("head_branch", "main"),
    ] {
        let mut wrong = runs[0].clone();
        wrong[field] = json!(value);
        wrong["id"] = json!(999);
        wrong["created_at"] = json!("2026-10-08T00:01:00Z");
        runs.push(wrong);
    }
    let accepted =
        accepted_evidence_shas(&release_evidence_shas(&target, true), "Check", &target.sha);
    let selected = latest_required_workflow_run(&runs, &accepted, "Check", Some(&plan))
        .unwrap()
        .unwrap();
    assert_eq!(selected["id"], 1);
    assert_eq!(selected["head_sha"], target.sha);
}

#[test]
fn explicit_check_inspection_rejects_newer_red_job_evidence() {
    let target = target();
    let plan = explicit_check_plan(&target);
    let mut runs = vec![green_run(&target, &plan)];
    let mut failed = runs[0].clone();
    failed["conclusion"] = json!("failure");
    failed["created_at"] = json!("2026-10-08T00:01:00Z");
    failed["id"] = json!(999);
    runs.push(failed);
    let current = latest_required_workflow_run(&runs, &[target.sha], "Check", Some(&plan))
        .unwrap()
        .unwrap();
    assert_eq!(
        current["id"], 999,
        "older green must not hide newer red evidence"
    );
    let jobs = vec![json!({
        "name": "test-scripts", "status": current["status"], "conclusion": current["conclusion"]
    })];
    assert!(required_jobs_contain_terminal_failure(
        "Check", &jobs, current
    ));
    let error = assert_required_workflow_jobs("Check", &jobs, current).unwrap_err();
    assert!(
        error.contains("test-scripts is completed/failure"),
        "{error}"
    );
}

#[test]
fn full_check_jobs_fail_closed_for_every_incomplete_result() {
    let run = json!({"event": "workflow_dispatch", "head_branch": "main"});
    let names = [
        "test-scripts",
        "test-js-packages",
        "build-js-packages",
        "full-js-report",
        "Full Rust workspace and doctests",
        "clippy-and-test",
        "full-rust-report",
    ];
    let jobs = names
        .iter()
        .map(|name| json!({"name": name, "status": "completed", "conclusion": "success"}))
        .collect::<Vec<_>>();
    assert!(assert_required_workflow_jobs("Check", &jobs, &run).is_ok());
    for (index, name) in names.iter().enumerate() {
        for variant in [
            "missing",
            "duplicate",
            "skipped",
            "failure",
            "cancelled",
            "in_progress",
        ] {
            let mut incomplete = jobs.clone();
            match variant {
                "missing" => {
                    incomplete.remove(index);
                }
                "duplicate" => incomplete.push(jobs[index].clone()),
                "in_progress" => incomplete[index]["status"] = json!(variant),
                _ => incomplete[index]["conclusion"] = json!(variant),
            }
            let error = assert_required_workflow_jobs("Check", &incomplete, &run).unwrap_err();
            assert!(error.contains(name), "{variant}: {error}");
        }
    }
    for branch in ["release/v0.458.0", "v0.458.0"] {
        let release = json!({"event": "workflow_dispatch", "head_branch": branch});
        let error = assert_required_workflow_jobs("Check", &jobs, &release).unwrap_err();
        assert!(
            error.contains("cargo-semver-checks (vize_armature)"),
            "{error}"
        );
        let mut cut_jobs = jobs.clone();
        cut_jobs.extend(REQUIRED_SEMVER_CRATES.iter().map(|name| json!({"name": format!("cargo-semver-checks ({name})"), "status": "completed", "conclusion": "success"})));
        assert!(assert_required_workflow_jobs("Check", &cut_jobs, &release).is_ok());
        for index in jobs.len()..cut_jobs.len() {
            let mut incomplete = cut_jobs.clone();
            incomplete[index]["conclusion"] = json!("skipped");
            assert!(assert_required_workflow_jobs("Check", &incomplete, &release).is_err());
        }
    }
}
