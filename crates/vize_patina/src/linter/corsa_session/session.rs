use super::{
    CorsaTypeAwareSession,
    errors::{compact_error, io_error_message},
    paths::{
        TSCONFIG_CONTENTS, TSCONFIG_FILE_NAME, VIRTUAL_FILE_NAME, allocate_session_root,
        path_to_wire, remove_session_root, resolve_corsa_executable, resolve_project_root,
    },
};
use corsa::{
    api::{
        ApiMode, ApiSpawnConfig, FileChangeSummary, FileChanges, OverlayChanges, OverlayUpdate,
        ProjectSession,
    },
    runtime::block_on,
};
use vize_carton::{String, ToCompactString, profile};

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

        let session_root = cleanup_guard.keep();
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

    fn keep(mut self) -> std::path::PathBuf {
        self.path.take().expect("session root cleanup path")
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
    if path.extension().and_then(|extension| extension.to_str()) == Some("js") {
        return ApiMode::AsyncJsonRpcStdio;
    }

    if path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        == Some(".bin")
    {
        return ApiMode::AsyncJsonRpcStdio;
    }

    let Some(parent) = path.parent() else {
        return ApiMode::SyncMsgpackStdio;
    };
    let Some(grandparent) = parent.parent() else {
        return ApiMode::SyncMsgpackStdio;
    };

    if parent.file_name().and_then(|name| name.to_str()) == Some("bin")
        && grandparent.file_name().and_then(|name| name.to_str()) == Some("native-preview")
    {
        ApiMode::AsyncJsonRpcStdio
    } else {
        ApiMode::SyncMsgpackStdio
    }
}

#[cfg(test)]
mod tests {
    use super::api_mode_for_executable;
    use crate::linter::corsa_session::CorsaTypeAwareSession;
    use corsa::api::ApiMode;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};
    use vize_carton::cstr;

    static NEXT_CASE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn uses_json_rpc_for_node_wrappers() {
        assert_eq!(
            api_mode_for_executable(Path::new("/workspace/node_modules/.bin/tsgo")),
            ApiMode::AsyncJsonRpcStdio
        );
        assert_eq!(
            api_mode_for_executable(Path::new(
                "/workspace/node_modules/@typescript/native-preview/bin/tsgo.js"
            )),
            ApiMode::AsyncJsonRpcStdio
        );
    }

    #[test]
    fn uses_msgpack_for_native_binaries() {
        assert_eq!(
            api_mode_for_executable(Path::new(
                "/workspace/node_modules/@typescript/native-preview-darwin-arm64/lib/tsgo"
            )),
            ApiMode::SyncMsgpackStdio
        );
    }

    #[test]
    fn cleans_session_root_when_spawn_fails() {
        let root = case_dir("spawn-fails");
        let _ = std::fs::remove_dir_all(&root);
        let source = root.join("Component.vue");
        let invalid_corsa = root.join("not-corsa");

        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("package.json"), "{}").unwrap();
        std::fs::write(&invalid_corsa, "").unwrap();

        let error = match CorsaTypeAwareSession::new_with_corsa_path(
            source.to_str().unwrap(),
            Some(invalid_corsa.as_path()),
        ) {
            Ok(mut session) => {
                session.close();
                panic!("invalid corsa executable unexpectedly started");
            }
            Err(error) => error,
        };

        assert!(error.contains("Failed to start corsa type-aware session"));
        assert!(!root.join(".vize").join("patina").exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    fn case_dir(name: &str) -> std::path::PathBuf {
        let id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join(&*cstr!(
                "patina-corsa-session-{name}-{}-{id}",
                std::process::id()
            ))
    }
}
