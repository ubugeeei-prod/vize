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

use vize_carton::String as CompactString;

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
        PathBuf::from(format!("{}\\", decode_prefix(&first)?))
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
/// mirror path that escapes the subtree. A Windows prefix must stay one
/// relative path component — a drive letter, but also a UNC share or a
/// verbatim prefix — so its separators and reserved characters are encoded.
pub(super) fn external_mirror_path(
    virtual_root: &Path,
    path: &Path,
) -> Result<PathBuf, CorsaError> {
    let mut mirrored = virtual_root.join(EXTERNAL_MIRROR_DIR);
    for component in path.components() {
        match component {
            Component::Normal(part) => mirrored.push(part),
            Component::Prefix(prefix) => {
                mirrored.push(encode_prefix(&prefix.as_os_str().to_string_lossy()));
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

/// Percent-encode a Windows prefix into one relative path component.
///
/// `PathBuf::push` of a rooted or separator-bearing component would discard
/// the mirror root, so every separator, `:` and verbatim `?` is escaped (and
/// `%` first, to keep the encoding reversible).
fn encode_prefix(prefix: &str) -> CompactString {
    let mut encoded = CompactString::with_capacity(prefix.len());
    for character in prefix.chars() {
        match character {
            '%' => encoded.push_str("%25"),
            ':' => encoded.push_str("%3A"),
            '?' => encoded.push_str("%3F"),
            '\\' => encoded.push_str("%5C"),
            '/' => encoded.push_str("%2F"),
            other => encoded.push(other),
        }
    }
    encoded
}

/// Reverse [`encode_prefix`]; `None` when the component was never a prefix.
#[cfg(any(windows, test))]
fn decode_prefix(component: &str) -> Option<CompactString> {
    let mut decoded = CompactString::with_capacity(component.len());
    let mut rest = component;
    let mut escaped = false;
    while let Some(index) = rest.find('%') {
        decoded.push_str(&rest[..index]);
        let character = match rest.get(index..index + 3)? {
            "%25" => '%',
            "%3A" => ':',
            "%3F" => '?',
            "%5C" => '\\',
            "%2F" => '/',
            _ => return None,
        };
        decoded.push(character);
        escaped = true;
        rest = &rest[index + 3..];
    }
    decoded.push_str(rest);
    escaped.then_some(decoded)
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
    fn every_windows_prefix_encodes_to_one_relative_component() {
        // A UNC share or verbatim prefix carries separators and `?`: left raw,
        // pushing it would reroot the mirror out of the virtual root.
        for prefix in [
            "C:",
            r"\\server\share",
            r"\\?\C:",
            r"\\?\UNC\server\share",
            r"\\.\pipe",
        ] {
            let encoded = super::encode_prefix(prefix);
            assert!(Path::new(&encoded).is_relative(), "{prefix} stayed rooted");
            assert_eq!(Path::new(&encoded).components().count(), 1, "{prefix}");
            assert_eq!(super::decode_prefix(&encoded).as_deref(), Some(prefix));
        }
    }

    #[test]
    fn an_unencoded_component_is_not_read_back_as_a_prefix() {
        assert_eq!(super::decode_prefix("packages"), None);
        assert_eq!(super::decode_prefix("%ZZ"), None);
    }

    #[test]
    #[cfg(windows)]
    fn external_mirror_path_round_trips_a_unc_path() {
        let original = Path::new(r"\\server\share\ws\packages\ui\src\UiButton.vue");
        let virtual_root = Path::new(r"C:\project\.vize");
        let mirrored = super::external_mirror_path(virtual_root, original).unwrap();
        assert!(mirrored.starts_with(virtual_root));
        assert_eq!(
            super::external_mirror_original_path(&mirrored),
            Some(original.into())
        );
    }

    #[test]
    #[cfg(windows)]
    fn external_mirror_path_round_trips_a_drive_letter_path() {
        // The drive prefix is the only encoded component: `%3A` must come back
        // off, and the rebuilt root must not be pushed twice.
        let original = Path::new(r"C:\ws\packages\ui\src\UiButton.vue");
        let mirrored =
            super::external_mirror_path(Path::new(r"C:\project\.vize"), original).unwrap();
        assert_eq!(
            super::external_mirror_original_path(&mirrored),
            Some(original.into())
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

    #[test]
    #[cfg(windows)]
    fn windows_external_mirror_path_round_trips_to_the_authored_path() {
        let original = Path::new(r"C:\ws\packages\ui\src\UiButton.vue");
        let mirrored =
            super::external_mirror_path(Path::new(r"C:\project\.vize"), original).unwrap();
        assert_eq!(
            super::external_mirror_original_path(&mirrored),
            Some(original.into())
        );
    }
}
