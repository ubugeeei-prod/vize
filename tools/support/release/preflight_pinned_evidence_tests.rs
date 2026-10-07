use super::*;

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
fn pinned_cut_rejects_parent_only_fuzz_and_matrix_while_legacy_keeps_its_contract() {
    let target = target();
    let plans =
        create_release_gate_dispatch_plans("release/v0.435.1", &target.sha, &target.base_sha)
            .unwrap();
    let runs = green_runs(&target, true);
    assert!(
        select_required_workflow_runs(
            &runs,
            &target.sha,
            &release_evidence_shas(&target, false),
            &plans,
            None
        )
        .is_ok()
    );
    let error = select_required_workflow_runs(
        &runs,
        &target.sha,
        &release_evidence_shas(&target, true),
        &plans,
        None,
    )
    .unwrap_err();
    assert!(error.contains("Fuzz: missing"), "{error}");
    assert!(error.contains("Real Project Matrix: missing"), "{error}");
}

#[test]
fn pinned_cut_requires_and_selects_all_five_original_head_runs() {
    let target = target();
    let plans =
        create_release_gate_dispatch_plans("release/v0.435.1", &target.sha, &target.base_sha)
            .unwrap();
    let mut runs = green_runs(&target, true);
    runs.extend(green_runs(&target, false));
    let selected = select_required_workflow_runs(
        &runs,
        &target.sha,
        &release_evidence_shas(&target, true),
        &plans,
        None,
    )
    .unwrap();
    assert_eq!(selected.len(), REQUIRED_RELEASE_WORKFLOWS.len());
    for name in REQUIRED_RELEASE_WORKFLOWS {
        assert_eq!(selected[*name]["head_sha"], target.sha);
    }
}

#[test]
fn newer_red_original_head_fuzz_cannot_borrow_parent_success() {
    let target = target();
    let plans =
        create_release_gate_dispatch_plans("release/v0.435.1", &target.sha, &target.base_sha)
            .unwrap();
    let mut runs = green_runs(&target, false);
    runs.extend(green_runs(&target, true));
    let mut failed = runs
        .iter()
        .find(|run| run["path"] == ".github/workflows/fuzz.yml" && run["head_sha"] == target.sha)
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
    assert!(error.contains("Fuzz: completed/failure"), "{error}");
}
