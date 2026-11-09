//! Reuse exact native payloads without rescanning retained text revisions.
#![expect(
    clippy::disallowed_types,
    reason = "immutable URI/code payloads cross native worker boundaries"
)]

use super::{CorsaProjectClient, tracked_documents::DocumentCheckpoint};
use corsa::api::{FileChangeSummary, FileChanges};
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use std::time::SystemTime;
use vize_l0::{String, cstr};

/// Native inputs share immutable URI/code payloads without carrying source
/// mappings or a catalog into the native worker.
#[expect(
    clippy::disallowed_types,
    reason = "immutable payload bytes cross native workers"
)]
#[derive(Clone, Default)]
pub(crate) struct CorsaNativeDocumentBatch {
    shared: std::sync::Arc<Vec<(String, String)>>,
    additions: Vec<std::sync::Arc<(String, String)>>,
}

#[expect(
    clippy::disallowed_types,
    reason = "immutable payload bytes cross native workers"
)]
impl CorsaNativeDocumentBatch {
    pub(crate) fn shared(shared: std::sync::Arc<Vec<(String, String)>>) -> Self {
        Self {
            shared,
            additions: Vec::new(),
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(String, String)> {
        self.shared
            .iter()
            .chain(self.additions.iter().map(std::sync::Arc::as_ref))
    }

    pub(crate) fn len(&self) -> usize {
        self.shared.len() + self.additions.len()
    }

    pub(crate) fn push_shared(&mut self, document: std::sync::Arc<(String, String)>) {
        self.additions.push(document);
    }

    /// Point each entry with a replacement at its new text. A listed payload
    /// for that URI is shared as is; the base is copied only when it changes,
    /// which also dissociates weak receipts of the old base.
    pub(crate) fn replace_texts(
        &mut self,
        replacements: &vize_l0::FxHashMap<String, String>,
        payloads: &[std::sync::Arc<(String, String)>],
    ) {
        if self
            .shared
            .iter()
            .any(|(uri, _)| replacements.contains_key(uri.as_str()))
        {
            for (uri, text) in std::sync::Arc::make_mut(&mut self.shared) {
                if let Some(replacement) = replacements.get(uri.as_str()) {
                    text.clone_from(replacement);
                }
            }
        }
        for document in &mut self.additions {
            let Some(replacement) = replacements.get(document.0.as_str()) else {
                continue;
            };
            *document = payloads
                .iter()
                .find(|payload| payload.0 == document.0 && payload.1 == *replacement)
                .map(std::sync::Arc::clone)
                .unwrap_or_else(|| std::sync::Arc::new((document.0.clone(), replacement.clone())));
        }
    }

    #[cfg(test)]
    pub(crate) fn push(&mut self, document: (String, String)) {
        self.push_shared(std::sync::Arc::new(document));
    }

    #[cfg(test)]
    pub(crate) fn shared_additions_for_test(&self) -> &[std::sync::Arc<(String, String)>] {
        &self.additions
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[cfg(test)]
    pub(crate) fn payload_owner_for_test(&self) -> std::sync::Weak<Vec<(String, String)>> {
        std::sync::Arc::downgrade(&self.shared)
    }

    #[cfg(test)]
    pub(crate) fn to_vec(&self) -> Vec<(String, String)> {
        self.iter().cloned().collect()
    }
}

#[expect(
    clippy::disallowed_types,
    reason = "owned payload bytes cross native workers"
)]
impl From<Vec<(String, String)>> for CorsaNativeDocumentBatch {
    fn from(documents: Vec<(String, String)>) -> Self {
        Self::shared(std::sync::Arc::new(documents))
    }
}

impl PartialEq for CorsaNativeDocumentBatch {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}
impl Eq for CorsaNativeDocumentBatch {}
impl std::fmt::Debug for CorsaNativeDocumentBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
#[cfg(test)]
impl PartialEq<Vec<(String, String)>> for CorsaNativeDocumentBatch {
    fn eq(&self, other: &Vec<(String, String)>) -> bool {
        self.iter().eq(other.iter())
    }
}
#[cfg(test)]
impl PartialEq<CorsaNativeDocumentBatch> for Vec<(String, String)> {
    fn eq(&self, other: &CorsaNativeDocumentBatch) -> bool {
        self.iter().eq(other.iter())
    }
}

const BATCH_CAPACITY: usize = 4;

type Document = (String, String);

struct PayloadIdentity {
    base: Weak<Vec<Document>>,
    additions: Vec<Weak<Document>>,
}

impl PayloadIdentity {
    fn capture(batch: &CorsaNativeDocumentBatch) -> Self {
        Self {
            base: Arc::downgrade(&batch.shared),
            additions: batch.additions.iter().map(Arc::downgrade).collect(),
        }
    }

    fn is_live(&self) -> bool {
        self.base.strong_count() != 0
            && self.additions.iter().all(|owner| owner.strong_count() != 0)
    }

    fn matches(&self, batch: &CorsaNativeDocumentBatch) -> bool {
        self.base.ptr_eq(&Arc::downgrade(&batch.shared))
            && self.additions.len() == batch.additions.len()
            && self
                .additions
                .iter()
                .zip(&batch.additions)
                .all(|(before, current)| before.ptr_eq(&Arc::downgrade(current)))
    }
}

struct CanonRow {
    path: PathBuf,
    desired: [u8; 32],
}

struct CanonBatch {
    identity: PayloadIdentity,
    texts: DocumentCheckpoint,
    rows: Vec<CanonRow>,
}

#[derive(Default)]
pub(super) struct CanonBatchCache {
    entries: Vec<CanonBatch>,
    synchronized: bool,
}

impl CanonBatchCache {
    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.synchronized = false;
    }

