#![expect(clippy::expect_used, reason = "tests assert by panicking")]
use std::path::{Path, PathBuf};
use std::process::Command;
use vize_s0::{String, ToCompactString};

pub(super) fn env_path(name: &str, repo_root: &Path) -> Option<PathBuf> {
    std::env::var_os(name).map(|value| {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            path
        } else {
            repo_root.join(path)
        }
    })
}

pub(super) fn git_revision(root: &Path) -> String {
    let output = Command::new("git")
        .args([
            "-C",
            root.to_str().expect("UTF-8 fixture path"),
            "rev-parse",
            "HEAD",
        ])
        .output()
        .expect("git should inspect the fixture");
    assert!(output.status.success(), "fixture revision lookup failed");
    std::str::from_utf8(&output.stdout)
        .expect("revision should be UTF-8")
        .trim()
        .to_compact_string()
}
