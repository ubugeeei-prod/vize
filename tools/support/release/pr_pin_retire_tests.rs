use super::super::pr_github as github;
use super::{retire, retire_archive as archive, retire_evidence as evidence, tests::Repo};
use serde_json::{Value, json};
use std::collections::BTreeMap;
#[path = "pr_pin_retire_fixtures.rs"]
mod fixtures;

pub(super) fn fixture(repo: &Repo) -> (Value, String, String, String) {
    fixtures::fixture(repo)
}

#[test]
fn unpublished_retirement_archive_keeps_real_original_refs_parents_and_exact_failure_bytes() {
    let repo = Repo::new();
    let (receipt, head, pin, integration) = fixture(&repo);
    let before = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    let mut logs = BTreeMap::new();
    logs.insert(
        "failure/123.log".into(),
        b"\xef\xbb\xbf2026-10-09T00:00:00Z error: dead_code\r\n\n".to_vec(),
    );
    let archived = archive::install(receipt.clone(), &logs, &repo.work).unwrap();
    let (_, saved) = archive::read("v0.8.0", &repo.work).unwrap().unwrap();
    assert_eq!(
        archive::identity(&saved).unwrap(),
        archive::identity(&receipt).unwrap()
    );
    assert_eq!(saved["registryAbsence"], receipt["registryAbsence"]);
    assert_eq!(
        saved["registryAbsence"]["observations"][1]["response"].as_str(),
        Some("version not found: 0.8.0")
    );
    assert_eq!(
        github::git(&["rev-list", "--parents", "-n", "1", &archived], &repo.work).unwrap(),
        format!("{archived} {pin} {integration}")
    );
    assert_eq!(
        super::metadata::bytes(
            &["show", &format!("{archived}:failure/123.log")],
            &repo.work
        )
        .unwrap(),
        logs["failure/123.log"]
    );
    let after = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    for row in before.lines() {
        assert!(after.lines().any(|actual| actual == row));
    }
    assert_eq!(github::tag_target("v0.8.0", &repo.work).unwrap(), None);
    assert_eq!(
        archive::install(receipt.clone(), &logs, &repo.work).unwrap(),
        archived
    );
    let mut foreign = receipt;
    foreign["identity"]["head"] = json!(integration);
    assert!(archive::install(foreign, &logs, &repo.work).is_err());
    assert_eq!(
        archive::remote(&archive::reference("v0.8.0"), &repo.work).unwrap(),
        Some(archived)
    );
    assert!(retire::reject_resume("v0.8.0", &repo.work).is_err());
    assert!(retire::reject_resume("v0.9.0", &repo.work).is_ok());
    assert_eq!(
        archive::remote("refs/heads/release/v0.8.0", &repo.work).unwrap(),
        Some(head)
    );
}

#[test]
fn retirement_archive_ref_collision_unsafe_log_paths_and_bounds_fail_without_original_ref_mutation()
{
    let repo = Repo::new();
    let (receipt, head, _, _) = fixture(&repo);
    let before = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    for logs in [
        BTreeMap::from([("failure/../escape.log".into(), b"failure".to_vec())]),
        BTreeMap::from([(
            "failure/123.log".into(),
            vec![b'x'; archive::MAX_LOG_BYTES + 1],
        )]),
    ] {
        assert!(archive::install(receipt.clone(), &logs, &repo.work).is_err());
        assert_eq!(
            github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
            before
        );
    }
    github::git(
        &[
            "push",
            "origin",
            &format!("{head}:{}", archive::reference("v0.8.0")),
        ],
        &repo.work,
    )
    .unwrap();
    assert!(archive::read("v0.8.0", &repo.work).is_err());
    assert!(archive::install(receipt, &BTreeMap::new(), &repo.work).is_err());
    assert_eq!(
        archive::remote("refs/heads/release/v0.8.0", &repo.work).unwrap(),
        Some(head)
    );
}

