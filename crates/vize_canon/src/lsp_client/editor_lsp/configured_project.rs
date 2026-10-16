//! Explicit Batch config ownership in the diagnosing LSP's shared session.

use super::{EditorLspSession, project_configuration::ConfigurationError};
use corsa::{api::UpdateSnapshotParams, runtime::block_on};
use std::path::Path;
use vize_l0::{String, cstr};

#[cfg(test)]
mod batch_source_receipt;
mod shared_transport;
pub(in crate::lsp_client) use shared_transport::try_shared_editor_project;

impl EditorLspSession {
    #[cfg(test)]
    pub(super) fn spawn_with_config(
        executable: &str,
        cwd: &Path,
        project_root: &Path,
        config: Option<&Path>,
    ) -> Result<Self, String> {
        Self::spawn_configured(executable, cwd, project_root, config, false, None)
    }

    pub(super) fn spawn_with_config_controlled(
        executable: &str,
        cwd: &Path,
        project_root: &Path,
        config: Option<&Path>,
        control: Option<crate::lsp_client::startup_control::StartupControl>,
    ) -> Result<Self, String> {
        Self::spawn_configured(executable, cwd, project_root, config, false, control)
    }

    pub(super) fn spawn_with_startup_configuration(
        executable: &str,
        cwd: &Path,
        project_root: &Path,
        config: &Path,
        control: Option<crate::lsp_client::startup_control::StartupControl>,
    ) -> Result<Self, String> {
        Self::spawn_configured(executable, cwd, project_root, Some(config), true, control)
    }

    fn spawn_configured(
        executable: &str,
        cwd: &Path,
        project_root: &Path,
        config: Option<&Path>,
        apply_startup_configuration: bool,
        control: Option<crate::lsp_client::startup_control::StartupControl>,
    ) -> Result<Self, String> {
        if let Some(config) = config {
            validate_config_path(config)?;
        }
        let mut session = Self::spawn_initializing(executable, cwd, project_root, control)?;
        if let Some(config) = config
            && let Err(error) = session.open_explicit_project(config, apply_startup_configuration)
        {
            crate::lsp_client::retirement::ownership_error(
                &session.client.process_ownership(),
                error.as_str(),
            );
            // Startup errors use the same owner-before-attachment cleanup.
            let _ = session.shutdown();
            return Err(error);
        }
        if !apply_startup_configuration {
            session.finish_startup()?;
        }
        Ok(session)
    }

    fn open_explicit_project(
        &mut self,
        config: &Path,
        apply_startup_configuration: bool,
    ) -> Result<(), String> {
        let config = config
            .to_str()
            .ok_or_else(|| cstr!("Explicit Corsa project path is not UTF-8"))?;
        // Retain the actual LSP attachment before opening, including errors.
        // Closing a snapshot does not release this session's project ref.
        self.configured_api = Some(self.attach_api().map_err(configuration_error)?);
        if apply_startup_configuration {
            self.apply_startup_configuration(Path::new(config))?;
        }
        let api = self
            .configured_api
            .as_ref()
            .ok_or_else(|| cstr!("Explicit Corsa project attachment is missing"))?;
        let snapshot = block_on(api.client.update_snapshot(UpdateSnapshotParams {
            open_project: Some(config.into()),
            file_changes: None,
            overlay_changes: None,
        }))
        .map_err(|error| cstr!("Cannot open explicit diagnosing project: {error}"))?;
        block_on(snapshot.release())
            .map_err(|error| cstr!("Cannot release diagnosing startup snapshot: {error}"))
    }

    #[cfg(test)]
    pub(super) fn configured_project_receipt(
        &self,
        uri: &str,
    ) -> Result<serde_json::Value, String> {
        let api = self
            .configured_api
            .as_ref()
            .ok_or_else(|| cstr!("No retained diagnosing project attachment"))?;
        let snapshot = block_on(api.client.update_snapshot(Default::default()))
            .map_err(|error| cstr!("Cannot observe diagnosing snapshot: {error}"))?;
        let result = (|| {
            let project =
                block_on(snapshot.get_default_project_for_file(
                    super::super::session::uri_document_identifier(uri),
                ))
                .map_err(|error| cstr!("Cannot observe diagnosing project: {error}"))?
                .ok_or_else(|| cstr!("The diagnosing document has no default project"))?;
            let parsed = block_on(api.client.parse_config_file(
                corsa::api::DocumentIdentifier::from(project.config_file_name.as_str()),
            ))
            .map_err(|error| cstr!("Cannot observe normalized diagnosing config: {error}"))?;
            let source_request =
                serde_json::json!({"snapshot":snapshot.handle,"project":project.id});
            let source_response = match block_on(
                api.client
                    .raw_json_request("getSourceFileNames", source_request.clone()),
            ) {
                Ok(value) => serde_json::json!({"status":"ok","wholeResponse":value}),
                Err(error) => serde_json::json!({"status":"error","wholeError":cstr!("{error}")}),
            };
            Ok(serde_json::json!({
                "project": project, "snapshotProjects": snapshot.projects,
                "normalizedRequestedOptions": parsed.options,
                "normalizedRequestedFiles": parsed.file_names,
                "attachment": api.session,
                "sourceFileNamesRequest": source_request,
                "sourceFileNamesResponse": source_response,
            }))
        })();
        let cleanup = block_on(snapshot.release())
            .map_err(|error| cstr!("Cannot release diagnosing observation: {error}"));
        match (result, cleanup) {
            (Ok(receipt), Ok(())) => Ok(receipt),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }
}

fn validate_config_path(config: &Path) -> Result<(), String> {
    if !config.is_absolute() || !config.is_file() {
        return Err(cstr!(
            "Explicit Corsa project configuration is not an absolute existing file: {}",
            config.display()
        ));
    }
    Ok(())
}

pub(super) fn configuration_error(error: ConfigurationError) -> String {
    match error {
        ConfigurationError::Communication(error) => error,
        ConfigurationError::Unconfigured => cstr!("Explicit diagnosing project is unconfigured"),
        ConfigurationError::Changed => cstr!("Explicit diagnosing project options changed"),
        #[cfg(not(unix))]
        ConfigurationError::Unsupported => {
            super::super::EXPLICIT_CONFIG_ATTACHMENT_UNSUPPORTED.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_project_refuses_absent_relative_and_directory_config_paths() {
        let root = tempfile::tempdir().unwrap();
        let absent = root.path().join("absent.json");
        for config in [Path::new("tsconfig.json"), root.path(), &absent] {
            assert!(validate_config_path(config).is_err());
        }
        let config = root.path().join("tsconfig.json");
        std::fs::write(&config, "{}").unwrap();
        validate_config_path(&config).unwrap();
    }
}
