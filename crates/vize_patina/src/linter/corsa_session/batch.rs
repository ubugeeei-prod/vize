//! Publish a complete set of probe documents before querying any of them.
//!
//! Input identities are checked before files are written. Retained source maps
//! become visible only after the single snapshot refresh succeeds.
use super::{
    CorsaTypeAwareSession,
    errors::compact_error,
    paths::{path_to_wire, virtual_file_path},
    session::virtual_source_hash,
    virtual_documents::{OpenedDocument, sync_stable_virtual_file},
    vue_dependencies,
};
use corsa::{
    api::{DocumentIdentifier, FileChangeSummary, FileChanges, OverlayChanges, OverlayUpdate},
    runtime::block_on,
};
use vize_l0::{FxHashSet, String, ToCompactString, profile};

impl CorsaTypeAwareSession {
    /// `(generated text, authored filename)` pairs in deterministic source order.
    pub(in crate::linter) fn open_virtual_projects(
        &mut self,
        inputs: &[(&str, &str)],
    ) -> Result<(), String> {
        let mut identities = FxHashSet::default();
        for (_, filename) in inputs {
            let path = virtual_file_path(&self.session_root, &self.project_root, filename);
            if !identities.insert(path) {
                return Err(vize_l0::cstr!(
                    "Duplicate authored document in patina snapshot batch: {filename}"
                ));
            }
        }
        let mut pending = Vec::new();
        let mut overlays = Vec::new();
        let mut changes = FileChangeSummary::default();
        for (source, filename) in inputs {
            let input_hash = virtual_source_hash(source);
            if self
                .opened_documents
                .get(*filename)
                .is_some_and(|document| document.input_hash == input_hash)
            {
                continue;
            }
            let next_path = virtual_file_path(&self.session_root, &self.project_root, filename);
            let prepared = profile!("patina.corsa_session.batch.prepare_dependencies", {
                vue_dependencies::prepare(
                    source,
                    filename,
                    &self.session_root,
                    &self.project_root,
                    &self.vue_path_aliases,
                    &mut self.dependency_cache,
                    &self.dependency_paths,
                )
            });
            let dependencies = vue_dependencies::materialize(&prepared.files, &[])?;
            self.dependency_paths
                .extend(prepared.files.iter().map(|(path, _)| path.clone()));
            append_changes(&mut changes, dependencies);
            let document = OpenedDocument {
                input_hash,
                rewritten_source: prepared.code,
                import_source_map: prepared.source_map,
            };
            let generated = document.rewritten_source.as_deref().unwrap_or(source);
            append_changes(
                &mut changes,
                profile!(
                    "patina.corsa_session.write_virtual_file",
                    sync_stable_virtual_file(
                        &mut self.published,
                        &next_path,
                        generated,
                        self.supports_overlay_updates,
                        &[],
                    )
                )?,
            );
            if self.supports_overlay_updates {
                overlays.push(OverlayUpdate {
                    document: path_to_wire(&next_path).as_str().into(),
                    text: generated.into(),
                    version: Some(self.overlay_version.saturating_add(1)),
                    language_id: Some("typescript".into()),
                });
            }
            pending.push((filename.to_compact_string(), document));
        }
        if !pending.is_empty() {
            if !self.supports_overlay_updates
                && changes.created.is_empty()
                && changes.changed.is_empty()
                && changes.deleted.is_empty()
                && let Some((filename, _)) = pending.last()
            {
                let path = virtual_file_path(&self.session_root, &self.project_root, filename);
                changes.changed.push(path_to_wire(&path).as_str().into());
            }
            changes
                .created
                .sort_unstable_by(|left, right| wire(left).cmp(wire(right)));
            changes.created.dedup();
            changes
                .changed
                .sort_unstable_by(|left, right| wire(left).cmp(wire(right)));
            changes.changed.dedup();
            changes
                .deleted
                .sort_unstable_by(|left, right| wire(left).cmp(wire(right)));
            changes.deleted.dedup();
            self.note_refresh(&changes);
            let summary = (!changes.created.is_empty()
                || !changes.changed.is_empty()
                || !changes.deleted.is_empty())
            .then_some(FileChanges::Summary(changes));
            let refreshed = if self.supports_overlay_updates {
                self.overlay_version = self.overlay_version.saturating_add(1);
                profile!(
                    "patina.corsa_session.batch.refresh_overlay",
                    block_on(self.session.refresh_with_overlay_changes(
                        summary,
                        Some(OverlayChanges {
                            upsert: overlays,
                            delete: Vec::new(),
                        })
                    ))
                )
            } else {
                profile!(
                    "patina.corsa_session.batch.refresh_files",
                    block_on(self.session.refresh(summary))
                )
            };
            refreshed.map_err(|error| {
                compact_error(
                    "Failed to update patina type snapshot",
                    error.to_compact_string().as_str(),
                )
            })?;
            for (filename, document) in pending {
                self.remember_virtual_input(&filename, document);
            }
        }
        if let Some((_, filename)) = inputs.last() {
            self.retarget_virtual_file(filename);
            self.active_filename = Some(filename.to_compact_string());
        }
        Ok(())
    }
}

fn append_changes(target: &mut FileChangeSummary, mut next: FileChangeSummary) {
    target.created.append(&mut next.created);
    target.changed.append(&mut next.changed);
    target.deleted.append(&mut next.deleted);
}

fn wire(document: &DocumentIdentifier) -> &str {
    match document {
        DocumentIdentifier::FileName(path) => path.as_str(),
        DocumentIdentifier::Uri { uri } => uri.as_str(),
    }
}
