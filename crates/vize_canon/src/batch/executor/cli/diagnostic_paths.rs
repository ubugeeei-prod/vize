//! Turn the file path of a Corsa CLI diagnostic line into the virtual path the
//! project's file index is keyed by.
//!
//! The CLI is started with its working directory set to the virtual root, and
//! it prints relative diagnostic paths against that directory. A process only
//! ever observes the *resolved* spelling of its working directory, though: a
//! project whose `node_modules` is a symlink has a virtual root that traverses
//! the link, so the paths the CLI prints resolve against the link target while
//! the project indexes files under the through-link spelling. Resolving those
//! paths against the through-link root alone lands outside the virtual project,
//! the lookup misses, and the diagnostic is dropped without a trace (#3320).

use std::path::{Path, PathBuf};

use vize_carton::path::canonicalize_non_verbatim;

/// Resolve the path of a Corsa CLI diagnostic to its virtual path, spelled the
/// way the project spells `virtual_root` so `find_by_virtual` can match it.
pub(super) fn normalize_cli_path(path: &str, virtual_root: &Path) -> PathBuf {
    let reported = Path::new(path);
    if reported.is_absolute() {
        return anchor_in_virtual_root(normalize_path_lexically(reported), virtual_root);
    }

    let canonical_root = canonicalize_non_verbatim(virtual_root);
    // The working directory the CLI printed against is the resolved spelling of
    // the virtual root; it coincides with the through-link spelling whenever no
    // symlink is involved, which is the common case and stays single-candidate.
    let mut bases = Vec::with_capacity(2);
    bases.push(virtual_root);
    if canonical_root != virtual_root {
        bases.push(canonical_root.as_path());
    }

    // Prefer whichever base places the file inside the virtual project: that is
    // the only spelling the project's index can be keyed by, so an ambiguous
    // relative path must not resolve to an unrelated file outside the project.
    for base in &bases {
        let candidate = normalize_path_lexically(&base.join(reported));
        if let Some(anchored) =
            virtual_root_relative(&candidate, virtual_root, canonical_root.as_path())
        {
            return anchored;
        }
    }

    // A real file outside the virtual project (an ancestor `.d.ts`, a package
    // under `node_modules`): report its resolved path.
    for base in &bases {
        let candidate = normalize_path_lexically(&base.join(reported));
        if candidate.exists() {
            return canonicalize_non_verbatim(&candidate);
        }
    }

    normalize_path_lexically(&virtual_root.join(reported))
}

/// Re-express an existing path inside the virtual project the way the project
/// spells `virtual_root`; a path that exists elsewhere keeps its resolved
/// spelling, and a path that does not exist is left exactly as given.
fn anchor_in_virtual_root(path: PathBuf, virtual_root: &Path) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let canonical_root = canonicalize_non_verbatim(virtual_root);
    if let Some(anchored) = virtual_root_relative(&path, virtual_root, canonical_root.as_path()) {
        return anchored;
    }
    canonicalize_non_verbatim(&path)
}

/// `Some(virtual_root/<relative>)` when `path` exists and resolves inside the
/// virtual project, `None` otherwise.
fn virtual_root_relative(
    path: &Path,
    virtual_root: &Path,
    canonical_root: &Path,
) -> Option<PathBuf> {
    if !path.exists() {
        return None;
    }
    let canonical_path = canonicalize_non_verbatim(path);
    let relative = canonical_path.strip_prefix(canonical_root).ok()?;
    Some(virtual_root.join(relative))
}

