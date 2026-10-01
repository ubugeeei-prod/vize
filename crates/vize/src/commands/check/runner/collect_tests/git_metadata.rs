//! Git metadata is never checker input.

use super::*;

#[test]
fn check_and_vue_discovery_never_select_git_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let source = write_file(root, "src/App.vue", "<template><div/></template>");
    let metadata = write_file(root, ".git/worktrees/cache/App.vue", "invalid");
    for input in [
        root.display().to_string(),
        root.join("**/*.vue").display().to_string(),
    ] {
        assert_eq!(
            collect_check_files(&[input.clone()], true),
            vec![source.clone()]
        );
        assert_eq!(collect_vue_files(&[input]), vec![source.clone()]);
    }
    for input in [
        metadata.display().to_string(),
        root.join(".git").display().to_string(),
        root.join("**/.git/**/*.vue").display().to_string(),
    ] {
        assert!(collect_check_files(&[input.clone()], true).is_empty());
        assert!(collect_vue_files(&[input]).is_empty());
    }
}
