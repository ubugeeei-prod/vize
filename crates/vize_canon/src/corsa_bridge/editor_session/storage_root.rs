//! Where this user's editor sessions live under the temporary directory.
//!
//! The temporary directory may be shared by every user of the machine. The
//! sessions directory holds generated checker inputs, so it is owned by the
//! user it serves: a directory somebody else made under the well-known name
//! is never adopted, and what this user makes is closed to everyone else.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The first name tried; it is kept whenever it is free or already this user's.
const SHARED_NAME: &str = "vize-canon";

struct Placement {
    base: PathBuf,
    /// The numeric user that owns what this process creates. Unix only.
    user: Option<u32>,
}

fn placement() -> &'static Placement {
    static PLACEMENT: OnceLock<Placement> = OnceLock::new();
    PLACEMENT.get_or_init(|| {
        let temporary = vize_carton::path::canonicalize_non_verbatim(&std::env::temp_dir());
        let user = creating_user(&temporary);
        let root = storage_root(&temporary, user);
        Placement {
            base: root.join("editor").join("sessions"),
            user,
        }
    })
}

/// The directory that holds one `session-*` directory per editor session.
pub(super) fn sessions_base() -> PathBuf {
    placement().base.clone()
}

/// Create the sessions directory, or adopt this user's existing one.
pub(super) fn create_sessions_base() -> std::io::Result<PathBuf> {
    let placement = placement();
    create_owned(&placement.base, placement.user)?;
    Ok(placement.base.clone())
}

fn storage_root(temporary: &Path, user: Option<u32>) -> PathBuf {
    let shared = temporary.join(SHARED_NAME);
    match user {
        Some(user) if !free_or_owned_by(&shared, user) => {
            temporary.join(vize_carton::cstr!("{SHARED_NAME}-{user}").as_str())
        }
        _ => shared,
    }
}

/// The user that owns a directory this process has just made. Reading it from
/// the filesystem is exact for every way a process can acquire an identity.
#[cfg(unix)]
fn creating_user(temporary: &Path) -> Option<u32> {
    use std::os::unix::fs::MetadataExt;
    let probe = tempfile::Builder::new()
        .prefix(".vize-owner-")
        .tempdir_in(temporary)
        .ok()?;
    std::fs::symlink_metadata(probe.path())
        .ok()
        .map(|metadata| metadata.uid())
}

#[cfg(not(unix))]
fn creating_user(_temporary: &Path) -> Option<u32> {
    None
}

#[cfg(unix)]
fn owned_directory(metadata: &std::fs::Metadata, user: u32) -> bool {
    use std::os::unix::fs::MetadataExt;
    metadata.is_dir() && !metadata.file_type().is_symlink() && metadata.uid() == user
}

#[cfg(unix)]
fn free_or_owned_by(path: &Path, user: u32) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => owned_directory(&metadata, user),
        Err(error) => error.kind() == std::io::ErrorKind::NotFound,
    }
}

#[cfg(not(unix))]
fn free_or_owned_by(_path: &Path, _user: u32) -> bool {
    true
}

/// Make `base` and its two parents below the temporary directory, each a real
/// directory of this user that nobody else can enter or change.
#[cfg(unix)]
fn create_owned(base: &Path, user: Option<u32>) -> std::io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    let user = user.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "the temporary directory is not writable",
        )
    })?;
    let mut levels: std::vec::Vec<&Path> = base.ancestors().take(3).collect();
    levels.reverse();
    for level in levels {
        match std::fs::DirBuilder::new().mode(0o700).create(level) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        let metadata = std::fs::symlink_metadata(level)?;
        if !owned_directory(&metadata, user) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "editor mirror storage belongs to another user",
            ));
        }
        if metadata.permissions().mode() & 0o077 != 0 {
            std::fs::set_permissions(level, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn create_owned(base: &Path, _user: Option<u32>) -> std::io::Result<()> {
    std::fs::create_dir_all(base)
}

#[cfg(all(test, unix))]
mod tests {
    use super::{create_owned, creating_user, storage_root};
    use std::os::unix::fs::PermissionsExt;

    fn mode(path: &std::path::Path) -> u32 {
        std::fs::symlink_metadata(path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777
    }

    #[test]
    fn a_free_or_owned_shared_name_is_kept() {
        let temporary = tempfile::tempdir().unwrap();
        let user = creating_user(temporary.path());
        assert!(user.is_some());
        let shared = temporary.path().join("vize-canon");
        assert_eq!(storage_root(temporary.path(), user), shared);
        std::fs::create_dir(&shared).unwrap();
        assert_eq!(storage_root(temporary.path(), user), shared);
    }

    #[test]
    fn a_shared_name_of_another_user_or_kind_is_not_adopted() {
        let temporary = tempfile::tempdir().unwrap();
        let user = creating_user(temporary.path()).unwrap();
        let shared = temporary.path().join("vize-canon");
        let per_user = temporary
            .path()
            .join(vize_carton::cstr!("vize-canon-{user}").as_str());

        // A directory this user owns, seen as another user.
        std::fs::create_dir(&shared).unwrap();
        let other = user.wrapping_add(1);
        assert_eq!(
            storage_root(temporary.path(), Some(other)),
            temporary
                .path()
                .join(vize_carton::cstr!("vize-canon-{other}").as_str())
        );

        // A link is not a directory of this user, wherever it points.
        std::fs::remove_dir(&shared).unwrap();
        let elsewhere = temporary.path().join("elsewhere");
        std::fs::create_dir(&elsewhere).unwrap();
        std::os::unix::fs::symlink(&elsewhere, &shared).unwrap();
        assert_eq!(storage_root(temporary.path(), Some(user)), per_user);
    }

    #[test]
    fn created_levels_are_closed_to_other_users() {
        let temporary = tempfile::tempdir().unwrap();
        let user = creating_user(temporary.path());
        let root = temporary.path().join("vize-canon");
        let base = root.join("editor").join("sessions");

        // An earlier, more open layout is adopted and closed.
        std::fs::create_dir(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o775)).unwrap();
        create_owned(&base, user).unwrap();
        for level in [&root, &root.join("editor"), &base] {
            assert_eq!(mode(level), 0o700);
        }
        create_owned(&base, user).unwrap();
    }

    #[test]
    fn a_linked_level_is_refused() {
        let temporary = tempfile::tempdir().unwrap();
        let user = creating_user(temporary.path());
        let root = temporary.path().join("vize-canon");
        let elsewhere = temporary.path().join("elsewhere");
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(&elsewhere).unwrap();
        std::os::unix::fs::symlink(&elsewhere, root.join("editor")).unwrap();

        let error = create_owned(&root.join("editor").join("sessions"), user).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(std::fs::read_dir(&elsewhere).unwrap().next().is_none());
    }
}
