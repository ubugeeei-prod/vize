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
use vize_carton::corsa_api_mode::uses_async_json_rpc_api;
use vize_l0::{String, ToCompactString, profile};

impl CorsaTypeAwareSession {
    pub(in crate::linter) fn new_with_corsa_path(
        filename: &str,
        corsa_path: Option<&std::path::Path>,
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
        profile!(
            "patina.corsa_session.write_tsconfig",
            std::fs::write(
                &config_path,
                session_tsconfig_contents(&project_root, filename).as_str(),
            )
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
        Ok(Self {
            session,
            project_root,
            session_root,
            virtual_file_wire,
            virtual_file_path,
            supports_overlay_updates,
            overlay_version: 0,
            rewritten_source: None,
            import_source_map: vize_canon::ImportSourceMap::empty(),
            dependency_paths: Vec::new(),
            dependency_cache: vize_l0::FxHashMap::default(),
            closed: false,
        })
    }

    pub(in crate::linter) fn matches_source_file(&self, filename: &str) -> bool {
        self.project_root == resolve_project_root(filename)
    }

    pub(in crate::linter) fn open_virtual_project(
        &mut self,
        generated_source: &str,
        filename: &str,
    ) -> Result<(), String> {
        let next_path = virtual_file_path(&self.session_root, &self.project_root, filename);
        let next_existed = next_path.exists();
        let prepared = super::vue_dependencies::prepare(
            generated_source,
            filename,
            &self.session_root,
            &self.project_root,
            &mut self.dependency_cache,
        );
        let prior_dependencies: Vec<_> = self
            .dependency_paths
            .iter()
            .filter(|path| **path != next_path)
            .cloned()
            .collect();
        let mut dependency_changes =
            super::vue_dependencies::materialize(&prepared.files, &prior_dependencies)?;
        self.dependency_paths = prepared
            .files
            .iter()
            .map(|(path, _)| path.clone())
            .collect();
        self.import_source_map = prepared.source_map;
        self.rewritten_source = prepared.code;
        let generated_source = self.rewritten_source.as_deref().unwrap_or(generated_source);
        let previous_wire = if next_path == self.virtual_file_path {
            None
        } else {
            if let Some(parent) = next_path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    io_error_message(
                        "Failed to create patina virtual directory",
                        &next_path,
                        &error,
                    )
                })?;
            }
            std::fs::write(&next_path, generated_source).map_err(|error| {
                io_error_message(
                    "Failed to write patina virtual TypeScript",
                    &next_path,
                    &error,
                )
            })?;
            let previous_wire =
                std::mem::replace(&mut self.virtual_file_wire, path_to_wire(&next_path));
            let previous_path = std::mem::replace(&mut self.virtual_file_path, next_path);
            if !self.dependency_paths.contains(&previous_path) {
                let _ = std::fs::remove_file(previous_path);
            }
            Some(previous_wire)
        };
        if let Some(previous) = &previous_wire {
            if next_existed {
                dependency_changes
                    .changed
                    .push(self.virtual_file_wire.as_str().into());
            } else {
                dependency_changes
                    .created
                    .push(self.virtual_file_wire.as_str().into());
            }
            if !self
                .dependency_paths
                .iter()
                .any(|path| path_to_wire(path) == *previous)
            {
                dependency_changes.deleted.push(previous.as_str().into());
            }
        } else if !self.supports_overlay_updates {
            dependency_changes
                .changed
                .push(self.virtual_file_wire.as_str().into());
        }
        let file_changes = (!dependency_changes.changed.is_empty()
            || !dependency_changes.created.is_empty()
            || !dependency_changes.deleted.is_empty())
        .then_some(FileChanges::Summary(dependency_changes));

        if self.supports_overlay_updates {
            self.overlay_version = self.overlay_version.saturating_add(1);
            return profile!(
                "patina.corsa_session.refresh_overlay",
                block_on(
                    self.session.refresh_with_overlay_changes(
                        file_changes,
                        Some(OverlayChanges {
                            upsert: vec![OverlayUpdate {
                                document: self.virtual_file_wire.as_str().into(),
                                text: generated_source.into(),
                                version: Some(self.overlay_version),
                                language_id: Some("typescript".into()),
                            }],
                            delete: previous_wire
                                .iter()
                                .map(|previous| previous.as_str().into())
                                .collect(),
                        }),
                    )
                )
            )
            .map_err(|error| {
                compact_error(
                    "Failed to update patina type snapshot",
                    error.to_compact_string().as_str(),
                )
            });
        }

        profile!(
            "patina.corsa_session.write_virtual_file",
            std::fs::write(&self.virtual_file_path, generated_source)
        )
        .map_err(|error| {
            io_error_message(
                "Failed to write patina virtual TypeScript",
                &self.virtual_file_path,
                &error,
            )
        })?;

        profile!(
            "patina.corsa_session.refresh_file",
            block_on(self.session.refresh(file_changes.or_else(|| {
                Some(FileChanges::Summary(FileChangeSummary {
                    changed: vec![self.virtual_file_wire.as_str().into()],
                    created: Vec::new(),
                    deleted: Vec::new(),
                }))
            })),)
        )
        .map_err(|error| {
            compact_error(
                "Failed to update patina type snapshot",
                error.to_compact_string().as_str(),
            )
        })?;
        Ok(())
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

fn api_mode_for_executable(path: &std::path::Path) -> ApiMode {
    if uses_async_json_rpc_api(path) {
        return ApiMode::AsyncJsonRpcStdio;
    }

    ApiMode::SyncMsgpackStdio
}

#[cfg(test)]
mod tests;
