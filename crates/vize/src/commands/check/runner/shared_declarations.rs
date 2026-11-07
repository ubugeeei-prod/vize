//! Declarations one program of a check emits for the programs that import its
//! sources, the way `tsc -b` hands a referenced project's output to its
//! importers instead of checking its sources again (#12102).

use std::path::{Path, PathBuf};

use vize_l0::{FxHashMap, String, cstr};

use crate::commands::check::path_cache::CanonicalPathCache;

/// `VIZE_CHECK_SHARE_DECLARATIONS=0` keeps every program on sources, for
/// paired measurements and as an escape hatch.
pub(super) fn sharing_enabled() -> bool {
    !matches!(
        std::env::var("VIZE_CHECK_SHARE_DECLARATIONS").as_deref(),
        Ok("0")
    )
}

#[derive(Default)]
pub(super) struct SharedDeclarations {
    /// Removed with the run; checks have joined by then.
    dir: Option<tempfile::TempDir>,
    programs: usize,
    /// Owner config to its emitted declarations, by canonical source path.
    emitted: FxHashMap<PathBuf, FxHashMap<PathBuf, PathBuf>>,
}

impl SharedDeclarations {
    /// The declaration an earlier program emitted for `canonical_source`.
    pub(super) fn declaration_for(&self, owner: &Path, canonical_source: &Path) -> Option<&Path> {
        self.emitted
            .get(owner)?
            .get(canonical_source)
            .map(PathBuf::as_path)
    }

    /// A fresh output directory for the next program's emit.
    pub(super) fn next_out_dir(&mut self) -> Result<PathBuf, String> {
        if self.dir.is_none() {
            self.dir = Some(
                tempfile::Builder::new()
                    .prefix("vize-check-declarations-")
                    .tempdir()
                    .map_err(|error| {
                        cstr!("Failed to create a directory for shared declarations: {error}")
                    })?,
            );
        }
        let Some(dir) = self.dir.as_ref() else {
            return Err("shared declaration directory was not created".into());
        };
        self.programs += 1;
        let out_dir = dir.path().join(cstr!("program-{}", self.programs).as_str());
        std::fs::create_dir_all(&out_dir).map_err(|error| {
            cstr!(
                "Failed to create {} for shared declarations: {error}",
                out_dir.display()
            )
        })?;
        Ok(out_dir)
    }

    /// Remember what `owner` emitted, keyed by canonical source.
    pub(super) fn record(
        &mut self,
        owner: PathBuf,
        declarations: Vec<(PathBuf, PathBuf)>,
        canonical_paths: &mut CanonicalPathCache,
    ) {
        let entries = self.emitted.entry(owner).or_default();
        for (source, declaration) in declarations {
            entries.insert(canonical_paths.canonicalize(&source), declaration);
        }
    }
}