pub(super) fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() && !normalized.has_root() {
                    normalized.push(component.as_os_str());
                }
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::{normalize_cli_path, normalize_path_lexically};
    use std::path::{Path, PathBuf};

    #[test]
    fn normalizes_lexical_parent_segments() {
        assert_eq!(
            normalize_path_lexically(Path::new("/a/b/../c/./d.ts")),
            PathBuf::from("/a/c/d.ts")
        );
    }

    #[test]
    fn keeps_missing_paths_untouched() {
        let virtual_root = crate::batch::project_virtual_root(Path::new("/does/not/exist"));
        assert_eq!(
            normalize_cli_path("src/App.vue.ts", &virtual_root),
            virtual_root.join("src/App.vue.ts")
        );
    }

    #[test]
    fn resolves_relative_paths_against_the_virtual_root() {
        let root = temp_dir("cli-diagnostic-paths-plain");
        let virtual_root = crate::batch::project_virtual_root(&root);
        write_virtual_file(&virtual_root, "src/App.vue.ts");

        assert_eq!(
            normalize_cli_path("src/App.vue.ts", &virtual_root),
            virtual_root.join("src/App.vue.ts")
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Regression for #3320: the Corsa CLI prints relative diagnostic paths
    /// against the resolved spelling of its working directory. With a
    /// symlinked `node_modules` that spelling is the link target, so the
    /// printed path climbs out of the target and back down the through-link
    /// path. Resolving it must still land on the virtual path the project
    /// registered, or the diagnostic is silently dropped.
    #[cfg(unix)]
    #[test]
    fn resolves_relative_paths_reported_against_a_symlinked_virtual_root() {
        let root = temp_dir("cli-diagnostic-paths-symlink");
        let project_root = root.join("app");
        let store = root.join("store");
        std::fs::create_dir_all(&project_root).unwrap();
        std::fs::create_dir_all(&store).unwrap();
        std::os::unix::fs::symlink(&store, project_root.join("node_modules")).unwrap();

        let physical_virtual_root = store.join(".vize/canon/projects/project-key");
        let through_link_virtual_root =
            project_root.join("node_modules/.vize/canon/projects/project-key");
        write_virtual_file(&through_link_virtual_root, "src/App.vue.ts");

        // What the CLI prints from the link target: up out of
        // the physical virtual root to the shared parent, then down the
        // through-link project path.
        let reported = relative_path(
            &physical_virtual_root,
            &through_link_virtual_root.join("src/App.vue.ts"),
        );
        assert_eq!(
            normalize_cli_path(reported.to_str().unwrap(), &through_link_virtual_root),
            through_link_virtual_root.join("src/App.vue.ts"),
            "a diagnostic reported against the link target must map back to the registered virtual path"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn keeps_files_outside_the_virtual_project_at_their_resolved_path() {
        let root = temp_dir("cli-diagnostic-paths-outside");
        let project_root = root.join("app");
        let virtual_root = crate::batch::project_virtual_root(&project_root);
        std::fs::create_dir_all(&virtual_root).unwrap();
        std::fs::create_dir_all(project_root.join("types")).unwrap();
        std::fs::write(project_root.join("types/globals.d.ts"), "export {};\n").unwrap();

        let reported = relative_path(&virtual_root, &project_root.join("types/globals.d.ts"));
        let resolved = normalize_cli_path(reported.to_str().unwrap(), &virtual_root);
        assert!(
            resolved.ends_with("types/globals.d.ts"),
            "unexpected resolved path: {resolved:?}"
        );
        assert!(
            resolved.exists(),
            "resolved path should exist: {resolved:?}"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    fn temp_dir(name: &str) -> PathBuf {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(vize_carton::cstr!(
            "vize-{name}-{}-{id}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_virtual_file(virtual_root: &Path, relative: &str) {
        let path = virtual_root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "export {};\n").unwrap();
    }

    fn relative_path(from_dir: &Path, to: &Path) -> PathBuf {
        let from = normalize_path_lexically(from_dir);
        let to = normalize_path_lexically(to);
        let from_components: Vec<_> = from.components().collect();
        let to_components: Vec<_> = to.components().collect();
        let common = from_components
            .iter()
            .zip(&to_components)
            .take_while(|(left, right)| left == right)
            .count();
        let mut relative = PathBuf::new();
        for _ in common..from_components.len() {
            relative.push("..");
        }
        for component in &to_components[common..] {
            relative.push(component.as_os_str());
        }
        relative
    }
}
