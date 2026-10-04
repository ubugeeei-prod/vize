//! Authored fixture path checks and the unchanged real Vue link.

use std::path::{Path, PathBuf};

pub(crate) fn safe_relative(input: &str) -> PathBuf {
    let path = PathBuf::from(input);
    assert!(
        path.components()
            .all(|part| matches!(part, std::path::Component::Normal(_)))
    );
    assert!(!path.as_os_str().is_empty());
    path
}

pub(crate) fn link_vue(source: &Path, destination: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, destination).expect("the real Vue package must link");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, destination).expect("the real Vue package must link");
}
