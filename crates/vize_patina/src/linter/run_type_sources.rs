//! One run's disk view for imported macro types.

use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};

use vize_atelier_sfc::script::TypeSourceSnapshot;
use vize_atelier_sfc::{TypeResolutionBatchGuard, begin_type_resolution_batch};

/// The disk view every file of one run reads imported macro types through,
/// and the working directory that run's relative filenames name.
pub(crate) struct RunTypeSources {
    pub(crate) snapshot: TypeSourceSnapshot,
    root: Option<PathBuf>,
    /// The run is one filesystem-stability window, as the snapshot already
    /// assumes for the sources it reads. Project discovery and `tsconfig`
    /// captures therefore validate their metadata once per run instead of
    /// re-`stat`ing it, under one shared lock, for every import they resolve.
    _batch: TypeResolutionBatchGuard,
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
            _batch: begin_type_resolution_batch(),
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

/// Reads the modules an SFC's imports name through one disk view, for rules
/// that follow an import into the module it resolves to.
#[derive(Clone, Copy)]
pub(crate) struct ModuleReader<'s> {
    snapshot: &'s TypeSourceSnapshot,
}

impl ModuleReader<'_> {
    /// `specifier` resolved from `importer` the way imported macro types
    /// resolve: relative paths, tsconfig `paths` and package entries.
    pub(crate) fn resolve(&self, importer: &Path, specifier: &str) -> Option<PathBuf> {
        self.snapshot.resolve_import(importer, specifier)
    }

    /// Run `read` on the text of `path`, when it can be read.
    pub(crate) fn with_source<R>(&self, path: &Path, read: impl FnOnce(&str) -> R) -> Option<R> {
        let source = self.snapshot.read(path)?;
        Some(read(&source))
    }
}

/// Run `read` with `filename` as the importer and the run's view, or a fresh
/// view when the linter holds none.
pub(crate) fn with_module_reader<R>(
    sources: Option<&RunTypeSources>,
    filename: &str,
    read: impl FnOnce(&Path, ModuleReader<'_>) -> R,
) -> R {
    match sources {
        Some(sources) => read(
            Path::new(sources.importer(filename).as_ref()),
            ModuleReader {
                snapshot: &sources.snapshot,
            },
        ),
        None => read(
            Path::new(filename),
            ModuleReader {
                snapshot: &TypeSourceSnapshot::default(),
            },
        ),
    }
}

#[cfg(test)]
mod tests;
