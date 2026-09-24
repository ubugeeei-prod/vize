use super::{
    CorsaTypeAwareSession,
    errors::{compact_error, io_error_message},
    paths::{
        TSCONFIG_CONTENTS, TSCONFIG_FILE_NAME, allocate_session_root, path_to_wire,
        remove_session_root, resolve_corsa_executable, resolve_project_root, virtual_file_path,
    },
};
use corsa::{
    api::{
        ApiMode, ApiSpawnConfig, FileChangeSummary, FileChanges, OverlayChanges, OverlayUpdate,
        ProjectSession,
    },
    runtime::block_on,
};
use vize_s0::{String, ToCompactString, corsa_api_mode::uses_async_json_rpc_api, profile};

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
            std::fs::write(&config_path, TSCONFIG_CONTENTS)
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
            let _ = std::fs::remove_file(previous_path);
            Some(previous_wire)
        };
        let file_changes = previous_wire.as_ref().map(|previous| {
            FileChanges::Summary(FileChangeSummary {
                changed: Vec::new(),
                created: vec![self.virtual_file_wire.as_str().into()],
                deleted: vec![previous.as_str().into()],
            })
        });

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
        let _ = block_on(self.session.close());
        remove_session_root(&self.session_root);
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
