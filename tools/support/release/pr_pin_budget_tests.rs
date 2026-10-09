use super::super::{pr_budget::Budget, pr_github as github};
use super::{
    lock,
    tests::{Repo, source},
};
use std::{fs, thread, time::Duration};

#[test]
fn expected_hosted_cut_is_strict_and_does_not_restrict_unconfigured_local_cuts() {
    let cut = "1111111111111111111111111111111111111111";
    let verify = super::start::expected_cut;
    assert!(verify(None, cut).is_ok());
    assert!(verify(Some(cut), cut).is_ok());
    for expected in [
        "",
        "main",
        "1111111",
        "2222222222222222222222222222222222222222",
    ] {
        assert!(verify(Some(expected), cut).is_err());
    }
}

fn expire() -> Result<(), String> {
    let budget = Budget::new(Duration::from_millis(1));
    thread::sleep(Duration::from_millis(2));
    budget.check()
}

#[test]
fn operator_timeout_releases_its_real_lease_and_preserves_resumable_source_and_pin() {
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
    let source = source(&head, &cut);
    let directory = repo.work.join(".git/vize-release-operator-v1.2.3.lock");
    let result = (|| {
        let operator = lock::acquire("v1.2.3", &head, &repo.work)?;
        lock::install_pin(&source, &operator, &repo.work)?;
        assert!(directory.join("remote-head.txt").is_file());
        expire()
    })();
    assert!(result.unwrap_err().contains("budget expired"));
    assert!(!directory.exists());
    assert!(
        github::git(
            &[
                "ls-remote",
                "--refs",
                "origin",
                "refs/heads/release-operator/v1.2.3"
            ],
            &repo.work
        )
        .unwrap()
        .is_empty()
    );
    lock::verify_pin(&source, &repo.work).unwrap();
    assert_eq!(
        github::git(&["rev-parse", "refs/heads/main"], &repo.remote).unwrap(),
        cut
    );
    assert!(github::tag_target("v1.2.3", &repo.work).unwrap().is_none());
    let resumed = lock::acquire("v1.2.3", &head, &repo.work).unwrap();
    resumed.verify().unwrap();
    lock::verify_pin(&source, &repo.work).unwrap();
    drop(resumed);
    assert!(!directory.exists());
}

#[test]
fn operator_timeout_never_deletes_a_changed_remote_lease() {
    let repo = Repo::new();
    let head = repo.commit(&[("source.rs", "// original source\n")]);
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    let directory = repo.work.join(".git/vize-release-operator-v1.2.3.lock");
    let result = (|| {
        let _operator = lock::acquire("v1.2.3", &head, &repo.work)?;
        github::git(
            &["update-ref", "refs/heads/release-operator/v1.2.3", &head],
            &repo.remote,
        )?;
        expire()
    })();
    assert!(result.is_err());
    assert_eq!(
        github::git(
            &["rev-parse", "refs/heads/release-operator/v1.2.3"],
            &repo.remote
        )
        .unwrap(),
        head
    );
    assert!(directory.join("owner.txt").is_file());
    assert!(directory.join("remote-head.txt").is_file());
    assert!(lock::acquire("v1.2.3", &head, &repo.work).is_err());
    // Test fixture cleanup has no remote ownership recovery semantics.
    fs::remove_dir_all(directory).unwrap();
}