    pub(super) fn observed_canonical_sync(&mut self) {
        self.synchronized = true;
    }
}

impl CorsaProjectClient {
    /// Payload identity certifies the complete ordered URI/code input; the
    /// mutation-owned checkpoint certifies the retained text still agrees.
    /// Namespace and physical observations remain mandatory on every reuse.
    pub(crate) fn did_open_immutable_batch(
        &mut self,
        batch: &CorsaNativeDocumentBatch,
    ) -> Result<(), String> {
        self.canon_batches.synchronized = false;
        let texts = self.document_texts.checkpoint();
        self.canon_batches
            .entries
            .retain(|entry| entry.texts == texts && entry.identity.is_live());
        let documents = batch
            .iter()
            .map(|(uri, text)| (uri.as_str(), text.as_str()))
            .collect::<Vec<_>>();
        if let Some(index) = self
            .canon_batches
            .entries
            .iter()
            .position(|entry| entry.identity.matches(batch))
        {
            let entry = self.canon_batches.entries.remove(index);
            if self.reuse_canon_batch(&documents, &entry)? {
                self.canon_batches.entries.push(entry);
                return Ok(());
            }
        }
        for payload in &batch.additions {
            self.shared_payloads.register(payload);
        }
        let opened = self.did_open_batch_fast(&documents);
        self.shared_payloads.clear();
        opened?;
        if self.canon_batches.synchronized
            && let Some(paths) = self.canon_batch_paths(&documents)
        {
            self.remember_canon_batch(batch, &documents, paths);
        }
        Ok(())
    }

    fn remember_canon_batch(
        &mut self,
        batch: &CorsaNativeDocumentBatch,
        documents: &[(&str, &str)],
        paths: Vec<PathBuf>,
    ) {
        let rows = documents
            .iter()
            .zip(paths)
            .map(|((uri, text), path)| CanonRow {
                path,
                desired: self.document_texts.desired_digest(uri, text),
            })
            .collect();
        let texts = self.document_texts.checkpoint();
        self.canon_batches
            .entries
            .retain(|entry| entry.texts == texts && entry.identity.is_live());
        if self.canon_batches.entries.len() == BATCH_CAPACITY {
            self.canon_batches.entries.remove(0);
        }
        self.canon_batches.entries.push(CanonBatch {
            identity: PayloadIdentity::capture(batch),
            texts,
            rows,
        });
    }

    fn can_reuse_canon_batch(&self) -> bool {
        !self.closed
            && self.has_project_session()
            && !self.supports_overlay_api()
            && !self.logical_workspace_overlays
            && !self.materialized_project_session
            && self.project_root.join("__vize_vue_modules.d.ts").is_file()
    }

    fn canon_batch_paths(&self, documents: &[(&str, &str)]) -> Option<Vec<PathBuf>> {
        if !self.can_reuse_canon_batch() {
            return None;
        }
        let paths = documents
            .iter()
            .map(|(uri, _)| crate::file_uri::file_uri_to_path(uri))
            .collect::<Option<Vec<_>>>()?;
        paths
            .iter()
            .all(|path| {
                super::session_paths::is_within_project_namespace(path, &self.project_root)
                    && path.is_file()
                    && super::session_paths::is_within_physical_project_namespace(
                        path,
                        &self.project_root,
                    )
            })
            .then_some(paths)
    }

    fn reuse_canon_batch(
        &mut self,
        documents: &[(&str, &str)],
        entry: &CanonBatch,
    ) -> Result<bool, String> {
        if !self.can_reuse_canon_batch() || entry.rows.len() != documents.len() {
            return Ok(false);
        }
        self.reuse_canonical_documents(documents, entry)
    }

    fn reuse_canonical_documents(
        &mut self,
        documents: &[(&str, &str)],
        entry: &CanonBatch,
    ) -> Result<bool, String> {
        let phase = crate::corsa_bridge::preparation_trace::Phase::start(
            "immutable_native_batch",
            documents.len(),
        );
        // Keep the original complete namespace and existence guard. An old
        // physical observation cannot authorize a retargeted directory link.
        if !entry.rows.iter().all(|row| {
            super::session_paths::is_within_project_namespace(&row.path, &self.project_root)
                && row.path.is_file()
        }) {
            return Ok(false);
        }
        if !entry.rows.iter().all(|row| {
            super::session_paths::is_within_physical_project_namespace(
                &row.path,
                &self.project_root,
            )
        }) {
            return Err(cstr!("Canon document left its private physical namespace"));
        }
        let now = SystemTime::now();
        let mut summary = FileChangeSummary::default();
        for ((uri, text), row) in documents.iter().zip(&entry.rows) {
            self.clear_document_state(uri);
            if !self
                .canon_file_cache
                .observation_is_current(&row.path, row.desired, now)
                && self
                    .canon_file_cache
                    .synchronize_with_digest(&row.path, text, || row.desired)
                    .map_err(|error| {
                        cstr!(
                            "Failed to update Canon document {}: {error}",
                            row.path.display()
                        )
                    })?
            {
                summary
                    .changed
                    .push(super::session::uri_document_identifier(uri));
            }
            self.remember_session_document_uri(uri, (*uri).into());
        }
        if !summary.changed.is_empty() {
            self.refresh_materialized_overlay_snapshot(Some(FileChanges::Summary(summary)))?;
        }
        phase.finish();
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
