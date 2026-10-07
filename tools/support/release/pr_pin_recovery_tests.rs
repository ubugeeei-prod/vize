use super::super::pr_github as github;
use super::delivery::Receipt;
use super::lock;
use super::tags;
use super::tests::{Repo, source};
use std::fs;

#[test]
fn interrupted_tag_push_reuses_only_the_exact_local_receipt() {
    let repo = Repo::new();
    let cut = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.2\"\n")]);
    let head = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.3\"\n")]);
    let source = source(&head, &cut);
    let receipt = Receipt {
        head: head.clone(),
        merge: cut.clone(),
        parent: cut.clone(),
        gate: cut,
    };
    tags::ensure_local_tag(&source, 7, &receipt, &repo.work).unwrap();
    let original = github::git(&["rev-parse", "refs/tags/v1.2.3"], &repo.work).unwrap();
    assert!(github::tag_target("v1.2.3", &repo.work).unwrap().is_none());
    // Simulate a failed/never-sent push after annotation creation. Resume must
    // reuse its precise object instead of recreating, deleting, or overwriting.
    tags::ensure_local_tag(&source, 7, &receipt, &repo.work).unwrap();
    assert_eq!(
        original,
        github::git(&["rev-parse", "refs/tags/v1.2.3"], &repo.work).unwrap()
    );
    assert!(tags::ensure_local_tag(&source, 8, &receipt, &repo.work).is_err());
    let mut different = receipt.clone();
    different.parent = head;
    assert!(tags::ensure_local_tag(&source, 7, &different, &repo.work).is_err());
    tags::push_tag(&repo.work, "refs/tags/v1.2.3").unwrap();
    assert_eq!(tags::tag_receipt(&source, &repo.work).unwrap().0, 7);
}

#[cfg(unix)]
#[test]
fn source_changed_after_advertisement_cannot_authorize_the_installed_marker() {
    use std::os::unix::fs::PermissionsExt;
    let repo = Repo::new();
    let cut = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.2\"\n")]);
    let head = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.3\"\n")]);
    let source = source(&head, &cut);
    github::git(
        &[
            "push",
            "origin",
            &format!("{head}:refs/heads/release/v1.2.3"),
        ],
        &repo.work,
    )
    .unwrap();
    let changed = repo.commit(&[("ordinary.rs", "// concurrent source change\n")]);
    github::git(
        &[
            "push",
            "origin",
            &format!("{changed}:refs/heads/race-object"),
        ],
        &repo.work,
    )
    .unwrap();
    let operator = lock::acquire("v1.2.3", &head, &repo.work).unwrap();
    let hook = repo.work.join(".git/hooks/pre-push");
    fs::write(&hook, format!("#!/bin/sh\ngit --git-dir=\"$2\" update-ref refs/heads/release/v1.2.3 {changed} {head}\n")).unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    let rejected = lock::install_pin(&source, &operator, &repo.work).unwrap_err();
    fs::remove_file(&hook).unwrap();
    assert!(rejected.contains("remote source changed"), "{rejected}");
    let actual = github::git(
        &[
            "ls-remote",
            "--heads",
            "origin",
            "refs/heads/release/v1.2.3",
        ],
        &repo.work,
    )
    .unwrap();
    assert_eq!(actual.split_whitespace().next(), Some(changed.as_str()));
    assert!(lock::verify_pin(&source, &repo.work).is_err());
    // This demonstrates the no-op ref really was omitted: the marker exists,
    // but its authentication fails and cannot advance body/cancellation/tag.
    assert!(
        !github::git(
            &[
                "ls-remote",
                "--heads",
                "origin",
                "refs/heads/release-pin/v1.2.3"
            ],
            &repo.work
        )
        .unwrap()
        .is_empty()
    );
    assert!(repo.remote.is_dir());
}
