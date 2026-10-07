use super::super::{pr_contract::Candidate, pr_github as github};
use super::{Source, delivery, dispatch, first_parent, lock, marker, metadata, run_identity};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
const HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CUT: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

pub(super) fn source(head: &str, cut: &str) -> Source {
    Source {
        candidate: Candidate {
            number: 42,
            repository: "owner/repo".into(),
            author: "maintainer".into(),
            branch: "release/v1.2.3".into(),
            head: head.into(),
            base: cut.into(),
            tag: "v1.2.3".into(),
            merged: false,
        },
        cut: cut.into(),
        base_version: "1.2.2".into(),
        integration: 99,
        closed: false,
    }
}
fn run(pinned: bool) -> Value {
    json!({"id":7,"event":"workflow_dispatch","head_sha":HEAD,"head_branch":"release/v1.2.3","head_repository":{"full_name":"owner/repo"},"path":".github/workflows/release.yml","display_title":format!("{}Release v1.2.3 PR #42 @ {HEAD}",if pinned {"Pinned "} else {""}),"status":"in_progress"})
}
#[test]
fn immutable_markers_reject_duplicates_and_missing_fields() {
    assert_eq!(marker("<!-- pin: value -->", "pin").unwrap(), "value");
    for body in [
        "",
        "<!-- pin:  -->",
        "<!-- pin: value -->\n<!-- pin: other -->",
    ] {
        assert!(marker(body, "pin").is_err());
    }
}
#[test]
fn pinned_runs_cannot_borrow_legacy_or_foreign_identity() {
    let source = source(HEAD, CUT);
    assert!(run_identity(&run(true), &source, 7).is_ok());
    assert!(run_identity(&run(false), &source, 7).is_err());
    for (pointer, value) in [
        ("/head_sha", json!(CUT)),
        ("/id", json!(8)),
        ("/event", json!("push")),
        ("/path", json!(".github/workflows/check.yml")),
        ("/head_branch", json!("main")),
        ("/head_repository/full_name", json!("fork/repo")),
    ] {
        let mut mutated = run(true);
        *mutated.pointer_mut(pointer).unwrap() = value;
        assert!(run_identity(&mutated, &source, 7).is_err(), "{pointer}");
    }
}
#[test]
fn mode_switch_can_retire_only_revoked_same_head_legacy_release() {
    let source = source(HEAD, CUT);
    assert!(dispatch::obsolete_legacy(&run(false), &source));
    assert!(!dispatch::obsolete_legacy(&run(true), &source));
    for (pointer, value) in [
        ("/head_sha", json!(CUT)),
        ("/event", json!("merge_group")),
        ("/path", json!(".github/workflows/fuzz.yml")),
        ("/status", json!("completed")),
        ("/head_repository/full_name", json!("fork/repo")),
    ] {
        let mut mutated = run(false);
        *mutated.pointer_mut(pointer).unwrap() = value;
        assert!(!dispatch::obsolete_legacy(&mutated, &source), "{pointer}");
    }
}
#[test]
fn actual_delivery_requires_valid_signature_and_single_parent() {
    let valid = json!({"sha":HEAD,"commit":{"verification":{"verified":true,"reason":"valid"}},"parents":[{"sha":CUT}]});
    assert!(delivery::signed_merge(&valid, HEAD).is_ok());
    for (pointer, value) in [
        ("/sha", json!(CUT)),
        ("/commit/verification/verified", json!(false)),
        ("/commit/verification/reason", json!("unsigned")),
        ("/parents", json!([])),
        ("/parents", json!([{"sha":CUT},{"sha":HEAD}])),
    ] {
        let mut mutated = valid.clone();
        *mutated.pointer_mut(pointer).unwrap() = value;
        assert!(delivery::signed_merge(&mutated, HEAD).is_err(), "{pointer}");
    }
}
#[test]
fn canonical_versions_preserve_oracles_non_version_values_and_newlines() {
    let readme = "current 1.2.2\n<!-- benchmark:readme:start -->\nmeasured 1.2.2\n<!-- benchmark:readme:end -->\n";
    assert_eq!(
        metadata::rewrite("README.md", readme, "1.2.2", "1.2.3"),
        "current 1.2.3\n<!-- benchmark:readme:start -->\nmeasured 1.2.2\n<!-- benchmark:readme:end -->\n"
    );
    let lock = "[[package]]\nname = \"vize_guest\"\nversion = \"1.2.2\"\n\n[[package]]\nname = \"unrelated\"\nversion = \"1.2.2\"\n";
    assert_eq!(
        metadata::rewrite("Cargo.lock", lock, "1.2.2", "1.2.3"),
        lock.replacen("version = \"1.2.2\"", "version = \"1.2.3\"", 1)
    );
    let aliased = "[workspace.dependencies]\noxc_formatter = { package = \"vize_oxc_formatter\", version = \"=1.2.2\", path = \"vendor/oxc_formatter\" }\n";
    assert_eq!(
        metadata::rewrite("Cargo.toml", aliased, "1.2.2", "1.2.3"),
        aliased.replace("=1.2.2", "=1.2.3")
    );
    assert_eq!(
        metadata::rewrite(
            "src/version.rs",
            "const SAMPLE: &str = \"1.2.2\";\n",
            "1.2.2",
            "1.2.3"
        ),
        "const SAMPLE: &str = \"1.2.2\";\n"
    );
}

