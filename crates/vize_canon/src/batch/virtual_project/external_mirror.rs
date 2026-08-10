//! Mirror subtree for files outside the project root (#3887).
//!
//! The mirror reproduces project-root-relative paths, which an out-of-root
//! file — a workspace-package `.vue` reached through an alias or a relative
//! climb — cannot have. Those land here instead, keyed by their absolute path
//! replayed as subdirectories, so relative imports *between* external files
//! keep resolving inside the mirror (a mirrored barrel's `./UiButton.vue` is
//! rewritten to `./UiButton.vue.ts`, which sits beside it here exactly as it
//! does in the real tree) and the `.vue.ts` companion naming applies
//! unchanged.

use std::path::{Component, Path, PathBuf};

use crate::batch::error::CorsaError;

pub(super) const EXTERNAL_MIRROR_DIR: &str = "__vize_external__";

/// Recover the authored absolute path replayed below the external mirror.
pub fn external_mirror_original_path(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    for component in components.by_ref() {
        if matches!(component, Component::Normal(part) if part == EXTERNAL_MIRROR_DIR) {
            break;
        }
    }
    let replayed = components.collect::<Vec<_>>();
    if replayed.is_empty() {
        return None;
    }

    #[cfg(windows)]
    let mut original = {
        let first = replayed.first()?.as_os_str().to_string_lossy();
        let drive = first.strip_suffix("%3A")?;
        PathBuf::from(format!("{drive}:\\"))
    };
    #[cfg(not(windows))]
    let mut original = PathBuf::from("/");

    for component in replayed.iter().skip(usize::from(cfg!(windows))) {
        original.push(component.as_os_str());
    }
    Some(original)
}

/// The escape-subtree location for an out-of-root `path`.
///
/// A canonicalized absolute path has no `..`; refuse one rather than mint a
/// mirror path that escapes the subtree. The Windows drive prefix must stay
/// one path component, so only its `:` is encoded.
pub(super) fn external_mirror_path(
    virtual_root: &Path,
    path: &Path,
) -> Result<PathBuf, CorsaError> {
    let mut mirrored = virtual_root.join(EXTERNAL_MIRROR_DIR);
    for component in path.components() {
        match component {
            Component::Normal(part) => mirrored.push(part),
            Component::Prefix(prefix) => {
                mirrored.push(prefix.as_os_str().to_string_lossy().replace(':', "%3A"))
            }
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => {
                return Err(CorsaError::PathError {
                    path: path.to_path_buf(),
                });
            }
        }
    }
    Ok(mirrored)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn an_absolute_path_replays_as_subdirectories() {
        let virtual_root = crate::batch::project_virtual_root(Path::new("/proj"));
        let mirrored = super::external_mirror_path(
            &virtual_root,
            Path::new("/ws/packages/ui/src/UiButton.vue"),
        )
        .unwrap();
        assert_eq!(
            mirrored,
            virtual_root.join("__vize_external__/ws/packages/ui/src/UiButton.vue")
        );
    }

    #[test]
    fn sibling_files_stay_siblings() {
        let root = crate::batch::project_virtual_root(Path::new("/proj"));
        let barrel = super::external_mirror_path(&root, Path::new("/ws/pkg/index.ts")).unwrap();
        let vue = super::external_mirror_path(&root, Path::new("/ws/pkg/UiButton.vue")).unwrap();
        assert_eq!(barrel.parent(), vue.parent());
    }

    #[test]
    fn a_parent_component_is_refused() {
        assert!(
            super::external_mirror_path(
                &crate::batch::project_virtual_root(Path::new("/proj")),
                Path::new("/ws/../outside/App.vue"),
            )
            .is_err()
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn external_mirror_path_round_trips_to_the_authored_path() {
        let original = Path::new("/ws/packages/ui/src/UiButton.vue");
        let mirrored = super::external_mirror_path(Path::new("/project/.vize"), original).unwrap();
        assert_eq!(
            super::external_mirror_original_path(&mirrored),
            Some(original.into())
        );
    }
}
