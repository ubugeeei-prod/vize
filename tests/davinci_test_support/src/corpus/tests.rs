use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use vize_s0::{String, cstr};

use super::{CorpusPreflightError, CorpusScope, prepare_corpus_root};

struct TempRepo {
    root: PathBuf,
}

impl TempRepo {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must move forward")
            .as_nanos();
        let root = std::env::temp_dir()
            .join(cstr!("vize-davinci-corpus-{label}-{}-{nonce}", std::process::id()).as_str());
        fs::create_dir_all(&root).expect("temp repo dir");
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.name", "Fixture"]);
        git(&root, &["config", "user.email", "fixture@example.com"]);
        fs::create_dir_all(root.join("crates/pkg")).expect("package dir");
        Self { root }
    }

    fn package_dir(&self) -> PathBuf {
        self.root.join("crates/pkg")
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git must run");
    assert!(
        output.status.success(),
        "git {} failed:\n{}",
        args.join(" "),
        output_text(&output.stderr)
    );
    output_text(&output.stdout)
}

fn commit_fixture(root: &Path, rel_path: &str, marker: &str) -> String {
    let path = root.join(rel_path);
    fs::create_dir_all(&path).expect("fixture dir");
    git(&path, &["init", "-q"]);
    git(&path, &["config", "user.name", "Fixture"]);
    git(&path, &["config", "user.email", "fixture@example.com"]);
    let readme = cstr!("{marker}\n");
    fs::write(path.join("README.md"), readme.as_bytes()).expect("fixture readme");
    git(&path, &["add", "README.md"]);
    git(&path, &["commit", "-qm", marker]);
    git(&path, &["rev-parse", "HEAD"])
}

fn pin_gitlink(root: &Path, rel_path: &str, revision: &str) {
    let gitmodules = root.join(".gitmodules");
    let mut content = fs::read_to_string(&gitmodules).unwrap_or_default();
    content.push_str(cstr!(
        "[submodule \"{rel_path}\"]\n\tpath = {rel_path}\n\turl = https://example.com/{rel_path}.git\n"
    ).as_str());
    fs::write(&gitmodules, content).expect("gitmodules");
    git(root, &["add", ".gitmodules"]);
    let cacheinfo = cstr!("160000,{revision},{rel_path}");
    git(
        root,
        &["update-index", "--add", "--cacheinfo", cacheinfo.as_str()],
    );
}

fn unpin_gitlink(root: &Path, rel_path: &str) {
    git(root, &["update-index", "--force-remove", rel_path]);
}

fn output_text(bytes: &[u8]) -> String {
    match core::str::from_utf8(bytes) {
        Ok(text) => text.trim().into(),
        Err(_) => "<non-utf8 output>".into(),
    }
}

#[test]
fn external_roots_are_smoke_shards_not_closure_evidence() {
    let repo = TempRepo::new("external");
    let external = repo.root.join("external/tests/_fixtures/_git");
    fs::create_dir_all(&external).expect("external corpus root");

    let root = prepare_corpus_root(&external, repo.package_dir()).expect("external root");

    assert_eq!(root.scope(), CorpusScope::SmokeShard);
    assert!(!root.closure_evidence());
    assert_eq!(root.path(), external);
}

#[test]
fn canonical_partial_hydration_fails_before_the_sweep() {
    let repo = TempRepo::new("partial");
    let revision = commit_fixture(&repo.root, "tests/_fixtures/_git/clean-0", "clean-0");
    pin_gitlink(&repo.root, "tests/_fixtures/_git/clean-0", &revision);
    for index in 0..5 {
        if index == 0 {
            continue;
        }
        let rel = cstr!("tests/_fixtures/_git/clean-{index}");
        let marker = cstr!("clean-{index}");
        let revision = commit_fixture(&repo.root, rel.as_str(), marker.as_str());
        pin_gitlink(&repo.root, rel.as_str(), &revision);
    }
    for index in 0..141 {
        let rel = cstr!("tests/_fixtures/_git/missing-{index:03}");
        pin_gitlink(&repo.root, rel.as_str(), &revision);
    }

    let error = prepare_corpus_root("tests/_fixtures/_git", repo.package_dir())
        .expect_err("partial canonical root must fail");
    let message = cstr!("{error}");

    assert!(message.contains("indexed gitlinks: 146"));
    assert!(message.contains("clean submodules: 5"));
    assert!(message.contains("missing submodules: 141"));
    assert!(message.contains("hydrate with:"));
    assert!(message.contains("partial fixture trees are refused before collecting .vue files"));
}

#[test]
fn canonical_preflight_reports_drift_empty_spaces_and_inventory_mismatches() {
    let repo = TempRepo::new("mixed");
    let spaced = "tests/_fixtures/_git/path with spaces caf\u{e9}";
    let expected = commit_fixture(&repo.root, spaced, "spaces");
    pin_gitlink(&repo.root, spaced, &expected);

    let actual = commit_fixture(&repo.root, "tests/_fixtures/_git/drifted", "actual");
    let other = commit_fixture(&repo.root, "other", "expected");
    pin_gitlink(&repo.root, "tests/_fixtures/_git/drifted", &other);
    assert_ne!(actual, other);

    fs::create_dir_all(repo.root.join("tests/_fixtures/_git/empty")).expect("empty fixture");
    pin_gitlink(&repo.root, "tests/_fixtures/_git/empty", &expected);

    let surplus = "tests/_fixtures/_git/surplus";
    let surplus_revision = commit_fixture(&repo.root, surplus, "surplus");
    pin_gitlink(&repo.root, surplus, &surplus_revision);
    unpin_gitlink(&repo.root, surplus);

    let mut error = match prepare_corpus_root("tests/_fixtures/_git", repo.package_dir())
        .expect_err("mixed canonical root must fail")
    {
        CorpusPreflightError::Hydration(report) => report,
        error => panic!("expected hydration error, got {error}"),
    };
    error.drifted.sort();
    error.missing.sort();

    assert_eq!(error.clean, 1);
    assert!(error.drifted.iter().any(|row| row.contains("drifted")));
    assert!(error.missing.iter().any(|row| row.contains("empty")));
    assert!(
        error
            .inventory_mismatch
            .iter()
            .any(|row| row.contains("surplus"))
    );
}
