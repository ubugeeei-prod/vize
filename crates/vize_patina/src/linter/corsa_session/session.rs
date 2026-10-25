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
    api::{ApiMode, ApiSpawnConfig, FileChangeSummary, ProjectSession},
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
        Self::new_with_configure(filename, corsa_path, tsconfig_path, std::convert::identity)
    }

    fn new_with_configure(
        filename: &str,
        corsa_path: Option<&std::path::Path>,
        tsconfig_path: Option<std::path::PathBuf>,
        configure: impl FnOnce(ApiSpawnConfig) -> ApiSpawnConfig,
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
                vize_canon::corsa_profile::with_request_profiling(
                    configure(
                        ApiSpawnConfig::new(executable)
                            .with_mode(api_mode)
                            .with_cwd(&session_root)
                    ),
                    vize_canon::corsa_profile::RequestOrigin::Patina
                ),
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

        self.open_virtual_projects(&[(generated_source, filename)])
    }

    pub(in crate::linter) fn select_virtual_document(
        &mut self,
        filename: &str,
    ) -> Result<(), String> {
        if !self.opened_documents.contains_key(filename) {
            return Err(vize_l0::cstr!(
                "Unpublished patina probe document: {filename}"
            ));
        }
        self.retarget_virtual_file(filename);
        self.active_filename = Some(filename.to_compact_string());
        Ok(())
    }

    pub(super) fn retarget_virtual_file(&mut self, filename: &str) {
        self.virtual_file_path =
            virtual_file_path(&self.session_root, &self.project_root, filename);
        self.virtual_file_wire = path_to_wire(&self.virtual_file_path);
    }

    pub(super) fn note_refresh(&mut self, summary: &FileChangeSummary) {
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

    pub(super) fn remember_virtual_input(
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

pub(super) fn virtual_source_hash(source: &str) -> u64 {
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
