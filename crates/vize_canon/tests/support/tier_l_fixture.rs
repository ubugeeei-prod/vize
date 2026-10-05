#![expect(clippy::expect_used, reason = "tests assert by panicking")]
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use vize_l0::{String, ToCompactString};

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

pub(super) const FIXTURE_ID: &str = "vue-vben-admin";
pub(super) const TIER_L_VUE_FILES: usize = 500;
pub(super) const INJECTED_FILE: &str = "apps/web-antd/src/__vize_batch_incremental_oracle__.vue";
pub(super) const CLEAN_SOURCE: &str = r#"<script setup lang="ts">
const __vizeBatchIncrementalOracle: number = 1;
</script>

<template><span>{{ __vizeBatchIncrementalOracle }}</span></template>
"#;
pub(super) const BROKEN_SOURCE: &str = r#"<script setup lang="ts">
const __vizeBatchIncrementalOracle: number = 'broken';
</script>

<template><span>{{ __vizeBatchIncrementalOracle }}</span></template>
"#;

pub(super) struct InjectedFixtureFile(PathBuf);

impl InjectedFixtureFile {
    pub(super) fn create(path: PathBuf) -> Self {
        assert!(!path.exists(), "injected fixture path must start absent");
        fs::write(&path, CLEAN_SOURCE).expect("clean fixture source should write");
        Self(path)
    }

    pub(super) fn write(&self, source: &str) {
        fs::write(&self.0, source).expect("fixture patch should write");
    }
}

impl Drop for InjectedFixtureFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub(super) fn collect_vue_paths(fixture_root: &Path) -> Vec<PathBuf> {
    let mut paths = ["apps", "packages", "playground"]
        .into_iter()
        .flat_map(|relative| walkdir::WalkDir::new(fixture_root.join(relative)))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("vue"))
        .collect::<Vec<_>>();
    paths.sort();
    assert!(
        paths.len() > TIER_L_VUE_FILES,
        "pinned fixture fell below Tier-L scale"
    );
    paths.truncate(TIER_L_VUE_FILES);
    assert!(paths.iter().any(|path| path.ends_with(INJECTED_FILE)));
    paths
}
