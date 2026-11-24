//! Parsed module summaries, kept across compiles by the bytes they came from.

#![expect(
    clippy::disallowed_types,
    reason = "a parsed summary is shared by the module graphs of parallel workers"
)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, RwLock};

use sha2::{Digest, Sha256};
use vize_carton::{FxHashMap, String};

use super::work::{Step, count};
use super::{FileTypeSummary, file_summary_from_source};

/// One transient read. Changed bytes and their digest move to the parser together.
pub(super) struct FileTypeSource {
    pub(super) text: String,
    pub(super) fingerprint: [u8; 32],
}

impl FileTypeSource {
    pub(super) fn read(path: &Path) -> Option<Self> {
        count(Step::SourceRead, 1);
        let text = std::fs::read_to_string(path).ok()?;
        let fingerprint = Sha256::digest(text.as_bytes()).into();
        Some(Self {
            text: text.into(),
            fingerprint,
        })
    }
}

/// A parsed summary and a strong source digest, without retaining the whole file.
struct CachedFileSummary {
    fingerprint: [u8; 32],
    summary: Arc<FileTypeSummary>,
}

/// Summaries by path, reused only for the bytes they were parsed from.
/// Metadata cannot prove that a same-length edit within filesystem timestamp
/// granularity is unchanged, so every load reads and digests the file. A
/// module graph loads each file once; this cache spares it the parse.
static FILE_TYPE_CACHE: LazyLock<RwLock<FxHashMap<PathBuf, CachedFileSummary>>> =
    LazyLock::new(|| RwLock::new(FxHashMap::default()));

/// Read `path` and summarize it. The cache lock covers the lookup and the
/// swap of a finished entry, never the read, the digest or the parse, so a
/// worker reading one file does not hold up workers reading others.
pub(super) fn load(path: &Path) -> Option<Arc<FileTypeSummary>> {
    let source = FileTypeSource::read(path)?;
    count(Step::SharedLock, 1);
    if let Ok(cache) = FILE_TYPE_CACHE.read()
        && let Some(entry) = cache.get(path)
        && entry.fingerprint == source.fingerprint
    {
        return Some(Arc::clone(&entry.summary));
    }
    let summary = Arc::new(file_summary_from_source(path, &source.text));
    count(Step::SharedLock, 1);
    if let Ok(mut cache) = FILE_TYPE_CACHE.write() {
        cache.insert(
            path.to_path_buf(),
            CachedFileSummary {
                fingerprint: source.fingerprint,
                summary: Arc::clone(&summary),
            },
        );
    }
    Some(summary)
}
