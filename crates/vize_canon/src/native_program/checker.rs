//! One existing Corsa bridge and a fixed, genuine workspace configuration.

use super::{NativeProgramCheck, mapping};
use crate::corsa_session_cache::tsconfig_chain_digest;
use crate::{CorsaBridge, CorsaBridgeConfig, CorsaBridgeError, CorsaError, VIRTUAL_URI_SCHEME};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use vize_l0::{String, cstr};
use vize_l4::targets::ts::{ProgramProjection, SourceKind};

#[derive(Debug, thiserror::Error)]
pub enum NativeProgramError {
    #[error("native Program checking requires an explicit real workspace")]
    WorkspaceRequired,
    #[error("native Program checking requires an existing tsconfig.json or jsconfig.json")]
    WorkspaceConfigurationRequired,
    #[error("native Program workspace cannot be resolved: {0}")]
    Workspace(#[source] std::io::Error),
    #[error("native Program configuration failed: {0}")]
    Configuration(#[source] CorsaError),
    #[error("workspace configuration changed; construct a fresh checker")]
    ConfigurationChanged,
    #[error("original JavaScript requires allowJs and checkJs in the actual configuration")]
    JavaScriptCheckingDisabled,
    #[error("imported Programs require a provider-owned authored filename")]
    ImportedProgram,
    #[error("original invocations require a provider-owned authored filename")]
    InvokedProgram,
    #[error("commented Programs require a provider-owned authored filename")]
    CommentedProgram,
    #[error("native Program checker identity space is exhausted")]
    IdentityLimit,
    #[error("native Program backend failed: {error}")]
    Backend {
        #[source]
        error: CorsaBridgeError,
        cleanup: Option<CorsaBridgeError>,
    },
}

/// Opt-in adapter; no public bridge or caller-selected document kind/path.
pub struct NativeProgramChecker {
    bridge: CorsaBridge,
    root: PathBuf,
    config: PathBuf,
    digest: String,
    identity: usize,
    configuration_changed: bool,
}

impl NativeProgramChecker {
    pub fn with_config(mut config: CorsaBridgeConfig) -> Result<Self, NativeProgramError> {
        let root = config
            .working_dir
            .as_ref()
            .ok_or(NativeProgramError::WorkspaceRequired)?
            .canonicalize()
            .map_err(NativeProgramError::Workspace)?;
        let project_config =
            selected_config(&root).ok_or(NativeProgramError::WorkspaceConfigurationRequired)?;
        crate::snapshot_tsconfig_compiler_options(&root, &project_config)
            .map_err(NativeProgramError::Configuration)?;
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let identity = NEXT_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| NativeProgramError::IdentityLimit)?;
        config.working_dir = Some(root.clone());
        Ok(Self {
            bridge: CorsaBridge::with_config(config),
            root,
            digest: tsconfig_chain_digest(&project_config),
            config: project_config,
            identity,
            configuration_changed: false,
        })
    }

    /// Check only the genuine provider. Imports remain refused until filename
    /// authority exists; no relative-module path is fabricated from source text.
    pub async fn check<'p, 'f, 'a>(
        &mut self,
        projection: &'p ProgramProjection<'f, 'a>,
    ) -> Result<NativeProgramCheck<'p, 'f, 'a>, NativeProgramError> {
        if self.configuration_changed || !self.configuration_matches() {
            self.configuration_changed = true;
            return Err(NativeProgramError::ConfigurationChanged);
        }
        if !projection.file().imports().is_empty()
            || projection
                .file()
                .exports()
                .iter()
                .any(|export| export.source.is_some())
        {
            return Err(NativeProgramError::ImportedProgram);
        }
        if projection.unit().has_invocations() {
            return Err(NativeProgramError::InvokedProgram);
        }
        if projection.unit().has_comments() {
            return Err(NativeProgramError::CommentedProgram);
        }
        let options = crate::snapshot_tsconfig_compiler_options(&self.root, &self.config)
            .map_err(NativeProgramError::Configuration)?;
        if projection.source_kind() == SourceKind::JavaScript
            && !(options.get("allowJs").and_then(serde_json::Value::as_bool) == Some(true)
                && options.get("checkJs").and_then(serde_json::Value::as_bool) == Some(true))
        {
            return Err(NativeProgramError::JavaScriptCheckingDisabled);
        }
        self.bridge
            .ensure_configured_virtual_project()
            .await
            .map_err(|error| NativeProgramError::Backend {
                error,
                cleanup: None,
            })?;
        let name = cstr!(
            "program-check-{}-{}.{}",
            std::process::id(),
            self.identity,
            projection.source_kind().extension()
        );
        let expected_uri = cstr!("{VIRTUAL_URI_SCHEME}://{name}");
        let uri = match self
            .bridge
            .open_virtual_document(&name, projection.document().as_str())
            .await
        {
            Ok(uri) => uri,
            Err(error) => {
                let cleanup = self
                    .bridge
                    .close_virtual_document(&expected_uri)
                    .await
                    .err();
                return Err(NativeProgramError::Backend { error, cleanup });
            }
        };
        let diagnostics = self.bridge.get_diagnostics(&uri).await;
        let cleanup_error = self.bridge.close_virtual_document(&uri).await.err();
        let diagnostics = diagnostics.map_err(|error| NativeProgramError::Backend {
            error,
            cleanup: cleanup_error.clone(),
        })?;
        if !self.configuration_matches() {
            self.configuration_changed = true;
        }
        Ok(NativeProgramCheck {
            projection,
            diagnostics: diagnostics
                .into_iter()
                .map(|diagnostic| mapping::observe(projection, diagnostic))
                .collect(),
            cleanup_error,
            configuration_changed: self.configuration_changed,
        })
    }

    pub async fn shutdown(&self) -> Result<(), CorsaBridgeError> {
        self.bridge.shutdown().await
    }

    fn configuration_matches(&self) -> bool {
        selected_config(&self.root).as_ref() == Some(&self.config)
            && tsconfig_chain_digest(&self.config) == self.digest
    }
}

fn selected_config(root: &Path) -> Option<PathBuf> {
    ["tsconfig.json", "jsconfig.json"]
        .into_iter()
        .map(|name| root.join(name))
        .find(|path| path.is_file())
}
