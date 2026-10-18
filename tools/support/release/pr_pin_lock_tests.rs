use super::super::tests::{Repo, source};
use super::*;
use serde_json::json;
use std::time::Duration;

#[test]
fn original_hosted_operator_identity_is_required_without_status_shortcuts() {
    let cut = "1111111111111111111111111111111111111111";
    let valid = json!({
        "id": 7, "run_attempt": 2, "head_sha": cut, "head_branch": "main",
        "event": "workflow_dispatch", "path": ".github/workflows/release-operator.yml",
        "status": "in_progress", "conclusion": null,
        "head_repository": {"full_name": "owner/repo"},
        "actor": {"login": "maintainer"}, "triggering_actor": {"login": "maintainer"}
    });
    let verify = |value: &serde_json::Value| {
        validate_operator_run(value, 7, 2, "owner/repo", cut, "maintainer")
    };
    assert!(verify(&valid).is_ok());
    for (pointer, replacement) in [
        ("/id", json!(8)),
        ("/run_attempt", json!(1)),
        (
            "/head_sha",
            json!("2222222222222222222222222222222222222222"),
        ),
        ("/head_branch", json!("release/v1.2.3")),
        ("/event", json!("push")),
        ("/path", json!(".github/workflows/check.yml")),
        ("/status", json!("completed")),
        ("/status", json!("queued")),
        ("/conclusion", json!("success")),
        ("/head_repository/full_name", json!("foreign/repo")),
        ("/actor/login", json!("other")),
        ("/triggering_actor/login", json!("other")),
    ] {
        let mut changed = valid.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        assert!(verify(&changed).is_err(), "{pointer}");
    }
    let mut missing = valid.clone();
    missing.as_object_mut().unwrap().remove("conclusion");
    assert!(verify(&missing).is_err());
    for value in [
        "0",
        "01",
        "+1",
        "-1",
        " 1",
        "1 ",
        "",
        "18446744073709551616",
    ] {
        assert!(positive(value).is_err(), "{value:?}");
    }
}

#[test]
fn versioned_same_ref_lease_captures_original_budget_without_source_or_pin_mutation() {
    let repo = Repo::new();
    let cut = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.2\"\n")]);
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    let head = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.3\"\n")]);
    github::git(
        &[
            "push",
            "origin",
            &format!("{head}:refs/heads/release/v1.2.3"),
        ],
        &repo.work,
    )
    .unwrap();
    let budget = pr_budget::Budget::new(Duration::from_secs(60));
    let (_, deadline) = budget.child_allowance().unwrap();
    let operator = acquire_with_admission(
        "v1.2.3",
        &head,
        Some(Admission {
            run: 7,
            attempt: 2,
            source: cut.clone(),
            deadline,
        }),
        &repo.work,
    )
    .unwrap();
    assert!(operator.allows_early_admission());
    operator.verify().unwrap();
    let lease = remote("refs/heads/release-operator/v1.2.3", &repo.work)
        .unwrap()
        .unwrap();
    assert_eq!(lease, operator.remote_head);
    assert_eq!(github::parent(&lease, &repo.work).unwrap(), head);
    assert_eq!(
        github::git(&["rev-parse", &format!("{lease}^{{tree}}")], &repo.work).unwrap(),
        github::git(&["rev-parse", &format!("{head}^{{tree}}")], &repo.work).unwrap()
    );
    let owned = fs::read_to_string(operator.directory.join("owner.txt")).unwrap();
    let expected = format!(
        "Vize release operator v2\n\nTag: v1.2.3\nSource-head: {head}\nOperator-run: 7\nOperator-attempt: 2\nOperator-source: {cut}\nOriginal-deadline-unix-millis: {deadline}\nPID: {}\nNonce: ",
        std::process::id()
    );
    let nonce = owned.strip_prefix(&expected).unwrap();
    assert!(
        nonce
            .parse::<u128>()
            .is_ok_and(|value| value > 0 && value.to_string() == nonce)
    );
    let object = github::git(&["cat-file", "commit", &lease], &repo.work).unwrap();
    assert_eq!(object.split_once("\n\n").unwrap().1, owned);
    assert_eq!(budget.child_allowance().unwrap().1, deadline);
    let source = source(&head, &cut);
    install_pin(&source, &operator, &repo.work).unwrap();
    verify_pin(&source, &repo.work).unwrap();
    let pin = remote("refs/heads/release-pin/v1.2.3", &repo.work).unwrap();
    assert!(acquire("v1.2.3", &head, &repo.work).is_err());
    assert_eq!(
        remote("refs/heads/main", &repo.work).unwrap().as_deref(),
        Some(cut.as_str())
    );
    assert_eq!(
        remote("refs/heads/release/v1.2.3", &repo.work)
            .unwrap()
            .as_deref(),
        Some(head.as_str())
    );
    assert!(github::tag_target("v1.2.3", &repo.work).unwrap().is_none());
    drop(operator);
    assert!(
        remote("refs/heads/release-operator/v1.2.3", &repo.work)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        remote("refs/heads/release-pin/v1.2.3", &repo.work).unwrap(),
        pin
    );
    let resumed = acquire("v1.2.3", &head, &repo.work).unwrap();
    assert!(
        !resumed.allows_early_admission(),
        "resume cannot mint a new pending-Q deadline"
    );
    verify_pin(&source, &repo.work).unwrap();
}

#[test]
fn changed_versioned_lease_is_preserved_on_error_and_drop() {
    let repo = Repo::new();
    let head = repo.commit(&[("source.rs", "// original source\n")]);
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    let operator = acquire_with_admission(
        "v1.2.3",
        &head,
        Some(Admission {
            run: 7,
            attempt: 2,
            source: "1111111111111111111111111111111111111111".into(),
            deadline: 1_900_000_000_000,
        }),
        &repo.work,
    )
    .unwrap();
    let directory = operator.directory.clone();
    let owned = fs::read(directory.join("owner.txt")).unwrap();
    let original = fs::read(directory.join("remote-head.txt")).unwrap();
    github::git(
        &["update-ref", "refs/heads/release-operator/v1.2.3", &head],
        &repo.remote,
    )
    .unwrap();
    assert!(operator.verify().is_err());
    drop(operator);
    assert_eq!(
        remote("refs/heads/release-operator/v1.2.3", &repo.work)
            .unwrap()
            .as_deref(),
        Some(head.as_str())
    );
    assert_eq!(fs::read(directory.join("owner.txt")).unwrap(), owned);
    assert_eq!(
        fs::read(directory.join("remote-head.txt")).unwrap(),
        original
    );
    assert!(acquire("v1.2.3", &head, &repo.work).is_err());
    // Only test-fixture teardown; production never steals the foreign lease.
    fs::remove_dir_all(directory).unwrap();
}

impl Operator {
    pub(in super::super) fn compensation_fixture(
        source: &Source,
        deadline: u64,
        root: &Path,
    ) -> Result<Self, String> {
        acquire_with_admission(
            &source.candidate.tag,
            &source.candidate.head,
            Some(Admission {
                run: 7,
                attempt: 2,
                source: source.cut.clone(),
                deadline,
            }),
            root,
        )
    }
}