#[test]
fn retirement_requires_original_terminal_failure_and_source_bound_unstarted_publication() {
    for (status, conclusion, valid) in [
        ("completed", "failure", true),
        ("completed", "cancelled", true),
        ("completed", "timed_out", true),
        ("completed", "success", false),
        ("in_progress", "failure", false),
        ("queued", "cancelled", false),
        ("completed", "neutral", false),
    ] {
        assert_eq!(
            evidence::terminal_failed(
                &json!({"status":status,"conclusion":conclusion,"run_attempt":1})
            )
            .is_ok(),
            valid
        );
    }
    assert!(
        evidence::terminal_failed(
            &json!({"status":"completed","conclusion":"failure","run_attempt":0})
        )
        .is_err()
    );
    let workflow = "jobs:\n  release-npm-law:\n    name: Release law to npm\n  create-github-release:\n    name: Create GitHub Release\n";
    let jobs = vec![
        json!({"name":"Release law to npm","run_attempt":1,"status":"completed","conclusion":"cancelled","steps":[]}),
        json!({"name":"Create GitHub Release","run_attempt":1,"status":"completed","conclusion":"skipped","steps":[]}),
    ];
    assert!(evidence::publication_jobs(&jobs, workflow).is_ok());
    for (pointer, value) in [
        ("/status", json!("in_progress")),
        ("/conclusion", json!("success")),
        ("/steps", json!([{"conclusion":"cancelled"}])),
        ("/name", json!("foreign")),
        ("/run_attempt", json!(0)),
    ] {
        let mut mutated = jobs.clone();
        *mutated[0].pointer_mut(pointer).unwrap() = value;
        assert!(
            evidence::publication_jobs(&mutated, workflow).is_err(),
            "{pointer}"
        );
    }
    assert!(evidence::publication_jobs(&jobs[..1], workflow).is_err());
    let mut duplicate = jobs.clone();
    duplicate.push(jobs[0].clone());
    assert!(evidence::publication_jobs(&duplicate, workflow).is_err());
}

#[test]
fn retirement_ledger_refuses_incomplete_views_and_actual_failure_blob_mismatches() {
    let repo = Repo::new();
    let (mut receipt, head, _, _) = fixture(&repo);
    let bytes = b"actual private fixture failed build\n".to_vec();
    let logs = BTreeMap::from([("failure/123.log".into(), bytes.clone())]);
    std::fs::write(repo.work.join("failure.txt"), &bytes).unwrap();
    let oid = github::git(&["hash-object", "-w", "failure.txt"], &repo.work).unwrap();
    receipt["failureLogObjects"] = json!({"failure/123.log":{"oid":oid,"bytes":bytes.len()}});
    super::retire_ledger::validate(&receipt, &repo.work).unwrap();
    let before = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    for (pointer, value) in fixtures::mutations(&head, bytes.len()) {
        let mut bad = receipt.clone();
        *bad.pointer_mut(&pointer).unwrap() = value;
        assert!(
            super::retire_ledger::validate(&bad, &repo.work).is_err(),
            "{pointer}"
        );
        if !pointer.starts_with("/failureLogObjects") {
            assert!(
                archive::install(bad, &logs, &repo.work).is_err(),
                "{pointer}"
            );
        }
        assert_eq!(
            github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
            before
        );
    }
    for index in [0, 2] {
        let mut bad = receipt.clone();
        bad["originalEvidence"]["observations"]
            .as_array_mut()
            .unwrap()
            .remove(index);
        assert!(archive::install(bad, &logs, &repo.work).is_err());
        assert_eq!(
            github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
            before
        );
    }
    for (field, value) in [
        ("path", json!(".github/workflows/foreign.yml")),
        ("head_branch", json!("main")),
        ("display_title", json!("foreign title")),
        ("event", json!("push")),
        ("run_attempt", json!(2)),
        ("status", json!("in_progress")),
        ("conclusion", json!("success")),
    ] {
        let mut bad = receipt.clone();
        bad["originalEvidence"]["HRunPages"][0]["workflow_runs"][0][field] = value.clone();
        bad["originalEvidence"]["observations"][0]["run"][field] = value.clone();
        bad["guards"]["releaseRun"][field] = value;
        assert!(
            archive::install(bad, &logs, &repo.work).is_err(),
            "R {field}"
        );
    }
    for (field, value) in [
        ("path", json!(".github/workflows/foreign.yml")),
        ("head_sha", json!(head)),
        ("head_branch", json!("release/v0.8.0")),
        ("event", json!("push")),
        ("run_attempt", json!(2)),
        ("actor", json!({"login":"foreign"})),
        ("triggering_actor", json!({"login":"foreign"})),
    ] {
        let mut bad = receipt.clone();
        bad["originalEvidence"]["observations"][2]["run"][field] = value.clone();
        bad["guards"]["operatorRun"][field] = value;
        assert!(
            archive::install(bad, &logs, &repo.work).is_err(),
            "operator {field}"
        );
    }
    for field in ["actor", "triggering_actor"] {
        let mut bad = receipt.clone();
        for run in [&mut bad["originalEvidence"]["HRunPages"][0]["workflow_runs"][0]] {
            run[field]["login"] = json!("foreign");
        }
        bad["originalEvidence"]["observations"][0]["run"][field]["login"] = json!("foreign");
        bad["guards"]["releaseRun"][field]["login"] = json!("foreign");
        assert!(archive::install(bad, &logs, &repo.work).is_err(), "{field}");
    }
    assert!(archive::install(receipt, &BTreeMap::new(), &repo.work).is_err());
    assert_eq!(
        github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
        before
    );
}

