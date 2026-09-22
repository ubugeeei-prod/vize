use super::{pr_contract as contract, pr_github as github, pr_promote};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BASE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn pull() -> Value {
    json!({"number": 42, "state": "open", "merged": false, "draft": false,
        "user": {"login": "maintainer"},
        "head": {"sha": HEAD, "ref": "release/v1.2.3", "repo": {"full_name": "owner/repo"}},
        "base": {"sha": BASE, "ref": "main", "repo": {"full_name": "owner/repo"}}})
}

#[test]
fn only_maintain_and_admin_authors_can_release() {
    for role in ["maintain", "admin"] {
        assert!(
            contract::candidate(
                &pull(),
                &json!({"role_name": role}),
                "owner/repo",
                HEAD,
                "v1.2.3",
                false
            )
            .is_ok()
        );
    }
    for role in ["write", "read", "triage", "none", "custom-role"] {
        assert!(
            contract::candidate(
                &pull(),
                &json!({"role_name": role, "permission": "admin"}),
                "owner/repo",
                HEAD,
                "v1.2.3",
                false
            )
            .is_err()
        );
    }
    assert!(contract::maintainer(&json!({"permission": "write"})).is_err());
}

#[test]
fn candidate_rejects_forks_drafts_wrong_bases_and_changed_heads() {
    for (pointer, value) in [
        ("/head/repo/full_name", json!("fork/repo")),
        ("/base/repo/full_name", json!("other/repo")),
        ("/base/ref", json!("develop")),
        ("/head/ref", json!("feature")),
        ("/head/sha", json!(BASE)),
        ("/state", json!("closed")),
        ("/draft", json!(true)),
    ] {
        let mut pr = pull();
        *pr.pointer_mut(pointer).unwrap() = value;
        assert!(
            contract::candidate(
                &pr,
                &json!({"role_name": "maintain"}),
                "owner/repo",
                HEAD,
                "v1.2.3",
                false
            )
            .is_err(),
            "{pointer}"
        );
    }
    assert!(contract::current_parent(BASE, HEAD).is_err());
    assert!(contract::current_parent(BASE, BASE).is_ok());
}

#[test]
fn readiness_never_accepts_missing_skipped_duplicate_or_failed_evidence() {
    assert!(!contract::ready_job(&[]).unwrap());
    let ready =
        json!({"name": "Release candidate ready", "status": "completed", "conclusion": "success"});
    assert!(contract::ready_job(&[ready.clone()]).unwrap());
    assert!(contract::ready_job(&[ready.clone(), ready]).is_err());
    for conclusion in ["failure", "skipped", "cancelled", "timed_out", "neutral"] {
        assert!(contract::ready_job(&[json!({"name": "Release candidate ready", "status": "completed", "conclusion": conclusion})]).is_err());
    }
    assert!(
        !contract::ready_job(&[
            json!({"name": "Release candidate ready", "status": "in_progress", "conclusion": null})
        ])
        .unwrap()
    );
}

struct Repo {
    directory: PathBuf,
    work: PathBuf,
    remote: PathBuf,
}

