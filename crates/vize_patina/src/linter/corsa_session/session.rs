use super::{
    errors::{compact_error, io_error_message},
    paths::{
        allocate_session_root, path_to_wire, resolve_corsa_executable, resolve_project_root,
        TSCONFIG_CONTENTS, TSCONFIG_FILE_NAME, VIRTUAL_FILE_NAME,
    },
    CorsaTypeAwareSession,
};
use corsa::{
    api::{
        ApiMode, ApiSpawnConfig, FileChangeSummary, FileChanges, OverlayChanges, OverlayUpdate,
        ProjectSession,
    },
    runtime::block_on,
};
use vize_carton::{profile, String, ToCompactString};

impl CorsaTypeAwareSession {
    pub(in crate::linter) fn new(filename: &str) -> Result<Self, String> {
        let project_root = resolve_project_root(filename);
        let session_root = allocate_session_root(&project_root);
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

        let virtual_file_path = session_root.join(VIRTUAL_FILE_NAME);
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
        let executable = resolve_corsa_executable(&project_root);
        let session = profile!(
            "patina.corsa_session.spawn",
            block_on(ProjectSession::spawn(
                ApiSpawnConfig::new(executable)
                    .with_mode(ApiMode::SyncMsgpackStdio)
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
    ) -> Result<(), String> {
        if self.supports_overlay_updates {
            self.overlay_version = self.overlay_version.saturating_add(1);
            return profile!(
                "patina.corsa_session.refresh_overlay",
                block_on(self.session.refresh_with_overlay_changes(
                    None,
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
            block_on(
                self.session
                    .refresh(Some(FileChanges::Summary(FileChangeSummary {
                        changed: vec![self.virtual_file_wire.as_str().into()],
                        created: Vec::new(),
                        deleted: Vec::new(),
                    }))),
            )
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
        let _ = std::fs::remove_dir_all(&self.session_root);
    }
}
