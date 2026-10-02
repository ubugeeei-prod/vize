//! Broad formatting never parses or rewrites repository metadata.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "CLI fixtures use std strings"
)]

use std::{fs, process::Command};

const SOURCE: &str = include_str!("fixtures/git-metadata/source.vue.txt");
const METADATA: &str = include_str!("fixtures/git-metadata/metadata.vue.txt");

#[test]
fn fmt_write_prunes_git_metadata_for_relative_and_absolute_globs() {
    for absolute in [false, true] {
        let project = tempfile::tempdir().unwrap();
        let root = project.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".github")).unwrap();
        fs::create_dir_all(root.join(".git/worktrees/cache")).unwrap();
        fs::write(root.join("src/App.vue"), SOURCE).unwrap();
        fs::write(root.join(".github/App.vue"), SOURCE).unwrap();
        let metadata = root.join(".git/worktrees/cache/Snapshot.vue");
        fs::write(&metadata, METADATA).unwrap();
        let pattern = if absolute {
            root.join("**/*.vue").display().to_string()
        } else {
            "**/*.vue".to_owned()
        };
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root)
            .args(["fmt", "--write", &pattern])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read_to_string(metadata).unwrap(), METADATA);
        assert_ne!(
            fs::read_to_string(root.join("src/App.vue")).unwrap(),
            SOURCE
        );
        assert_ne!(
            fs::read_to_string(root.join(".github/App.vue")).unwrap(),
            SOURCE
        );
    }
}
