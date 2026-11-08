//! Query-captured authored mappings for every live native project root.
#![expect(
    clippy::disallowed_types,
    reason = "immutable mapping leaves are shared across queries"
)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};

use vize_carton::FxHashMap;
use vize_l0::FxHashSet;
use vize_l0::String;

use super::CorsaMaterializedSource;
use super::vue_dependencies_alias::recover_lock;

type SourceFacts = FxHashMap<PathBuf, Arc<CorsaMaterializedSource>>;

#[derive(Clone, Default)]
struct NativePayloads(FxHashMap<PathBuf, NativePayload>);

#[derive(Clone)]
struct NativePayload {
    source: Weak<CorsaMaterializedSource>,
    document: Arc<(String, String)>,
}

impl NativePayloads {
    fn document(
        &mut self,
        source: &Arc<CorsaMaterializedSource>,
        uri: String,
    ) -> Arc<(String, String)> {
        let identity = Arc::downgrade(source);
        if let Some(payload) = self.0.get_mut(&source.materialized_path) {
            if payload.source.ptr_eq(&identity) {
                return Arc::clone(&payload.document);
            }
            if payload.document.0 == uri && payload.document.1 == source.code {
                payload.source = identity;
                return Arc::clone(&payload.document);
            }
        }
        let document = Arc::new((uri, source.code.clone()));
        self.0.insert(
            source.materialized_path.clone(),
            NativePayload {
                source: identity,
                document: Arc::clone(&document),
            },
        );
        document
    }
}

/// Source facts retained by one exact editor project revision.
///
/// Hosts added to the same revision share mapping facts. Each leaf is immutable;
/// changed source bytes fork those facts. Native members are captured separately
/// after materialization, so later facts cannot change an in-flight native view.
#[derive(Clone, Default)]
pub struct CorsaSourceCatalog {
    sources: Arc<Mutex<SourceFacts>>,
    native_members: Arc<Vec<Arc<CorsaMaterializedSource>>>,
    /// Only one payload per live physical path. Weak source identities cannot
    /// keep retired mappings alive or confuse a replaced immutable leaf.
    native_payloads: Arc<Mutex<NativePayloads>>,
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

    /// Capture immutable leaves only after their complete project materialized.
    /// Later mapping facts can grow without admitting unmaterialized members.
    pub(in crate::corsa_bridge) fn with_native_snapshot(
        &self,
        snapshot: &crate::batch::virtual_project::MaterializedFileSnapshot,
    ) -> Self {
        let mut members = recover_lock(&self.sources)
            .values()
            .filter(|source| snapshot.contains_file(&source.materialized_path))
            .cloned()
            .collect::<Vec<_>>();
        members
            .sort_unstable_by(|left, right| left.materialized_path.cmp(&right.materialized_path));
        recover_lock(&self.native_payloads)
            .0
            .retain(|path, _| snapshot.contains_file(path));
        Self {
            sources: Arc::clone(&self.sources),
            native_members: Arc::new(members),
            native_payloads: Arc::clone(&self.native_payloads),
        }
    }

    /// Synchronize the admitted live project, while keeping the query's own
    /// closure and projection packet unchanged. Opening a sibling's already
    /// materialized dependency later must not change the native document view.
    #[cfg(test)]
    pub(crate) fn extend_native_project_documents(
        &self,
        project_root: &Path,
        documents: &mut Vec<(String, String)>,
    ) {
        let mut opened = documents
            .iter()
            .map(|(uri, _)| uri.clone())
            .collect::<FxHashSet<_>>();
        for source in self.native_members.iter().filter(|source| {
            source.materialized_path.starts_with(project_root)
                && source.mapping_kind != super::CorsaMaterializedMappingKind::Synthetic
        }) {
            let uri = crate::file_uri::path_to_file_uri(&source.materialized_path);
            if opened.insert(uri.clone()) {
                documents.push((uri, source.code.clone()));
            }
        }
    }

    /// Borrow immutable URI/code payloads without carrying mapping/catalog
    /// ownership into the native worker or copying every closed source again.
    pub(crate) fn extend_native_project_document_batch(
        &self,
        project_root: &Path,
        documents: &mut super::vue_document::CorsaNativeDocumentBatch,
    ) {
        let mut opened = documents
            .iter()
            .map(|(uri, _)| uri.clone())
            .collect::<FxHashSet<_>>();
        let mut payloads = recover_lock(&self.native_payloads);
        for source in self.native_members.iter().filter(|source| {
            source.materialized_path.starts_with(project_root)
                && source.mapping_kind != super::CorsaMaterializedMappingKind::Synthetic
        }) {
            let uri = crate::file_uri::path_to_file_uri(&source.materialized_path);
            if opened.insert(uri.clone()) {
                documents.push_shared(payloads.document(source, uri));
            }
        }
    }

    /// Preserve the outgoing open-root packet across an overlay epoch without
    /// joining independent source graphs or copying their closed dependencies.
    pub(in crate::corsa_bridge) fn extend_open_project_document_batch(
        &self,
        root: &Path,
        overlays: &dyn super::vue_document::OverlayAccess,
        documents: &mut super::vue_document::CorsaNativeDocumentBatch,
    ) {
        let mut opened = documents
            .iter()
            .map(|(uri, _)| uri.clone())
            .collect::<FxHashSet<_>>();
        let mut payloads = recover_lock(&self.native_payloads);
        for source in self.native_members.iter().filter(|source| {
            source.materialized_path.starts_with(root)
                && overlays.contains_key(&source.source_path)
                && source.mapping_kind != super::CorsaMaterializedMappingKind::Synthetic
        }) {
            let uri = crate::file_uri::path_to_file_uri(&source.materialized_path);
            if opened.insert(uri.clone()) {
                documents.push_shared(payloads.document(source, uri));
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
                native_members: Arc::clone(&self.native_members),
                native_payloads: Arc::new(Mutex::new(recover_lock(&self.native_payloads).clone())),
            };
        }
        for source in sources {
            retained
                .entry(source.materialized_path.clone())
                .or_insert_with(|| Arc::new(source));
        }
        self.clone()
    }

    pub(in crate::corsa_bridge) fn retaining_live_files(
        &self,
        current: &vize_carton::FxHashSet<PathBuf>,
        preserved: &vize_carton::FxHashSet<PathBuf>,
    ) -> Self {
        let sources = recover_lock(&self.sources);
        if sources
            .keys()
            .all(|path| current.contains(path) || preserved.contains(path))
        {
            return self.clone();
        }
        let retained = sources
            .iter()
            .filter(|(path, _)| current.contains(*path) || preserved.contains(*path))
            .map(|(path, source)| (path.clone(), Arc::clone(source)))
            .collect();
        let mut payloads = recover_lock(&self.native_payloads).clone();
        payloads
            .0
            .retain(|path, _| current.contains(path) || preserved.contains(path));
        Self {
            sources: Arc::new(Mutex::new(retained)),
            native_members: Arc::clone(&self.native_members),
            native_payloads: Arc::new(Mutex::new(payloads)),
        }
    }

    pub(in crate::corsa_bridge) fn current_overlay_sources(
        &self,
        overlays: &dyn super::vue_document::OverlayAccess,
    ) -> FxHashSet<PathBuf> {
        recover_lock(&self.sources)
            .values()
            .filter(|source| source.mapping_kind.is_mappable())
            .filter(|source| {
                overlays
                    .get(&source.source_path)
                    .is_some_and(|text| text == source.source.as_str())
            })
            .map(|source| source.source_path.clone())
            .collect()
    }
}

#[cfg(test)]
mod payload_tests;
