//! Shared scratch directories, which ancestor walks never look inside.

use std::path::Path;

/// Whether `dir` is a shared scratch directory such as `/tmp`: anyone may
/// create entries in it (world-writable) and only owners may remove them
/// (sticky). What another user planted there is not part of any project, so
/// ancestor walks for configuration and tools never look inside one.
///
/// A world-writable directory without the sticky bit is not matched: mounts
/// that report every directory as `0777` (a Windows drive under WSL, for
/// example) are not shared scratch space.
#[cfg(unix)]
pub(crate) fn is_shared_scratch_directory(dir: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    const STICKY_AND_WORLD_WRITABLE: u32 = 0o1002;
    std::fs::metadata(dir).is_ok_and(|metadata| {
        metadata.is_dir()
            && metadata.permissions().mode() & STICKY_AND_WORLD_WRITABLE
                == STICKY_AND_WORLD_WRITABLE
    })
}

/// Windows has no sticky directories; ancestor walks keep their behaviour.
#[cfg(not(unix))]
pub(crate) fn is_shared_scratch_directory(_dir: &Path) -> bool {
    false
}

#[cfg(all(test, unix))]
mod tests {
    use super::is_shared_scratch_directory;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn only_sticky_world_writable_directories_are_shared_scratch() {
        let dir = tempfile::tempdir().unwrap();
        let set = |mode: u32| {
            std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(mode)).unwrap();
            is_shared_scratch_directory(dir.path())
        };
        assert!(set(0o1777));
        assert!(set(0o1733));
        assert!(!set(0o777));
        assert!(!set(0o1755));
        assert!(!set(0o700));
        let file = dir.path().join("file");
        std::fs::write(&file, "").unwrap();
        assert!(!is_shared_scratch_directory(&file));
        assert!(!is_shared_scratch_directory(&dir.path().join("missing")));
    }
}
