use super::{
    CorsaTypeAwareSession,
    errors::{compact_error, io_error_message},
    paths::{
        TSCONFIG_FILE_NAME, allocate_session_root, path_to_wire, remove_session_root,
        resolve_corsa_executable, resolve_project_root, session_tsconfig_contents,
        virtual_file_path,
    },
};
use corsa::{
    api::{
        ApiMode, ApiSpawnConfig, FileChangeSummary, FileChanges, OverlayChanges, OverlayUpdate,
        ProjectSession,
    },
    runtime::block_on,
};
use std::hash::{Hash, Hasher};

use vize_carton::corsa_api_mode::uses_async_json_rpc_api;
use vize_l0::{String, ToCompactString, profile};

impl CorsaTypeAwareSession {
    pub(in crate::linter) fn new_with_corsa_path(
        filename: &str,
        corsa_path: Option<&std::path::Path>,
        tsconfig_path: Option<std::path::PathBuf>,
    ) -> Result<Self, String> {
        let project_root = resolve_project_root(filename);
        let executable = resolve_corsa_executable(&project_root, corsa_path)?;
        let session_root = allocate_session_root(&project_root);
        let cleanup_guard = SessionRootCleanup::new(session_root.clone());
        profile!(
            "patina.corsa_session.create_dir",
            std::fs::create_dir_all(&session_root)
        )
        .map_err(|error| {
            io_error_message(
                "Failed to create patina session directory",
                &session_root,
                &error,
            )
        })?;

        let config_path = session_root.join(TSCONFIG_FILE_NAME);
        let tsconfig_contents = session_tsconfig_contents(&project_root, tsconfig_path.as_deref());
        let vue_path_aliases =
            super::vue_dependencies::PathAliases::from_tsconfig(&tsconfig_contents, &project_root);
        profile!(
            "patina.corsa_session.write_tsconfig",
            std::fs::write(&config_path, tsconfig_contents.as_str(),)
        )
        .map_err(|error| {
            io_error_message("Failed to write patina tsconfig", &config_path, &error)
        })?;

        let virtual_file_path = virtual_file_path(&session_root, &project_root, filename);
        if let Some(parent) = virtual_file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                io_error_message(
                    "Failed to create patina virtual directory",
                    &virtual_file_path,
                    &error,
                )
            })?;
        }
        profile!(
            "patina.corsa_session.prime_virtual_file",
            std::fs::write(&virtual_file_path, "")
        )
        .map_err(|error| {
            io_error_message(
                "Failed to prime patina virtual TypeScript",
                &virtual_file_path,
                &error,
            )
        })?;

        let config_path_wire = path_to_wire(&config_path);
        let virtual_file_wire = path_to_wire(&virtual_file_path);
        let api_mode = api_mode_for_executable(&executable);
        let session = profile!(
            "patina.corsa_session.spawn",
            block_on(ProjectSession::spawn(
                ApiSpawnConfig::new(executable)
                    .with_mode(api_mode)
                    .with_cwd(&session_root),
                config_path_wire.as_str(),
                Some(virtual_file_wire.as_str().into()),
            ))
        )
        .map_err(|error| {
            compact_error(
                "Failed to start corsa type-aware session",
                error.to_compact_string().as_str(),
            )
        })?;
        let supports_overlay_updates = profile!(
            "patina.corsa_session.describe_capabilities",
            block_on(session.describe_capabilities())
        )
        .map(|capabilities| capabilities.overlay.update_snapshot_overlay_changes)
        .unwrap_or(false);

        cleanup_guard.disarm();
        let published =
            super::virtual_documents::PublishedDocuments::primed(virtual_file_wire.clone());
        Ok(Self {
            session,
            project_root,
            tsconfig_path,
            session_root,
            virtual_file_wire,
            virtual_file_path,
            supports_overlay_updates,
            overlay_version: 0,
            dependency_paths: vize_l0::FxHashSet::default(),
            dependency_cache: vize_l0::FxHashMap::default(),
            vue_path_aliases,
            active_filename: None,
            opened_documents: vize_l0::FxHashMap::default(),
            published,
            #[cfg(test)]
            last_refresh: super::virtual_documents::RefreshCounts::default(),
            closed: false,
        })
    }

    pub(in crate::linter) fn matches_source_file(
        &self,
        filename: &str,
        tsconfig_path: Option<&std::path::Path>,
    ) -> bool {
        self.project_root == resolve_project_root(filename)
            && self.tsconfig_path.as_deref() == tsconfig_path
    }

    pub(in crate::linter) fn open_virtual_project(
        &mut self,
        generated_source: &str,
        filename: &str,
    ) -> Result<(), String> {
        // The open snapshot already types this generated document.
        let input_hash = virtual_source_hash(generated_source);
        if self
            .opened_documents
            .get(filename)
            .is_some_and(|document| document.input_hash == input_hash)
        {
            self.retarget_virtual_file(filename);
            self.active_filename = Some(filename.to_compact_string());
            return Ok(());
        }

        let next_path = virtual_file_path(&self.session_root, &self.project_root, filename);
        let prepared = super::vue_dependencies::prepare(
            generated_source,
            filename,
            &self.session_root,
            &self.project_root,
            &self.vue_path_aliases,
            &mut self.dependency_cache,
            &self.dependency_paths,
        );
        let mut dependency_changes = super::vue_dependencies::materialize(&prepared.files, &[])?;
        self.dependency_paths
            .extend(prepared.files.iter().map(|(path, _)| path.clone()));
        let document = super::virtual_documents::OpenedDocument {
            input_hash,
            rewritten_source: prepared.code,
            import_source_map: prepared.source_map,
        };
        let generated_source = document
            .rewritten_source
            .as_deref()
            .unwrap_or(generated_source);
        let mut virtual_changes = profile!(
            "patina.corsa_session.write_virtual_file",
            super::virtual_documents::sync_stable_virtual_file(
                &mut self.published,
                &next_path,
                generated_source,
                self.supports_overlay_updates,
                &[],
            )
        )?;
        self.virtual_file_path = next_path;
        self.virtual_file_wire = path_to_wire(&self.virtual_file_path);
        dependency_changes
            .created
            .append(&mut virtual_changes.created);
        dependency_changes
            .changed
            .append(&mut virtual_changes.changed);
        dependency_changes
            .deleted
            .append(&mut virtual_changes.deleted);
        if dependency_changes.changed.is_empty()
            && dependency_changes.created.is_empty()
            && dependency_changes.deleted.is_empty()
            && !self.supports_overlay_updates
        {
            dependency_changes
                .changed
                .push(self.virtual_file_wire.as_str().into());
        }
        self.note_refresh(&dependency_changes);
        let file_changes = (!dependency_changes.changed.is_empty()
            || !dependency_changes.created.is_empty()
            || !dependency_changes.deleted.is_empty())
        .then_some(FileChanges::Summary(dependency_changes));

        if self.supports_overlay_updates {
            self.overlay_version = self.overlay_version.saturating_add(1);
            profile!(
                "patina.corsa_session.refresh_overlay",
                block_on(self.session.refresh_with_overlay_changes(
                    file_changes,
                    Some(OverlayChanges {
                        upsert: vec![OverlayUpdate {
                            document: self.virtual_file_wire.as_str().into(),
                            text: generated_source.into(),
                            version: Some(self.overlay_version),
                            language_id: Some("typescript".into()),
                        }],
                        delete: Vec::new(),
                    }),
                ))
            )
            .map_err(|error| {
                compact_error(
                    "Failed to update patina type snapshot",
                    error.to_compact_string().as_str(),
                )
            })?;
            self.remember_virtual_input(filename, document);
            return Ok(());
        }

        profile!(
            "patina.corsa_session.refresh_file",
            block_on(self.session.refresh(file_changes))
        )
        .map_err(|error| {
            compact_error(
                "Failed to update patina type snapshot",
                error.to_compact_string().as_str(),
            )
        })?;
        self.remember_virtual_input(filename, document);
        Ok(())
    }

    fn retarget_virtual_file(&mut self, filename: &str) {
        self.virtual_file_path =
            virtual_file_path(&self.session_root, &self.project_root, filename);
        self.virtual_file_wire = path_to_wire(&self.virtual_file_path);
    }

    fn note_refresh(&mut self, summary: &FileChangeSummary) {
        let counts = super::virtual_documents::RefreshCounts::from_summary(summary);
        #[cfg(test)]
        {
            self.last_refresh = counts;
        }
        let profiler = vize_l0::profiler::global_profiler();
        profiler.record_counter(
            "patina.corsa_session.refresh.created",
            counts.created as u64,
        );
        profiler.record_counter(
            "patina.corsa_session.refresh.changed",
            counts.changed as u64,
        );
        profiler.record_counter(
            "patina.corsa_session.refresh.deleted",
            counts.deleted as u64,
        );
    }

    #[cfg(test)]
    pub(super) fn refresh_counts(&self) -> super::virtual_documents::RefreshCounts {
        self.last_refresh
    }

    fn remember_virtual_input(
        &mut self,
        filename: &str,
        document: super::virtual_documents::OpenedDocument,
    ) {
        self.opened_documents
            .insert(filename.to_compact_string(), document);
        self.active_filename = Some(filename.to_compact_string());
    }

    pub(super) fn active_document(&self) -> Option<&super::virtual_documents::OpenedDocument> {
        self.active_filename
            .as_deref()
            .and_then(|filename| self.opened_documents.get(filename))
    }

    pub(in crate::linter) fn close(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        // Remove the directory even when closing the runtime panics. A successful
        // lint used to return while this process's session was still live, so
        // stale-pid cleanup left `.vize/patina/session-*` on disk.
        let _cleanup = SessionRootCleanup::new(self.session_root.clone());
        let _ = block_on(self.session.close());
    }
}

struct SessionRootCleanup {
    path: Option<std::path::PathBuf>,
}

impl SessionRootCleanup {
    fn new(path: std::path::PathBuf) -> Self {
        Self { path: Some(path) }
    }

    /// Keep the session root on disk: the session now owns it.
    fn disarm(mut self) {
        self.path = None;
    }
}

impl Drop for SessionRootCleanup {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            remove_session_root(path);
        }
    }
}

fn virtual_source_hash(source: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

fn api_mode_for_executable(path: &std::path::Path) -> ApiMode {
    if uses_async_json_rpc_api(path) {
        return ApiMode::AsyncJsonRpcStdio;
    }

    ApiMode::SyncMsgpackStdio
}

#[cfg(test)]
mod tests;
