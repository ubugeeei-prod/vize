//! Immutable type sources shared by every provider at one editor revision.
#![expect(
    clippy::disallowed_types,
    reason = "source snapshots share std Arc<str> with Atelier"
)]

use std::sync::Arc;

use tower_lsp::lsp_types::Url;
use vize_atelier_sfc::script::TypeSourceSnapshot;
use vize_l0::{FxHashMap, FxHashSet};

use crate::document::DocumentStore;

struct Entry {
    revision: u64,
    text: Arc<str>,
}

#[derive(Default)]
pub(super) struct SourceState {
    revision: Option<u64>,
    snapshot: Option<Arc<TypeSourceSnapshot>>,
    entries: FxHashMap<Url, Entry>,
}

impl SourceState {
    pub(super) fn close(&mut self, uri: &Url) {
        self.entries.remove(uri);
        self.invalidate();
    }

    pub(super) fn invalidate(&mut self) {
        self.revision = None;
        self.snapshot = None;
    }

    pub(super) fn snapshot(
        &mut self,
        documents: &DocumentStore,
    ) -> Option<(u64, Arc<TypeSourceSnapshot>)> {
        for _ in 0..3 {
            let revision = documents.revision();
            if self.revision == Some(revision)
                && let Some(snapshot) = &self.snapshot
            {
                return Some((revision, Arc::clone(snapshot)));
            }
            let mut sources = Vec::with_capacity(documents.len());
            let mut open = FxHashSet::default();
            for document in documents.iter() {
                let Ok(path) = document.key().to_file_path() else {
                    continue;
                };
                let uri = document.key();
                let stamp = document.value().revision();
                let entry = self.entries.entry(uri.clone()).or_insert_with(|| Entry {
                    revision: stamp,
                    text: Arc::from(document.value().text()),
                });
                if entry.revision != stamp {
                    entry.revision = stamp;
                    entry.text = Arc::from(document.value().text());
                }
                sources.push((path, Arc::clone(&entry.text)));
                open.insert(uri.clone());
            }
            self.entries.retain(|uri, _| open.contains(uri));
            if documents.revision() != revision {
                continue;
            }
            let snapshot = Arc::new(TypeSourceSnapshot::new(sources));
            self.revision = Some(revision);
            self.snapshot = Some(Arc::clone(&snapshot));
            return Some((revision, snapshot));
        }
        None
    }
}