pub(super) struct Repo {
    directory: PathBuf,
    pub(super) work: PathBuf,
    pub(super) remote: PathBuf,
}
impl Repo {
    pub(super) fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "vize-release-pin-law-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let work = directory.join("work");
        let remote = directory.join("remote.git");
        fs::create_dir_all(&work).unwrap();
        github::git(&["init", "--bare", remote.to_str().unwrap()], &directory).unwrap();
        github::git(&["init", "-b", "main"], &work).unwrap();
        for (key, value) in [
            ("user.name", "Release law"),
            ("user.email", "law@example.invalid"),
            ("commit.gpgsign", "false"),
            ("tag.gpgsign", "false"),
        ] {
            github::git(&["config", key, value], &work).unwrap();
        }
        github::git(
            &["remote", "add", "origin", remote.to_str().unwrap()],
            &work,
        )
        .unwrap();
        Self {
            directory,
            work,
            remote,
        }
    }
    pub(super) fn commit(&self, files: &[(&str, &str)]) -> String {
        for (path, text) in files {
            let target = self.work.join(path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(target, text).unwrap();
        }
        github::git(&["add", "."], &self.work).unwrap();
        github::git(
            &[
                "-c",
                "core.hooksPath=/dev/null",
                "commit",
                "--allow-empty",
                "-m",
                "law",
            ],
            &self.work,
        )
        .unwrap();
        github::git(&["rev-parse", "HEAD"], &self.work).unwrap()
    }
    fn main(&self) -> String {
        github::git(&["rev-parse", "refs/heads/main"], &self.remote).unwrap()
    }
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}
const BASE_CARGO: &str = "[workspace.package]\nversion = \"1.2.2\"\n";
const NEXT_CARGO: &str = "[workspace.package]\nversion = \"1.2.3\"\n";

