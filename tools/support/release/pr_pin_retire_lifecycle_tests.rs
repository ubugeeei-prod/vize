use super::super::pr_github as github;
use super::{
    lock, metadata, retire, retire_archive as archive, retire_evidence as evidence,
    retire_log_storage as storage, retire_prequalification_archive_tests::prequalification_fixture,
    retire_source, retire_tests::fixture, tests::Repo,
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn retirement_archive_mutation_rechecks_deadline_and_exact_operator_lease() {
    let repo = Repo::new();
    let (receipt, head, _, _) = fixture(&repo);
    let logs = BTreeMap::from([(
        "failure/123.log".into(),
        b"complete original failure\n".to_vec(),
    )]);
    let before = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    let budget = super::super::pr_budget::Budget::new(std::time::Duration::ZERO);
    let refused =
        archive::install_authorized(receipt.clone(), &logs, &repo.work, &|| budget.check())
            .unwrap_err();
    assert!(refused.contains("budget expired"));
    assert_eq!(
        github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
        before
    );
    let operator = lock::acquire("v0.8.0", &head, &repo.work).unwrap();
    let foreign = github::git(
        &[
            "commit-tree",
            &github::git(&["rev-parse", &format!("{head}^{{tree}}")], &repo.work).unwrap(),
            "-p",
            &head,
            "-m",
            "foreign operator",
        ],
        &repo.work,
    )
    .unwrap();
    github::git(
        &[
            "push",
            "origin",
            &format!("{foreign}:refs/heads/foreign-operator-object"),
        ],
        &repo.work,
    )
    .unwrap();
    let refused = archive::install_authorized(receipt, &logs, &repo.work, &|| {
        github::git(
            &["update-ref", "refs/heads/release-operator/v0.8.0", &foreign],
            &repo.remote,
        )?;
        operator.verify()
    })
    .unwrap_err();
    assert!(refused.contains("lease changed"), "{refused}");
    assert!(
        archive::remote(&archive::reference("v0.8.0"), &repo.work)
            .unwrap()
            .is_none()
    );
    drop(operator);
    assert_eq!(
        archive::remote("refs/heads/release-operator/v0.8.0", &repo.work).unwrap(),
        Some(foreign)
    );
    let after = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    for row in before.lines() {
        assert!(after.lines().any(|actual| actual == row));
    }
}

#[test]
fn current_operator_exemption_binds_sha_workflow_and_both_authenticated_actors() {
    let repository = "owner/repo";
    let context = json!({"id":8,"sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","actor":"maintainer","workflowRef":"owner/repo/.github/workflows/release-operator.yml@refs/heads/main"});
    let run = json!({"id":8,"head_sha":context["sha"],"head_branch":"main","event":"workflow_dispatch","path":".github/workflows/release-operator.yml","head_repository":{"full_name":repository},"actor":{"login":"maintainer"},"triggering_actor":{"login":"maintainer"}});
    assert!(evidence::current_operator(&run, &context, repository).is_ok());
    for (pointer, value) in [
        ("/id", json!(9)),
        (
            "/head_sha",
            json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        ),
        ("/head_branch", json!("release/v0.8.0")),
        ("/event", json!("pull_request")),
        ("/path", json!(".github/workflows/check.yml")),
        ("/head_repository/full_name", json!("foreign/repo")),
        ("/actor/login", json!("other")),
        ("/triggering_actor/login", json!("other")),
    ] {
        let mut changed = run.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        assert!(
            evidence::current_operator(&changed, &context, repository).is_err(),
            "{pointer}"
        );
    }
    let mut foreign = context;
    foreign["workflowRef"] =
        json!("owner/repo/.github/workflows/release-operator.yml@refs/heads/foreign");
    assert!(evidence::current_operator(&run, &foreign, repository).is_err());
}

#[test]
fn retirement_refuses_draft_public_duplicate_or_incomplete_release_inventory() {
    let tag = "v0.8.0";
    let page = json!([{"id":10,"tag_name":"v0.7.0","draft":false,"prerelease":false,"published_at":"2026-10-10T00:00:00Z","url":"https://api.github.com/repos/owner/repo/releases/10"}]);
    assert!(
        super::retire_guard::release_page(&page, "owner/repo", tag, &mut Default::default())
            .unwrap()
    );
    for draft in [false, true] {
        let present = json!([{"id":11,"tag_name":tag,"draft":draft,"prerelease":false,"published_at":null,"url":"https://api.github.com/repos/owner/repo/releases/11"}]);
        assert!(
            super::retire_guard::release_page(&present, "owner/repo", tag, &mut Default::default())
                .is_err()
        );
    }
    for invalid in [
        json!(null),
        json!([{"id":10,"tag_name":"v0.7.0"}]),
        json!([{"id":0,"tag_name":"v0.7.0","draft":false}]),
    ] {
        assert!(
            super::retire_guard::release_page(&invalid, "owner/repo", tag, &mut Default::default())
                .is_err()
        );
    }
    let mut seen = Default::default();
    super::retire_guard::release_page(&page, "owner/repo", tag, &mut seen).unwrap();
    assert!(super::retire_guard::release_page(&page, "owner/repo", tag, &mut seen).is_err());
}

#[test]
fn expired_retirement_loops_return_naturally_and_drop_only_their_own_lease() {
    let repo = Repo::new();
    let (_, head, _, _) = fixture(&repo);
    let before = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    for inventory in [true, false] {
        let operator = lock::acquire("v0.8.0", &head, &repo.work).unwrap();
        let budget = super::super::pr_budget::Budget::new(std::time::Duration::ZERO);
        let result = if inventory {
            evidence::inventory(
                "owner/repo",
                "actions/runs",
                "workflow_runs",
                Some(&budget),
                &repo.work,
            )
            .map(|_| ())
        } else {
            super::retire::next_minor("owner/repo", "0.7.0", Some(&budget), &repo.work).map(|_| ())
        };
        assert!(result.unwrap_err().contains("budget expired"));
        drop(operator);
        assert_eq!(
            github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
            before
        );
        assert!(
            archive::remote(&archive::reference("v0.8.0"), &repo.work)
                .unwrap()
                .is_none()
        );
    }
}

fn sync_qualification_run(receipt: &mut serde_json::Value, run: serde_json::Value) {
    receipt["guards"]["qualificationRun"] = run.clone();
    receipt["guards"]["releaseRunAbsence"]["hRunPages"][0]["workflow_runs"][0] = run.clone();
    receipt["originalEvidence"]["HRunPages"][0]["workflow_runs"][0] = run.clone();
    receipt["originalEvidence"]["observations"][0]["run"] = run;
}

fn sync_qualification_jobs(receipt: &mut serde_json::Value, jobs: Vec<serde_json::Value>) {
    let pages = json!([{"total_count":jobs.len(),"jobs":jobs}]);
    receipt["guards"]["qualificationJobs"] = json!(jobs);
    receipt["guards"]["originalFailureJobPages"]["qualification"] = pages.clone();
    receipt["originalEvidence"]["observations"][0]["jobPages"] = pages;
}

#[test]
fn qualification_archive_replays_actual_source_policy_and_preserves_typed_failure_origin() {
    let repo = Repo::new();
    let (mut receipt, head, _, _) = prequalification_fixture(&repo);
    let raw = b"whole actual inert Q failure\xff\x00\r\n\n";
    receipt["originalEvidence"]["failureLogEncoding"] = json!("gzip");
    receipt["originalEvidence"]["failureLogSources"] = json!({
        "failure/123.log.gz":{"oid":storage::blob_identity(raw, &repo.work).unwrap(),"bytes":raw.len()}
    });
    let logs = BTreeMap::from([("failure/123.log.gz".into(), storage::gzip(raw).unwrap())]);
    let before = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    let source = retire_source::validate(&receipt, &repo.work).unwrap();
    assert_eq!(source.candidate.head, head);
    let request = retire::Request::from_identity(archive::identity(&receipt).unwrap()).unwrap();
    assert!(matches!(request.failure, retire::Failure::BeforeRelease(8)));
    assert_eq!(request.failure.key(), "qualificationRun");
    assert_eq!(request.failure.attempt_key(), "qualificationAttempt");
    assert_eq!(request.operator, 6);
    let legacy_repo = Repo::new();
    let (legacy, _, _, _) = fixture(&legacy_repo);
    let legacy_request =
        retire::Request::from_identity(archive::identity(&legacy).unwrap()).unwrap();
    assert!(matches!(
        legacy_request.failure,
        retire::Failure::Release(7)
    ));
    for (key, value) in [
        ("origin", json!("release")),
        ("origin", serde_json::Value::Null),
        ("qualificationRun", json!(0)),
        ("releaseRun", json!(0)),
        ("releaseRun", serde_json::Value::Null),
        ("releaseAttempt", json!(1)),
    ] {
        let mut fields = receipt["identity"].clone();
        fields[key] = value;
        assert!(retire::Request::from_identity(&fields).is_err(), "{key}");
        assert_eq!(
            github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
            before
        );
    }
    let mut mutations = Vec::new();
    for (status, conclusion) in [
        ("completed", json!("success")),
        ("completed", json!("cancelled")),
        ("in_progress", serde_json::Value::Null),
    ] {
        let mut bad = receipt.clone();
        let mut run = bad["guards"]["qualificationRun"].clone();
        run["status"] = json!(status);
        run["conclusion"] = conclusion;
        sync_qualification_run(&mut bad, run);
        mutations.push(bad);
    }
    let mut missing = receipt.clone();
    let mut jobs = missing["guards"]["qualificationJobs"]
        .as_array()
        .unwrap()
        .clone();
    jobs.pop();
    sync_qualification_jobs(&mut missing, jobs);
    mutations.push(missing);
    let mut historical = receipt.clone();
    historical["identity"]["qualificationAttempt"] = json!(2);
    historical["guards"]["identity"] = historical["identity"].clone();
    let mut run = historical["guards"]["qualificationRun"].clone();
    run["run_attempt"] = json!(2);
    sync_qualification_run(&mut historical, run);
    let mut jobs = historical["guards"]["qualificationJobs"]
        .as_array()
        .unwrap()
        .clone();
    let current = jobs
        .iter()
        .enumerate()
        .map(|(index, job)| {
            let mut job = job.clone();
            job["id"] = json!(5000 + index);
            job["run_attempt"] = json!(2);
            job["conclusion"] = json!("success");
            job
        })
        .collect::<Vec<_>>();
    jobs.extend(current);
    sync_qualification_jobs(&mut historical, jobs);
    let error = retire_source::validate(&historical, &repo.work).unwrap_err();
    assert!(error.contains("current Q attempt"), "{error}");
    mutations.push(historical);
    for pages in [
        json!([]),
        json!([{"total_count":1,"workflow_runs":[]}]),
        json!([{"total_count":0,"workflow_runs":[],"message":"Bad credentials"}]),
    ] {
        let mut bad = receipt.clone();
        bad["guards"]["releaseRunAbsence"]["pages"] = pages;
        mutations.push(bad);
    }
    let q = receipt["guards"]["qualificationRun"].clone();
    let mut duplicate = receipt.clone();
    duplicate["guards"]["releaseRunAbsence"]["hRunPages"] =
        json!([{"total_count":2,"workflow_runs":[q,q]}]);
    mutations.push(duplicate);
    let mut rows = vec![q.clone()];
    for id in 2000..2099 {
        let mut diagnostic = q.clone();
        diagnostic["id"] = json!(id);
        diagnostic["display_title"] = json!("unrelated diagnostic");
        rows.push(diagnostic);
    }
    let mut any_release = q;
    any_release["id"] = json!(9999);
    any_release["path"] = json!(".github/workflows/release.yml");
    any_release["event"] = json!("pull_request");
    any_release["display_title"] = json!("unrelated title");
    any_release["conclusion"] = json!("success");
    let mut page_two = receipt.clone();
    page_two["guards"]["releaseRunAbsence"]["hRunPages"] = json!([
        {"total_count":101,"workflow_runs":rows},{"total_count":101,"workflow_runs":[any_release]}
    ]);
    let error = retire_source::validate(&page_two, &repo.work).unwrap_err();
    assert!(error.contains("Release run forbids"), "{error}");
    mutations.push(page_two);
    for bad in mutations {
        assert!(retire_source::validate(&bad, &repo.work).is_err());
        assert!(archive::install(bad, &logs, &repo.work).is_err());
        assert_eq!(
            github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
            before
        );
    }
    let archived = archive::install(receipt, &logs, &repo.work).unwrap();
    let (_, saved) = archive::read("v0.8.0", &repo.work).unwrap().unwrap();
    assert_eq!(
        retire_source::validate(&saved, &repo.work)
            .unwrap()
            .candidate
            .head,
        head
    );
    let stored = metadata::bytes(
        &["show", &format!("{archived}:failure/123.log.gz")],
        &repo.work,
    )
    .unwrap();
    assert_eq!(storage::decode(&stored, raw.len()).unwrap(), raw);
    for row in before.lines() {
        assert!(
            github::git(&["ls-remote", "--refs", "origin"], &repo.work)
                .unwrap()
                .lines()
                .any(|actual| actual == row)
        );
    }
    assert!(retire::reject_resume("v0.8.0", &repo.work).is_err());
    assert_eq!(github::tag_target("v0.8.0", &repo.work).unwrap(), None);
}
