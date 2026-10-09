use super::super::pr_github as github;
use super::{
    lock, retire_archive as archive, retire_evidence as evidence, retire_tests::fixture,
    tests::Repo,
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
