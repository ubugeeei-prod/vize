//! Stable ownership for materialized Canon projects.
//!
//! Dependency storage is not project identity. Workspace hoisting, package
//! manager stores, fixture symlinks, and CI caches can make distinct project
//! roots share one physical `node_modules` tree. Every mutable Canon artifact
//! therefore lives below a namespace derived only from the canonical project
//! root, and outside `node_modules` and the working tree when Git storage is
//! available so a typecheck cannot change project detection performed by later
//! commands.

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

const PROJECTS_DIR: &str = "projects";
/// Versioned project-key input schema. Changing this deliberately moves every
/// namespace, so the known-vector test below must change in the same review.
const PROJECT_KEY_SCHEMA: &[u8] = b"vize-canon-project-key:v1\0";
const PROJECT_CONTEXT_KEY_SCHEMA: &[u8] = b"vize-canon-project-context-key:v1\0";

/// Return the stable, project-owned namespace for Canon's mutable artifacts.
///
/// The full SHA-256 digest keeps the namespace a single portable path
/// component without lossy path conversion or platform path-length growth.
/// The input bytes preserve the operating system's exact path identity, so two
/// non-UTF-8 Unix roots cannot collapse through `to_string_lossy`.
pub fn project_virtual_root(project_root: &Path) -> PathBuf {
    let project_root = vize_carton::path::canonicalize_non_verbatim(project_root);
    let project_key = project_key(&project_root);
    project_virtual_root_for_key(&project_root, &project_key)
}

/// Return a namespace shared only by editor snapshots with the same effective
/// config and generation options. A solution shell may own multiple referenced
/// projects whose compiler options are intentionally incompatible; they must
/// never overwrite one mirror `tsconfig.json` merely because their source tree
/// has the same root.
pub(super) fn project_virtual_root_with_identity(
    storage_root: &Path,
    project_root: &Path,
    identity: u64,
) -> PathBuf {
    let project_root = vize_carton::path::canonicalize_non_verbatim(project_root);
    let mut digest = Sha256::new();
    digest.update(PROJECT_CONTEXT_KEY_SCHEMA);
    update_path_digest(&mut digest, &project_root);
    digest.update(identity.to_le_bytes());
    let project_key = encode_digest(digest.finalize());
    // TypeScript's editor project service classifies every path below a
    // `node_modules` segment as an external library. The bridge supplies a
    // session-private temp root so unsaved overlay bytes never cross native
    // project lifetimes; this digest scopes incompatible configs inside it.
    vize_carton::path::canonicalize_non_verbatim(storage_root)
        .join(PROJECTS_DIR)
        .join(project_key.as_str())
}

fn project_virtual_root_for_key(project_root: &Path, project_key: &vize_carton::String) -> PathBuf {
    project_canon_storage_root(project_root)
        .join("canon")
        .join(PROJECTS_DIR)
        .join(project_key.as_str())
}

fn project_canon_storage_root(project_root: &Path) -> PathBuf {
    git_storage_root(project_root).unwrap_or_else(|| project_root.join(".vize"))
}

fn git_storage_root(project_root: &Path) -> Option<PathBuf> {
    let dot_git = project_root.join(".git");
    let metadata = fs::metadata(&dot_git).ok()?;
    if metadata.is_dir() {
        return Some(dot_git.join("vize"));
    }
    if metadata.is_file() {
        let gitdir = parse_gitdir_file(&fs::read_to_string(&dot_git).ok()?)?;
        let gitdir = if gitdir.is_absolute() {
            gitdir
        } else {
            project_root.join(gitdir)
        };
        return Some(vize_carton::path::canonicalize_non_verbatim(&gitdir).join("vize"));
    }
    None
}

fn parse_gitdir_file(contents: &str) -> Option<PathBuf> {
    let value = contents.trim().strip_prefix("gitdir:")?.trim();
    (!value.is_empty()).then(|| PathBuf::from(value))
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
    use super::{
        project_key, project_virtual_lock_paths, project_virtual_root,
        project_virtual_root_with_identity,
    };
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

    #[test]
    fn batch_namespace_stays_outside_node_modules() {
        let root = project_virtual_root(Path::new("/workspace/project"));
        assert!(
            root.components()
                .all(|component| component.as_os_str() != "node_modules")
        );
        assert!(root.starts_with(Path::new("/workspace/project/.vize/canon")));
    }

    #[test]
    fn git_checkout_namespace_stays_outside_the_working_tree() {
        let project = tempfile::tempdir().unwrap();
        std::fs::create_dir(project.path().join(".git")).unwrap();

        let root = project_virtual_root(project.path());
        let expected =
            vize_carton::path::canonicalize_non_verbatim(project.path()).join(".git/vize/canon");

        assert!(root.starts_with(expected));
        assert!(
            root.components()
                .all(|component| component.as_os_str() != "node_modules")
        );
    }

    #[test]
    fn git_worktree_namespace_uses_the_resolved_gitdir() {
        let holder = tempfile::tempdir().unwrap();
        let project = holder.path().join("worktree");
        let gitdir = holder.path().join("git/worktrees/worktree");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&gitdir).unwrap();
        std::fs::write(project.join(".git"), "gitdir: ../git/worktrees/worktree\n").unwrap();

        let root = project_virtual_root(&project);
        let expected = vize_carton::path::canonicalize_non_verbatim(&gitdir).join("vize/canon");

        assert!(root.starts_with(expected));
    }

    #[test]
    fn editor_namespace_stays_outside_node_modules() {
        let storage = std::env::temp_dir().join("private-editor-session");
        let root = project_virtual_root_with_identity(&storage, Path::new("/workspace/project"), 7);
        let storage = vize_carton::path::canonicalize_non_verbatim(&storage).join("projects");
        assert!(root.starts_with(storage));
        assert!(
            root.components()
                .all(|component| component.as_os_str() != "node_modules")
        );
        assert_eq!(root.file_name().unwrap().to_string_lossy().len(), 64);
    }

    #[test]
    fn editor_namespace_is_session_scoped_even_for_the_same_project_identity() {
        let project = Path::new("/workspace/project");
        let first =
            project_virtual_root_with_identity(Path::new("/private/session-first"), project, 7);
        let second =
            project_virtual_root_with_identity(Path::new("/private/session-second"), project, 7);
        assert_ne!(first, second);
        assert_eq!(first.file_name(), second.file_name());
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
