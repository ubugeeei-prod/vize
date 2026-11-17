//! One run's disk view for imported macro types.

use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};

use vize_atelier_sfc::script::TypeSourceSnapshot;

/// The disk view every file of one run reads imported macro types through,
/// and the working directory that run's relative filenames name.
pub(crate) struct RunTypeSources {
    pub(crate) snapshot: TypeSourceSnapshot,
    root: Option<PathBuf>,
}

impl RunTypeSources {
    /// A fresh view for a run whose filenames are relative to the current
    /// working directory.
    pub(crate) fn for_current_dir() -> Self {
        Self::with_root(std::env::current_dir().ok())
    }

    pub(crate) fn with_root(root: Option<PathBuf>) -> Self {
        Self {
            snapshot: TypeSourceSnapshot::default(),
            root,
        }
    }

    /// `filename` as the importer path type resolution starts from.
    ///
    /// A relative filename names the same file against the run's working
    /// directory. Spelled absolute, the snapshot finds the importer's identity
    /// with one directory `stat` instead of a full `realpath` per SFC. `..`
    /// stays, so such a path still takes the snapshot's full canonicalization.
    pub(crate) fn importer<'f>(&self, filename: &'f str) -> Cow<'f, str> {
        let path = Path::new(filename);
        let Some(root) = self
            .root
            .as_ref()
            .filter(|_| !filename.is_empty() && path.is_relative())
        else {
            return Cow::Borrowed(filename);
        };
        let mut absolute = root.clone();
        absolute.extend(
            path.components()
                .filter(|component| !matches!(component, Component::CurDir)),
        );
        absolute
            .into_os_string()
            .into_string()
            .map_or(Cow::Borrowed(filename), Cow::Owned)
    }
}

#[cfg(test)]
mod tests;
