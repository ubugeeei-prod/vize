use super::{pr_contract as contract, pr_github as github, pr_promote, pr_watch};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BASE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

pub(super) fn git_commit_packet(root: &std::path::Path, id: &str) -> Value {
    let raw = github::git(&["cat-file", "commit", id], root).unwrap();
    let (headers, message) = raw.split_once("\n\n").unwrap();
    let tree = headers
        .lines()
        .find_map(|line| line.strip_prefix("tree "))
        .unwrap();
    let parents: Vec<_> = headers
        .lines()
        .filter_map(|line| line.strip_prefix("parent "))
        .map(|sha| json!({"sha":sha}))
        .collect();
    json!({"sha":id,"tree":{"sha":tree},"parents":parents,"message":message})
}

pub(super) fn direct_compensation_packet(
    runner: &str,
    script: &str,
    deadline: u64,
    root: &std::path::Path,
) -> Value {
    let input =
        json!({"program":"node","args":["-e",script],"deadline":deadline,"root":root}).to_string();
    let raw = github::output(
        "node",
        &["--input-type=module-typescript", "-e", runner, &input],
        root,
    )
    .unwrap();
    serde_json::from_str(&raw).unwrap()
}

pub(super) fn assert_compensation_processes_stopped(
    root: &std::path::Path,
    pids: &std::path::Path,
) {
    // Rust reaps its direct timeout child. Orphan descendants may be PID1's
    // zombies; those must be non-runnable rather than falsely called reaped.
    let script = r#"const fs=require('node:fs');const rows=fs.readFileSync(process.argv[1],'utf8').trim().split('\n').map(JSON.parse);if(rows.length<2)throw Error('Missing PID evidence');for(const row of rows){try{process.kill(row.pid,0)}catch(error){if(error.code==='ESRCH')continue;throw error}if(row.role==='launcher')throw Error('Direct launcher was not reaped');let stat;try{stat=fs.readFileSync('/proc/'+row.pid+'/stat','utf8')}catch(error){if(error.code==='ENOENT')continue;throw error}if(stat.slice(stat.lastIndexOf(')')+2).split(' ')[0]!=='Z')throw Error('Descendant remains runnable')}process.stdout.write(JSON.stringify(rows))"#;
    github::output("node", &["-e", script, pids.to_str().unwrap()], root).unwrap();
}

pub(super) fn queue_mutations(log: &std::path::Path) -> usize {
    fs::read_to_string(log)
        .unwrap()
        .matches("query=mutation")
        .count()
}

pub(super) fn compensation_deadline(milliseconds: u64) -> u64 {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
    .unwrap()
        + milliseconds
}

#[test]
fn shared_release_checks_accept_only_documented_pending_exit_with_actual_evidence() {
    let pending = json!([
        {"name":"check-js","bucket":"pending","link":"https://example.invalid/running"},
        {"name":"test-report","bucket":"pass","link":"https://example.invalid/report"}
    ]);
    let parse = |code, rows: &Value, stderr: &[u8]| {
        pr_promote::parse_checks_output(code, &serde_json::to_vec(rows).unwrap(), stderr)
    };
    let (observed, waiting) = parse(Some(8), &pending, b"").unwrap();
    assert_eq!(observed, pending);
    assert!(
        waiting,
        "pending exit must return false from shared checks_pass"
    );
    for code in [None, Some(1), Some(2), Some(4), Some(7), Some(9)] {
        assert!(parse(code, &pending, b"authentication or transport failure").is_err());
    }
    for rows in [
        json!([]),
        json!({}),
        json!([{"name":"check-js","bucket":"pass"}]),
        json!([{"bucket":"pending"}]),
        json!([{"name":"check-js","bucket":"unknown"}]),
    ] {
        assert!(parse(Some(8), &rows, b"").is_err());
    }
    for bucket in ["fail", "cancel"] {
        let rows =
            json!([{"name":"check-js","bucket":"pending"},{"name":"test-report","bucket":bucket}]);
        for code in [Some(0), Some(8)] {
            assert!(parse(code, &rows, b"").is_err());
        }
    }
    assert!(parse(Some(8), &pending, b"HTTP 401: authentication failed").is_err());
    assert!(pr_promote::parse_checks_output(Some(8), b"{invalid json", b"").is_err());
}

#[test]
fn legacy_and_integration_checks_keep_exact_required_names_heads_apps_and_success() {
    let names = ["check-js", "check-vize-apps", "fmt-rust", "test-report"];
    let rules = json!([{"type":"required_status_checks","parameters":{"required_status_checks":
        names.map(|context|json!({"context":context,"integration_id":15368}))}}]);
    for head in [HEAD, "cccccccccccccccccccccccccccccccccccccccc"] {
        let selected:Vec<_>=names.iter().map(|name|json!({"name":name,"bucket":"pass","link":format!("https://example.invalid/{name}")})).collect();
        let checks:Vec<_>=names.iter().map(|name|json!({"name":name,"head_sha":head,"app":{"id":15368},
            "details_url":format!("https://example.invalid/{name}"),"status":"completed","conclusion":"success"})).collect();
        let (parsed, waiting) =
            pr_promote::parse_checks_output(Some(0), &serde_json::to_vec(&selected).unwrap(), b"")
                .unwrap();
        assert!(!waiting);
        assert!(
            super::pr_checks::required_checks(&rules, &checks, parsed.as_array().unwrap(), head)
                .unwrap()
        );
        for index in 0..4 {
            for (pointer, value) in [
                ("/head_sha", json!(BASE)),
                ("/app/id", json!(999)),
                ("/conclusion", json!("skipped")),
            ] {
                let mut changed = checks.clone();
                *changed[index].pointer_mut(pointer).unwrap() = value;
                assert!(
                    !super::pr_checks::required_checks(&rules, &changed, &selected, head)
                        .unwrap_or(false)
                );
            }
            let mut missing = checks.clone();
            missing.remove(index);
            assert!(!super::pr_checks::required_checks(&rules, &missing, &selected, head).unwrap());
        }
    }
}