#[test]
fn retirement_archived_guards_bind_source_markers_operator_inventory_and_all_releases() {
    let repo = Repo::new();
    let (receipt, _, _, _) = fixture(&repo);
    let logs = BTreeMap::from([(
        "failure/123.log".into(),
        b"actual private fixture failure\n".to_vec(),
    )]);
    fixtures::guard_controls(&receipt, &logs, &repo);
}

#[test]
fn retirement_read_refuses_bare_valid_identity_archive_with_empty_observations() {
    let repo = Repo::new();
    let (mut receipt, head, pin, integration) = fixture(&repo);
    let before = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    receipt["originalEvidence"]["observations"] = json!([]);
    receipt["failureLogObjects"] = json!({});
    std::fs::write(
        repo.work.join("retirement.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    let oid = github::git(&["hash-object", "-w", "retirement.json"], &repo.work).unwrap();
    github::git(&["read-tree", "--empty"], &repo.work).unwrap();
    github::git(
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "100644",
            &oid,
            "retirement.json",
        ],
        &repo.work,
    )
    .unwrap();
    let tree = github::git(&["write-tree"], &repo.work).unwrap();
    let archived = github::git(
        &[
            "commit-tree",
            &tree,
            "-p",
            &pin,
            "-p",
            &integration,
            "-m",
            "incomplete inert ledger",
        ],
        &repo.work,
    )
    .unwrap();
    let reference = archive::reference("v0.8.0");
    github::git(
        &["push", "origin", &format!("{archived}:{reference}")],
        &repo.work,
    )
    .unwrap();
    assert!(archive::read("v0.8.0", &repo.work).is_err());
    assert!(retire::next_minor("owner/repo", "0.7.0", None, &repo.work).is_err());
    assert!(archive::install(receipt, &BTreeMap::new(), &repo.work).is_err());
    assert_eq!(
        archive::remote(&reference, &repo.work).unwrap(),
        Some(archived)
    );
    let after = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    assert!(
        before
            .lines()
            .all(|row| after.lines().any(|actual| actual == row))
    );
    assert_eq!(
        archive::remote("refs/heads/release/v0.8.0", &repo.work).unwrap(),
        Some(head)
    );
}

#[test]
fn reservation_minors_are_canonical_bounded_and_never_silent_patch_or_arbitrary_targets() {
    let repo = Repo::new();
    assert_eq!(
        retire::next_minor("owner/repo", "0.437.1", None, &repo.work).unwrap(),
        "0.438.0"
    );
    assert_eq!(
        retire::next_minor("owner/repo", "1.2.3", None, &repo.work).unwrap(),
        "1.3.0"
    );
    assert_eq!(retire::ordinary_minor("0.437.0").unwrap(), "0.438.0");
    assert_eq!(retire::ordinary_minor("0.438.0").unwrap(), "0.439.0");
    assert_eq!(retire::ordinary_minor("0.437.1").unwrap(), "0.438.0");
    assert_eq!(retire::ordinary_minor("1.2.3").unwrap(), "1.3.0");
    assert_eq!(
        retire::ordinary_minor("0.437.1-alpha.1").unwrap(),
        "0.438.0"
    );
    for version in ["0.0437.0", "0.-1.0", "0.18446744073709551615.0"] {
        assert!(retire::ordinary_minor(version).is_err(), "{version}");
    }
}
