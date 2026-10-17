//! Query-captured authored mappings for every live native project root.
#![expect(
    clippy::disallowed_types,
    reason = "immutable mapping leaves are shared across queries"
)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use vize_carton::FxHashMap;
use vize_l0::{FxHashSet, String};

use super::CorsaMaterializedSource;
use super::vue_dependencies_alias::recover_lock;

type SourceFacts = FxHashMap<PathBuf, Arc<CorsaMaterializedSource>>;

/// Source facts retained by one exact editor project revision.
///
/// Hosts added to the same revision share this catalog. Each leaf is immutable;
/// an overlay, disk or membership change replaces the catalog, so an in-flight
/// query keeps the source bytes and coordinate maps it opened with.
#[derive(Clone, Default)]
pub struct CorsaSourceCatalog {
    sources: Arc<Mutex<SourceFacts>>,
}

impl CorsaSourceCatalog {
    /// Whether two query handles retain the same editor project revision.
    pub fn shares_revision_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.sources, &other.sources)
    }

    /// Look up an exact Canon materialized identity without copying the project.
    pub fn get(&self, path: &Path) -> Option<Arc<CorsaMaterializedSource>> {
        let sources = recover_lock(&self.sources);
        sources.get(path).cloned().or_else(|| {
            sources
                .get(&vize_carton::path::canonicalize_non_verbatim(path))
                .cloned()
        })
    }

    /// Synchronize the admitted live project, while keeping the query's own
    /// closure and projection packet unchanged. Opening a sibling's already
    /// materialized dependency later must not change the native document view.
    pub(in crate::corsa_bridge) fn extend_native_project_documents(
        &self,
        project_root: &Path,
        documents: &mut Vec<(String, String)>,
    ) {
        let mut opened = documents
            .iter()
            .map(|(uri, _)| uri.clone())
            .collect::<FxHashSet<_>>();
        let mut sources = recover_lock(&self.sources)
            .values()
            .filter(|source| {
                source.materialized_path.starts_with(project_root)
                    && source.mapping_kind != super::CorsaMaterializedMappingKind::Synthetic
            })
            .cloned()
            .collect::<Vec<_>>();
        sources
            .sort_unstable_by(|left, right| left.materialized_path.cmp(&right.materialized_path));
        for source in sources {
            let uri = crate::file_uri::path_to_file_uri(&source.materialized_path);
            if opened.insert(uri.clone()) {
                documents.push((uri, source.code.clone()));
            }
        }
    }

    pub(in crate::corsa_bridge) fn include(&self, sources: Vec<CorsaMaterializedSource>) -> Self {
        let mut retained = recover_lock(&self.sources);
        if sources.iter().any(|source| {
            retained.get(&source.materialized_path).is_some_and(|old| {
                old.source_path != source.source_path
                    || old.source != source.source
                    || old.code != source.code
                    || old.mapping != source.mapping
                    || old.import_source_map != source.import_source_map
                    || old.mapping_kind != source.mapping_kind
            })
        }) {
            // A caller can change host text without including it in overlays.
            // Fork only that revision; unchanged leaf data stays shared.
            let mut refreshed = retained.clone();
            for source in sources {
                refreshed.insert(source.materialized_path.clone(), Arc::new(source));
            }
            return Self {
                sources: Arc::new(Mutex::new(refreshed)),
            };
        }
        for source in sources {
            retained
                .entry(source.materialized_path.clone())
                .or_insert_with(|| Arc::new(source));
        }
        self.clone()
    }

    pub(in crate::corsa_bridge) fn retain_live_files(
        &self,
        current: &vize_carton::FxHashSet<PathBuf>,
        preserved: &vize_carton::FxHashSet<PathBuf>,
    ) {
        recover_lock(&self.sources)
            .retain(|path, _| current.contains(path) || preserved.contains(path));
    }
}
