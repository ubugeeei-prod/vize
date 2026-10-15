use super::*;

#[test]
fn release_dispatch_requires_only_the_full_check() {
    let target = target();
    let plans =
        create_release_gate_dispatch_plans("release/v0.435.1", &target.sha, &target.base_sha)
            .unwrap();
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].workflow_name, "Check");
    assert_eq!(
        plans[0].expected_run_name,
        format!("Check full @ {}", target.sha)
    );
    let runs = green_runs(&target, false);
    let selected = select_required_workflow_runs(
        &runs,
        &target.sha,
        &release_evidence_shas(&target, true),
        &plans,
        None,
    )
    .unwrap();
    assert_eq!(selected["Check"]["head_sha"], target.sha);
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

fn green_runs(target: &ReleaseTarget, parent_corpus: bool) -> Vec<Value> {
    create_release_gate_dispatch_plans("release/v0.435.1", &target.sha, &target.base_sha)
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, plan)| {
            let sha = if parent_corpus
                && PARENT_EVIDENCE_REUSABLE_WORKFLOWS.contains(&plan.workflow_name)
            {
                &target.base_sha
            } else {
                &target.sha
            };
            let prefix = plan.expected_run_name.rsplit_once(" @ ").unwrap().0;
            json!({
                "id": index + 1,
                "path": format!(".github/workflows/{}", plan.workflow_id),
                "display_title": format!("{prefix} @ {sha}"),
                "head_sha": sha,
                "head_branch": "release/v0.435.1",
                "event": "workflow_dispatch",
                "status": "completed",
                "conclusion": "success",
                "created_at": "2026-10-08T00:00:00Z"
            })
        })
        .collect()
}

#[test]
fn pinned_cut_selects_check_at_the_release_head() {
    let target = target();
    let plans =
        create_release_gate_dispatch_plans("release/v0.435.1", &target.sha, &target.base_sha)
            .unwrap();
    let runs = green_runs(&target, false);
    let selected = select_required_workflow_runs(
        &runs,
        &target.sha,
        &release_evidence_shas(&target, true),
        &plans,
        None,
    )
    .unwrap();
    assert_eq!(selected.len(), REQUIRED_RELEASE_WORKFLOWS.len());
    assert_eq!(selected["Check"]["head_sha"], target.sha);
}

#[test]
fn newer_red_check_fails_the_release_gate() {
    let target = target();
    let plans =
        create_release_gate_dispatch_plans("release/v0.435.1", &target.sha, &target.base_sha)
            .unwrap();
    let mut runs = green_runs(&target, false);
    let mut failed = runs
        .iter()
        .find(|run| run["path"] == ".github/workflows/check.yml")
        .unwrap()
        .clone();
    failed["conclusion"] = json!("failure");
    failed["created_at"] = json!("2026-10-08T00:01:00Z");
    failed["id"] = json!(999);
    runs.push(failed);
    let error = select_required_workflow_runs(
        &runs,
        &target.sha,
        &release_evidence_shas(&target, true),
        &plans,
        None,
    )
    .unwrap_err();
    assert!(
        error.split("Check: completed/failure").nth(1).is_some(),
        "{error}"
    );
}