fn pull() -> Value {
    json!({"number": 42, "state": "open", "merged": false, "draft": true,
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
fn candidate_rejects_forks_ready_prs_wrong_bases_and_changed_heads() {
    for (pointer, value) in [
        ("/head/repo/full_name", json!("fork/repo")),
        ("/base/repo/full_name", json!("other/repo")),
        ("/base/ref", json!("develop")),
        ("/head/ref", json!("feature")),
        ("/head/sha", json!(BASE)),
        ("/state", json!("closed")),
        ("/draft", json!(false)),
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
fn release_pr_stays_draft_until_atomic_promotion() {
    let mut pr = pull();
    assert!(!contract::needs_draft(&pr).unwrap());
    pr["draft"] = json!(false);
    assert!(contract::needs_draft(&pr).unwrap());
    assert!(
        contract::candidate(
            &pr,
            &json!({"role_name": "maintain"}),
            "owner/repo",
            HEAD,
            "v1.2.3",
            false
        )
        .is_err()
    );
    pr["draft"] = json!(true);
    assert!(
        contract::candidate(
            &pr,
            &json!({"role_name": "maintain"}),
            "owner/repo",
            HEAD,
            "v1.2.3",
            false
        )
        .is_ok()
    );
    // GitHub may retain or clear draft when the head is indirectly merged.
    pr["merged"] = json!(true);
    pr["state"] = json!("closed");
    for draft in [true, false] {
        pr["draft"] = json!(draft);
        assert!(!contract::needs_draft(&pr).unwrap());
        assert!(
            contract::candidate(
                &pr,
                &json!({"role_name": "maintain"}),
                "owner/repo",
                HEAD,
                "v1.2.3",
                true
            )
            .is_ok()
        );
    }
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
fn refreshed_pr_head_replaces_stale_local_validation_head() {
    let repo = Repo::new();
    let original = repo.main();
    let first = repo.candidate();
    github::git(&["push", "origin", "release/v1.2.3"], &repo.work).unwrap();
    github::git(&["checkout", "main"], &repo.work).unwrap();
    assert!(pr_watch::sync_head("release/v1.2.3", &first, &repo.work).unwrap());
    assert_eq!(
        github::git(&["rev-parse", "HEAD"], &repo.work).unwrap(),
        first
    );
    assert!(!pr_watch::sync_head("release/v1.2.3", &first, &repo.work).unwrap());

    github::git(
        &[
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "--allow-empty",
            "-m",
            "refresh",
        ],
        &repo.work,
    )
    .unwrap();
    let second = github::git(&["rev-parse", "HEAD"], &repo.work).unwrap();
    github::git(
        &[
            "push",
            "origin",
            &format!("{second}:refs/heads/release/v1.2.3"),
        ],
        &repo.work,
    )
    .unwrap();
    github::git(&["reset", "--hard", &original], &repo.work).unwrap();
    assert!(pr_watch::sync_head("release/v1.2.3", &first, &repo.work).unwrap());
    assert_eq!(
        github::git(&["rev-parse", "HEAD"], &repo.work).unwrap(),
        original
    );
    assert!(pr_watch::sync_head("release/v1.2.3", &second, &repo.work).unwrap());
    assert_eq!(
        github::git(&["rev-parse", "HEAD"], &repo.work).unwrap(),
        second
    );
}

#[test]
fn required_checks_follow_the_selected_pr_run_and_verify_exact_head_and_app() {
    let rules = json!([{"type": "required_status_checks", "parameters": {
        "required_status_checks": [{"context": "test-report", "integration_id": 15368}]
    }}]);
    let check = json!({"id": 1, "name": "test-report", "head_sha": HEAD,
        "app": {"id": 15368}, "status": "completed", "conclusion": "success",
        "details_url": "https://github.com/owner/repo/actions/runs/1/job/1"});
    let selected = json!([{"name": "test-report", "bucket": "pass",
        "link": "https://github.com/owner/repo/actions/runs/1/job/1"}]);
    let ready = |checks: &[Value], selected: &Value| {
        super::pr_checks::required_checks(&rules, checks, selected.as_array().unwrap(), HEAD)
    };
    assert!(
        !ready(&[], &selected).unwrap(),
        "a job absent from the PR rollup still blocks promotion"
    );
    assert!(!ready(&[check.clone()], &json!([])).unwrap());
    assert!(ready(&[check.clone()], &selected).unwrap());
    for (pointer, value) in [("/app/id", json!(999)), ("/head_sha", json!(BASE))] {
        let mut wrong = check.clone();
        *wrong.pointer_mut(pointer).unwrap() = value;
        assert!(!ready(&[wrong], &selected).unwrap());
    }
    let mut later_full = check.clone();
    later_full["id"] = json!(2);
    later_full["details_url"] = json!("https://github.com/owner/repo/actions/runs/2/job/2");
    later_full["conclusion"] = json!("skipped");
    assert!(ready(&[check.clone(), later_full.clone()], &selected).unwrap());
    let later_selected = json!([{"name": "test-report", "bucket": "pass",
        "link": "https://github.com/owner/repo/actions/runs/2/job/2"}]);
    assert!(ready(&[check.clone(), later_full], &later_selected).is_err());
    let duplicate = json!([selected[0], selected[0]]);
    assert!(ready(&[check], &duplicate).is_err());
    assert!(super::pr_checks::required_checks(&json!([]), &[], &[], HEAD).is_err());
}