#[test]
fn version_delivery_uses_actual_parent_after_an_ordinary_predecessor() {
    let repo = Repo::new();
    let cut = repo.commit(&[("Cargo.toml", BASE_CARGO), ("src/code.rs", "original\n")]);
    let predecessor = repo.commit(&[("src/code.rs", "ordinary fix\n")]);
    let version = repo.commit(&[("Cargo.toml", NEXT_CARGO)]);
    assert_ne!(predecessor, cut);
    first_parent(&cut, &version, &repo.work).unwrap();
    metadata::verify_delta(&predecessor, &version, "1.2.2", "1.2.3", &repo.work).unwrap();
    assert!(metadata::verify_delta(&cut, &version, "1.2.2", "1.2.3", &repo.work).is_err());
    assert_eq!(
        metadata::bytes(&["show", &format!("{version}:src/code.rs")], &repo.work).unwrap(),
        b"ordinary fix\n"
    );
}
#[test]
fn version_delivery_rejects_hidden_edits_omitted_pins_and_mode_changes() {
    let repo = Repo::new();
    let parent = repo.commit(&[
        ("Cargo.toml", BASE_CARGO),
        (
            "npm/native/package.json",
            "{\n  \"version\": \"1.2.2\"\n}\n",
        ),
        ("src/code.rs", "original\n"),
    ]);
    let omitted = repo.commit(&[("Cargo.toml", NEXT_CARGO)]);
    assert!(metadata::verify_delta(&parent, &omitted, "1.2.2", "1.2.3", &repo.work).is_err());
    let changed = repo.commit(&[
        (
            "npm/native/package.json",
            "{\n  \"version\": \"1.2.3\"\n}\n",
        ),
        ("src/code.rs", "smuggled\n"),
    ]);
    assert!(metadata::verify_delta(&parent, &changed, "1.2.2", "1.2.3", &repo.work).is_err());
    github::git(&["reset", "--hard", &parent], &repo.work).unwrap();
    repo.commit(&[
        ("Cargo.toml", NEXT_CARGO),
        (
            "npm/native/package.json",
            "{\n  \"version\": \"1.2.3\"\n}\n",
        ),
    ]);
    github::git(&["update-index", "--chmod=+x", "Cargo.toml"], &repo.work).unwrap();
    github::git(
        &["-c", "core.hooksPath=/dev/null", "commit", "-m", "mode"],
        &repo.work,
    )
    .unwrap();
    let mode = github::git(&["rev-parse", "HEAD"], &repo.work).unwrap();
    assert!(metadata::verify_delta(&parent, &mode, "1.2.2", "1.2.3", &repo.work).is_err());
}
#[test]
fn tag_only_push_preserves_moving_main_and_rejects_foreign_tags() {
    let repo = Repo::new();
    let cut = repo.commit(&[("Cargo.toml", BASE_CARGO)]);
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    let head = repo.commit(&[("Cargo.toml", NEXT_CARGO)]);
    github::git(&["tag", "-a", "v1.2.3", &head, "-m", "pinned"], &repo.work).unwrap();
    github::git(&["reset", "--hard", &cut], &repo.work).unwrap();
    let advanced = repo.commit(&[("ordinary", "later source\n")]);
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    delivery::push_tag(&repo.work, "refs/tags/v1.2.3").unwrap();
    assert_eq!(repo.main(), advanced);
    assert_eq!(
        github::tag_target("v1.2.3", &repo.work).unwrap().as_deref(),
        Some(head.as_str())
    );
    github::git(&["tag", "-d", "v1.2.3"], &repo.work).unwrap();
    github::git(&["tag", "-a", "v1.2.3", &cut, "-m", "foreign"], &repo.work).unwrap();
    assert!(delivery::push_tag(&repo.work, "refs/tags/v1.2.3").is_err());
    assert_eq!(repo.main(), advanced);
}
#[test]
fn operator_and_marker_leases_serialize_and_leave_main_untouched() {
    let repo = Repo::new();
    let cut = repo.commit(&[("Cargo.toml", BASE_CARGO)]);
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    let head = repo.commit(&[("Cargo.toml", NEXT_CARGO)]);
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
    let guard = lock::acquire("v1.2.3", &head, &repo.work).unwrap();
    assert!(lock::acquire("v1.2.3", &head, &repo.work).is_err());
    guard.verify().unwrap();
    lock::install_pin(&source, &guard, &repo.work).unwrap();
    lock::install_pin(&source, &guard, &repo.work).unwrap();
    lock::verify_pin(&source, &repo.work).unwrap();
    assert!(lock::reject_pinned("v1.2.3", &repo.work).is_err());
    assert_eq!(repo.main(), cut);
    let mut wrong = source.clone();
    wrong.integration = 100;
    assert!(lock::verify_pin(&wrong, &repo.work).is_err());
    drop(guard);
    let retry = lock::acquire("v1.2.3", &head, &repo.work).unwrap();
    retry.verify().unwrap();
    drop(retry);
    assert_eq!(repo.main(), cut);
}
#[test]
fn actual_squash_receipt_does_not_require_pr_head_ancestry() {
    let repo = Repo::new();
    let cut = repo.commit(&[("Cargo.toml", BASE_CARGO)]);
    github::git(&["checkout", "-b", "integration"], &repo.work).unwrap();
    let pr_head = repo.commit(&[("Cargo.toml", NEXT_CARGO)]);
    github::git(&["checkout", "main"], &repo.work).unwrap();
    repo.commit(&[("Cargo.toml", NEXT_CARGO)]);
    github::git(
        &[
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "--amend",
            "-m",
            "actual squash",
        ],
        &repo.work,
    )
    .unwrap();
    let actual = github::git(&["rev-parse", "HEAD"], &repo.work).unwrap();
    assert!(
        github::git(
            &["merge-base", "--is-ancestor", &pr_head, &actual],
            &repo.work
        )
        .is_err()
    );
    first_parent(&cut, &actual, &repo.work).unwrap();
    metadata::verify_delta(&cut, &actual, "1.2.2", "1.2.3", &repo.work).unwrap();
    let paths = [
        "check.yml",
        "n8n-adoption.yml",
        "musea-browser.yml",
        "nuxt-style-build.yml",
        "nuxt3-module-build.yml",
    ];
    let runs:Vec<Value>=paths.iter().enumerate().map(|(id,path)|json!({"id":id+1,"event":"merge_group","head_sha":actual,"head_branch":format!("gh-readonly-queue/main/pr-99-{cut}"),"head_repository":{"full_name":"owner/repo"},"path":format!(".github/workflows/{path}"),"status":"completed","conclusion":"success"})).collect();
    assert!(delivery::gates::full_workflows(&runs, "owner/repo", &actual, 99, &cut).unwrap());
    assert!(!delivery::gates::full_workflows(&runs, "owner/repo", &pr_head, 99, &cut).unwrap());
    let mut failed = runs.clone();
    failed[0]["conclusion"] = json!("skipped");
    assert!(delivery::gates::full_workflows(&failed, "owner/repo", &actual, 99, &cut).is_err());
    let mut foreign = runs;
    foreign[0]["head_branch"] = json!(format!("gh-readonly-queue/main/pr-98-{cut}"));
    assert!(!delivery::gates::full_workflows(&foreign, "owner/repo", &actual, 99, &cut).unwrap());
}
