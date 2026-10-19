//! Source facts shared by every reachability scan of one preparation pass.
//!
//! Importers of different packages scan overlapping closures, and each scan
//! used to stat, read and parse the same package files again (#9727). A pass
//! now loads each file once, reading a scan's whole frontier in parallel.
//! Facts never outlive the pass that treats the filesystem as stable.

#![expect(
    clippy::disallowed_types,
    reason = "parsed specifiers are shared by every scan that visits a file"
)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use oxc_span::SourceType;
use rayon::prelude::*;
use vize_carton::{FxHashMap, String as CompactString};

pub(super) type Occurrences = Arc<[(CompactString, crate::PackageResolutionMode)]>;

/// A readable source's length and module specifiers, in source order.
#[derive(Clone)]
pub(super) struct SourceText {
    pub(super) len: usize,
    pub(super) occurrences: Occurrences,
}

#[derive(Clone)]
enum Facts {
    /// `metadata` failed: the scan skips the file.
    Missing,
    /// `text` stays unread while the file exceeded the loading scan's
    /// per-file budget; `Some(None)` records a failed read.
    Sized {
        len: u64,
        text: Option<Option<SourceText>>,
    },
}

#[derive(Default)]
pub struct ReachabilitySources {
    files: FxHashMap<PathBuf, Facts>,
}

impl ReachabilitySources {
    /// File size from metadata, or `None` when the file cannot be stat'ed.
    pub(super) fn len(&mut self, path: &Path) -> Option<u64> {
        if !self.files.contains_key(path) {
            self.files.insert(path.to_path_buf(), stat(path));
        }
        match self.files.get(path)? {
            Facts::Missing => None,
            Facts::Sized { len, .. } => Some(*len),
        }
    }

    /// The parsed source, read on first use; `None` when it cannot be read.
    pub(super) fn text(&mut self, path: &Path) -> Option<SourceText> {
        let facts = self
            .files
            .entry(path.to_path_buf())
            .or_insert_with(|| stat(path));
        match facts {
            Facts::Missing => None,
            Facts::Sized {
                text: Some(text), ..
            } => text.clone(),
            Facts::Sized { text, .. } => text.insert(read(path)).clone(),
        }
    }

    pub(super) fn is_loaded(&self, path: &Path) -> bool {
        matches!(
            self.files.get(path),
            Some(Facts::Missing | Facts::Sized { text: Some(_), .. })
        )
    }

    /// Load every path not loaded yet in parallel. Sources over
    /// `max_file_bytes` are only stat'ed: a scan rejects them before reading.
    pub(super) fn prefetch<'a>(
        &mut self,
        paths: impl IntoIterator<Item = &'a PathBuf>,
        max_file_bytes: usize,
    ) {
        let pending = paths
            .into_iter()
            .filter(|path| !self.is_loaded(path))
            .collect::<Vec<_>>();
        let loaded = pending
            .into_par_iter()
            .map(|path| (path.clone(), load(path, max_file_bytes)))
            .collect::<Vec<_>>();
        self.files.extend(loaded);
    }
}

fn stat(path: &Path) -> Facts {
    match std::fs::metadata(path) {
        Ok(metadata) => Facts::Sized {
            len: metadata.len(),
            text: None,
        },
        Err(_) => Facts::Missing,
    }
}

fn load(path: &Path, max_file_bytes: usize) -> Facts {
    match stat(path) {
        Facts::Sized { len, .. } if u64::try_from(max_file_bytes).is_ok_and(|max| len <= max) => {
            Facts::Sized {
                len,
                text: Some(read(path)),
            }
        }
        facts => facts,
    }
}

fn read(path: &Path) -> Option<SourceText> {
    let content = std::fs::read_to_string(path).ok()?;
    let source_type = if path.extension().is_some_and(|extension| extension == "tsx") {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let occurrences = crate::batch::ImportRewriter::new()
        .collect_all_specifier_occurrences(&content, source_type)
        .into();
    Some(SourceText {
        len: content.len(),
        occurrences,
    })
}
