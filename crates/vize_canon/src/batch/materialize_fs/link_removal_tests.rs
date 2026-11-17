//! Stale package links in the project mirror must be detached even after the
//! package they pointed at is gone (#13088). On Windows the mirror links are
//! directory junctions (or directory symlinks with Developer Mode): a dangling
//! one answers `DeleteFileW` with `ERROR_ACCESS_DENIED`, so it must be removed
//! as a directory entry, from facts about the link itself.

use std::path::{Path, PathBuf};

use vize_carton::FxHashSet;

use super::{Removal, prune_unexpected_entries, removal_error, removal_for, remove_path};
use crate::batch::runtime_deps::symlink_package_dir;

fn package(root: &Path, name: &str) -> PathBuf {
    let package = root.join("store").join(name);
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(package.join("package.json"), "{}").unwrap();
    package
}

fn entry_kind(path: &Path) -> Removal {
    removal_for(std::fs::symlink_metadata(path).unwrap().file_type())
}

fn missing(path: &Path) -> Option<std::io::ErrorKind> {
    std::fs::symlink_metadata(path)
        .map_err(|error| error.kind())
        .err()
}

/// What the platform must report for a package link, live or dangling.
fn expected_link_removal() -> Removal {
    if cfg!(windows) {
        Removal::DirectoryLink
    } else {
        Removal::Leaf
    }
}

#[test]
fn a_dangling_package_link_is_classified_from_the_link_itself() {
    let temp = tempfile::tempdir().unwrap();
    let target = package(temp.path(), "extra");
    let link = temp.path().join("mirror/node_modules/extra");
    symlink_package_dir(&target, &link).unwrap();
    assert_eq!(entry_kind(&link), expected_link_removal());

    std::fs::remove_dir_all(&target).unwrap();

    // The target is gone: nothing may depend on following the link.
    assert!(std::fs::metadata(&link).is_err());
    assert_eq!(entry_kind(&link), expected_link_removal());
    assert_eq!(entry_kind(&temp.path().join("store")), Removal::Tree);
}

#[test]
fn removing_a_dangling_package_link_detaches_only_the_link() {
    let temp = tempfile::tempdir().unwrap();
    let target = package(temp.path(), "extra");
    let sibling = package(temp.path(), "kept");
    let link = temp.path().join("mirror/node_modules/extra");
    symlink_package_dir(&target, &link).unwrap();
    std::fs::remove_dir_all(&target).unwrap();

    remove_path(&link).unwrap();
    // Idempotent: a second sync sees nothing left to remove.
    remove_path(&link).unwrap();

    assert_eq!(missing(&link), Some(std::io::ErrorKind::NotFound));
    assert_eq!(
        std::fs::read_to_string(sibling.join("package.json")).unwrap(),
        "{}"
    );
}

#[test]
fn mirror_prune_drops_a_stale_dangling_link_and_keeps_live_links() {
    let temp = tempfile::tempdir().unwrap();
    let removed = package(temp.path(), "extra");
    let live = package(temp.path(), "vue");
    let node_modules = temp.path().join("mirror/node_modules");
    let stale_link = node_modules.join("extra");
    let live_link = node_modules.join("vue");
    symlink_package_dir(&removed, &stale_link).unwrap();
    symlink_package_dir(&live, &live_link).unwrap();
    // The package is uninstalled between two `vize check` runs.
    std::fs::remove_dir_all(&removed).unwrap();

    prune_unexpected_entries(
        &node_modules,
        &FxHashSet::default(),
        std::slice::from_ref(&live_link),
    )
    .unwrap();

    assert_eq!(missing(&stale_link), Some(std::io::ErrorKind::NotFound));
    assert_eq!(
        std::fs::read_to_string(live_link.join("package.json")).unwrap(),
        "{}"
    );

    // Reinstalling the package later links it again under the same name.
    let recreated = package(temp.path(), "extra");
    symlink_package_dir(&recreated, &stale_link).unwrap();
    assert_eq!(
        std::fs::read_to_string(stale_link.join("package.json")).unwrap(),
        "{}"
    );
}

#[test]
fn relinking_over_a_dangling_link_replaces_it() {
    let temp = tempfile::tempdir().unwrap();
    let removed = package(temp.path(), "old");
    let replacement = package(temp.path(), "new");
    let link = temp.path().join("mirror/node_modules/pkg");
    symlink_package_dir(&removed, &link).unwrap();
    std::fs::remove_dir_all(&removed).unwrap();

    symlink_package_dir(&replacement, &link).unwrap();

    assert_eq!(
        std::fs::read_to_string(link.join("package.json")).unwrap(),
        "{}"
    );
    assert_eq!(
        vize_carton::path::canonicalize_non_verbatim(&link),
        vize_carton::path::canonicalize_non_verbatim(&replacement)
    );
}

#[test]
fn removal_errors_name_the_cache_entry_and_keep_the_os_kind() {
    let path = Path::new("mirror").join("node_modules").join("extra");
    let error = removal_error(
        &path,
        std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access is denied."),
    );
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    let expected = vize_carton::cstr!(
        "failed to remove cache entry {}: Access is denied.",
        path.display()
    );
    assert_eq!(error.to_string().as_str(), expected.as_str());
}

/// `symlink_package_dir` prefers a directory symlink and only falls back to a
/// junction, so a runner that may create symlinks never exercises the
/// junction that stock Windows installs get (#13088). Create one directly.
#[cfg(windows)]
#[test]
fn a_dangling_junction_is_removed_without_touching_the_mirror() {
    let temp = tempfile::tempdir().unwrap();
    let target = package(temp.path(), "extra");
    let live = package(temp.path(), "vue");
    let node_modules = temp.path().join("mirror/node_modules");
    std::fs::create_dir_all(&node_modules).unwrap();
    let stale = node_modules.join("extra");
    let kept = node_modules.join("vue");
    junction::create(&target, &stale).unwrap();
    junction::create(&live, &kept).unwrap();
    assert_eq!(entry_kind(&stale), Removal::DirectoryLink);

    std::fs::remove_dir_all(&target).unwrap();
    assert!(std::fs::metadata(&stale).is_err());
    assert_eq!(entry_kind(&stale), Removal::DirectoryLink);
    // The pre-fix fallback: unlinking a dangling junction as a file.
    assert_eq!(
        std::fs::remove_file(&stale)
            .map_err(|error| error.kind())
            .err(),
        Some(std::io::ErrorKind::PermissionDenied)
    );

    prune_unexpected_entries(
        &node_modules,
        &FxHashSet::default(),
        std::slice::from_ref(&kept),
    )
    .unwrap();

    assert_eq!(missing(&stale), Some(std::io::ErrorKind::NotFound));
    assert_eq!(
        std::fs::read_to_string(kept.join("package.json")).unwrap(),
        "{}"
    );
    assert_eq!(
        std::fs::read_to_string(live.join("package.json")).unwrap(),
        "{}"
    );
}
