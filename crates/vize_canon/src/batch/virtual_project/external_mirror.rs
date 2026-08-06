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
        let mirrored = super::external_mirror_path(
            Path::new("/proj/node_modules/.vize/canon"),
            Path::new("/ws/packages/ui/src/UiButton.vue"),
        )
        .unwrap();
        assert_eq!(
            mirrored,
            Path::new(
                "/proj/node_modules/.vize/canon/__vize_external__/ws/packages/ui/src/UiButton.vue"
            )
        );
    }

    #[test]
    fn sibling_files_stay_siblings() {
        let root = Path::new("/proj/node_modules/.vize/canon");
        let barrel = super::external_mirror_path(root, Path::new("/ws/pkg/index.ts")).unwrap();
        let vue = super::external_mirror_path(root, Path::new("/ws/pkg/UiButton.vue")).unwrap();
        assert_eq!(barrel.parent(), vue.parent());
    }

    #[test]
    fn a_parent_component_is_refused() {
        assert!(
            super::external_mirror_path(
                Path::new("/proj/node_modules/.vize/canon"),
                Path::new("/ws/../outside/App.vue"),
            )
            .is_err()
        );
    }
}
