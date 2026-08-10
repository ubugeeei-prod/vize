//! Stable ownership for materialized Canon projects.
//!
//! Dependency storage is not project identity. Workspace hoisting, package
//! manager stores, fixture symlinks, and CI caches can make distinct project
//! roots share one physical `node_modules` tree. Every mutable Canon artifact
//! therefore lives below a namespace derived only from the canonical project
//! root.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

const PROJECTS_DIR: &str = "projects";
/// Versioned project-key input schema. Changing this deliberately moves every
/// namespace, so the known-vector test below must change in the same review.
const PROJECT_KEY_SCHEMA: &[u8] = b"vize-canon-project-key:v1\0";

/// Return the stable, project-owned namespace for Canon's mutable artifacts.
///
/// The full SHA-256 digest keeps the namespace a single portable path
/// component without lossy path conversion or platform path-length growth.
/// The input bytes preserve the operating system's exact path identity, so two
/// non-UTF-8 Unix roots cannot collapse through `to_string_lossy`.
pub fn project_virtual_root(project_root: &Path) -> PathBuf {
    let project_root = vize_carton::path::canonicalize_non_verbatim(project_root);
    let project_key = project_key(&project_root);
    // Corsa compares workspace and document paths by their filesystem
    // identity. Resolve an existing dependency-store symlink once here so the
    // workspace root and every URI share one spelling; the project key still
    // comes exclusively from the canonical source root above.
    vize_carton::path::canonicalize_non_verbatim(&project_root.join("node_modules"))
        .join(".vize")
        .join("canon")
        .join(PROJECTS_DIR)
        .join(project_key.as_str())
}

/// Lock files owned by the current project namespace.
///
/// Both spellings are returned so `vize clean` can remove the active platform
/// lock and a stale lock left by the other spelling after a cross-platform
/// cache restore.
pub fn project_virtual_lock_paths(project_root: &Path) -> [PathBuf; 2] {
    let virtual_root = project_virtual_root(project_root);
    [
        virtual_root.with_extension("lock"),
        virtual_root.with_extension("materialize.lock"),
    ]
}

fn project_key(project_root: &Path) -> vize_carton::String {
    let mut digest = Sha256::new();
    digest.update(PROJECT_KEY_SCHEMA);
    update_path_digest(&mut digest, project_root);
    encode_digest(digest.finalize())
}

fn encode_digest(digest: impl AsRef<[u8]>) -> vize_carton::String {
    let digest = digest.as_ref();
    let mut encoded = vize_carton::String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

#[cfg(unix)]
fn update_path_digest(digest: &mut Sha256, path: &Path) {
    use std::os::unix::ffi::OsStrExt;
    digest.update(path.as_os_str().as_bytes());
}

#[cfg(windows)]
fn update_path_digest(digest: &mut Sha256, path: &Path) {
    use std::os::windows::ffi::OsStrExt;
    for unit in path.as_os_str().encode_wide() {
        digest.update(unit.to_le_bytes());
    }
}

#[cfg(not(any(unix, windows)))]
fn update_path_digest(digest: &mut Sha256, path: &Path) {
    digest.update(path.as_os_str().to_string_lossy().as_bytes());
}

#[cfg(test)]
mod tests {
    use super::{project_key, project_virtual_lock_paths, project_virtual_root};
    use std::path::Path;

    #[test]
    fn project_namespace_is_stable_and_root_specific() {
        let first = project_virtual_root(Path::new("/workspace/first"));
        let repeated = project_virtual_root(Path::new("/workspace/first"));
        let second = project_virtual_root(Path::new("/workspace/second"));

        assert_eq!(first, repeated);
        assert_ne!(first, second);
        assert_eq!(first.parent().unwrap().file_name().unwrap(), "projects");
        let key = first.file_name().unwrap().to_str().unwrap();
        assert_eq!(key.len(), 64);
        assert!(key.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }

    #[cfg(unix)]
    #[test]
    fn unix_project_key_schema_has_a_reviewed_known_vector() {
        assert_eq!(
            project_key(Path::new("/workspace/project")),
            "75f1ce1c5f19d5351a5db6aa00d2c1365f245368acefdcd5d8f119f466cd29e1"
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_project_key_schema_has_a_reviewed_known_vector() {
        assert_eq!(
            project_key(Path::new(r"C:\workspace\project")),
            "5513ca835150777804e200d8754b63e464fc44787a318b93e4d4da056ec7dfeb"
        );
    }

    #[test]
    fn lock_names_stay_inside_the_project_namespace_parent() {
        let virtual_root = project_virtual_root(Path::new("/workspace/project"));
        let [legacy, windows] = project_virtual_lock_paths(Path::new("/workspace/project"));
        assert_eq!(legacy.parent(), virtual_root.parent());
        assert_eq!(windows.parent(), virtual_root.parent());
        assert_eq!(legacy.file_stem(), virtual_root.file_name());
        assert!(windows.to_string_lossy().ends_with(".materialize.lock"));
    }
}