impl Repo {
    fn new() -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "vize-release-atomic-test-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let work = directory.join("work");
        let remote = directory.join("remote.git");
        fs::create_dir_all(&work).unwrap();
        github::git(&["init", "--bare", remote.to_str().unwrap()], &directory).unwrap();
        github::git(&["init", "-b", "main"], &work).unwrap();
        github::git(&["config", "user.name", "Release test"], &work).unwrap();
        github::git(&["config", "user.email", "release@example.invalid"], &work).unwrap();
        github::git(&["config", "commit.gpgsign", "false"], &work).unwrap();
        github::git(&["config", "tag.gpgsign", "false"], &work).unwrap();
        github::git(
            &["remote", "add", "origin", remote.to_str().unwrap()],
            &work,
        )
        .unwrap();
        github::git(
            &[
                "-c",
                "core.hooksPath=/dev/null",
                "commit",
                "--allow-empty",
                "-m",
                "main",
            ],
            &work,
        )
        .unwrap();
        github::git(&["push", "origin", "main"], &work).unwrap();
        Self {
            directory,
            work,
            remote,
        }
    }
    fn candidate(&self) -> String {
        github::git(&["checkout", "-b", "release/v1.2.3"], &self.work).unwrap();
        github::git(
            &[
                "-c",
                "core.hooksPath=/dev/null",
                "commit",
                "--allow-empty",
                "-m",
                "release",
            ],
            &self.work,
        )
        .unwrap();
        github::git(&["tag", "-a", "v1.2.3", "-m", "candidate"], &self.work).unwrap();
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

#[test]
fn validated_commit_and_annotated_tag_are_promoted_together() {
    let repo = Repo::new();
    let head = repo.candidate();
    pr_promote::push_atomic(&repo.work, &head, "refs/tags/v1.2.3").unwrap();
    assert_eq!(repo.main(), head);
    assert_eq!(
        github::tag_target("v1.2.3", &repo.work).unwrap().as_deref(),
        Some(head.as_str())
    );
}

#[test]
fn main_advance_rejects_the_entire_transaction() {
    let repo = Repo::new();
    let head = repo.candidate();
    github::git(&["checkout", "main"], &repo.work).unwrap();
    github::git(
        &[
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "--allow-empty",
            "-m",
            "concurrent change",
        ],
        &repo.work,
    )
    .unwrap();
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    let advanced = repo.main();
    assert!(pr_promote::push_atomic(&repo.work, &head, "refs/tags/v1.2.3").is_err());
    assert_eq!(repo.main(), advanced);
    assert_eq!(github::tag_target("v1.2.3", &repo.work).unwrap(), None);
}

#[test]
fn existing_tag_rejects_main_update_too() {
    let repo = Repo::new();
    let original = repo.main();
    github::git(&["tag", "v1.2.3"], &repo.work).unwrap();
    github::git(&["push", "origin", "refs/tags/v1.2.3"], &repo.work).unwrap();
    github::git(&["tag", "-d", "v1.2.3"], &repo.work).unwrap();
    let head = repo.candidate();
    assert!(pr_promote::push_atomic(&repo.work, &head, "refs/tags/v1.2.3").is_err());
    assert_eq!(repo.main(), original);
    assert_eq!(
        github::tag_target("v1.2.3", &repo.work).unwrap().as_deref(),
        Some(original.as_str())
    );
}

#[test]
fn workspace_moonbit_paths_survive_the_isolated_worktree() {
    let root = std::path::Path::new("/original/workspace");
    let command = super::pr_start::preparation_command(
        root,
        Some(".cache/moonbit/bin/moon"),
        Some(".cache/moonbit"),
    );
    assert_eq!(command.get_program(), root.join(".cache/moonbit/bin/moon"));
    assert!(
        command.get_envs().any(|(key, value)| key == "MOON_HOME"
            && value == Some(root.join(".cache/moonbit").as_os_str()))
    );
    let command = super::pr_start::preparation_command(root, Some("moon"), None);
    assert_eq!(command.get_program(), "moon");
}

#[test]
fn required_checks_include_uncreated_jobs_and_the_configured_app() {
    let rules = json!([{"type": "required_status_checks", "parameters": {
        "required_status_checks": [{"context": "test-report", "integration_id": 15368}]
    }}]);
    let check = json!({"id": 1, "name": "test-report", "head_sha": HEAD,
        "app": {"id": 15368}, "status": "completed", "conclusion": "success"});
    let ready = |checks: &[Value]| super::pr_checks::required_checks(&rules, checks, HEAD);
    assert!(
        !ready(&[]).unwrap(),
        "a job absent from the PR rollup still blocks promotion"
    );
    assert!(ready(&[check.clone()]).unwrap());
    for (pointer, value) in [("/app/id", json!(999)), ("/head_sha", json!(BASE))] {
        let mut wrong = check.clone();
        *wrong.pointer_mut(pointer).unwrap() = value;
        assert!(!ready(&[wrong]).unwrap());
    }
    let mut newer = check.clone();
    newer["id"] = json!(2);
    newer["status"] = json!("queued");
    newer["conclusion"] = Value::Null;
    assert!(!ready(&[check.clone(), newer.clone()]).unwrap());
    newer["status"] = json!("completed");
    newer["conclusion"] = json!("failure");
    assert!(ready(&[check, newer]).is_err());
    assert!(super::pr_checks::required_checks(&json!([]), &[], HEAD).is_err());
}
