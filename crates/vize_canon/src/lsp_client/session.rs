#![allow(clippy::disallowed_types)]

use super::{
    CorsaProjectClient,
    session_paths::{build_materialized_session_document_uri, build_session_document_uri},
    utils::remap_serialized_uris,
    virtual_overlay,
};
use crate::file_uri::file_uri_to_path;
use corsa::{
    CorsaError,
    api::{
        ApiMode, ApiSpawnConfig, CapabilitiesResponse, DocumentIdentifier, FileChangeSummary,
        FileChanges, OverlayChanges, OverlayUpdate, ProjectSession,
    },
    fast::CompactString,
    runtime::block_on,
};
use lsp_types::Diagnostic;
use serde_json::Value;
use std::{path::Path, sync::Arc};
use vize_carton::{String, cstr};

pub(super) fn spawn_project_session(
    executable: &str,
    cwd: &Path,
    config_path: &Path,
) -> Result<(ProjectSession, Arc<CapabilitiesResponse>), ProjectSessionSpawnError> {
    let config_path_wire = config_path.to_string_lossy();
    let mode = api_mode_for_executable(executable);
    let session = match block_on(spawn_project_session_with_mode(
        executable,
        cwd,
        config_path_wire.as_ref(),
        mode,
    )) {
        Ok(session) => session,
        Err(error) if should_retry_json_rpc(mode, &error) => {
            match block_on(spawn_project_session_with_mode(
                executable,
                cwd,
                config_path_wire.as_ref(),
                ApiMode::AsyncJsonRpcStdio,
            )) {
                Ok(session) => session,
                Err(fallback) => {
                    return Err(classify_project_session_error(
                        fallback,
                        Some(cstr!("after msgpack error: {error}")),
                    ));
                }
            }
        }
        Err(error) => return Err(classify_project_session_error(error, None)),
    };
    let capabilities = block_on(session.describe_capabilities())
        .unwrap_or_else(|_| Arc::new(CapabilitiesResponse::default()));
    Ok((session, capabilities))
}

#[derive(Debug)]
pub(super) enum ProjectSessionSpawnError {
    Unavailable(String),
    Failed(String),
}

fn classify_project_session_error(
    error: CorsaError,
    context: Option<String>,
) -> ProjectSessionSpawnError {
    let message = context.map_or_else(
        || cstr!("Failed to start Corsa API session: {error}"),
        |context| cstr!("Failed to start Corsa API session: {error} ({context})"),
    );
    if matches!(
        &error,
        CorsaError::Protocol(detail)
            if detail.contains("project session did not resolve a project")
    ) {
        ProjectSessionSpawnError::Unavailable(message)
    } else {
        ProjectSessionSpawnError::Failed(message)
    }
}

async fn spawn_project_session_with_mode(
    executable: &str,
    cwd: &Path,
    config_path: &str,
    mode: ApiMode,
) -> Result<ProjectSession, CorsaError> {
    ProjectSession::spawn(
        ApiSpawnConfig::new(executable)
            .with_mode(mode)
            .with_cwd(cwd),
        config_path,
        None,
    )
    .await
}

fn should_retry_json_rpc(mode: ApiMode, error: &CorsaError) -> bool {
    if mode != ApiMode::SyncMsgpackStdio {
        return false;
    }

    let CorsaError::Protocol(message) = error else {
        return false;
    };

    let message = message.as_str();
    message.contains("expected tuple marker")
        || message.contains("expected uint8 marker")
        || message.contains("expected bin marker")
}

fn api_mode_for_executable(executable: &str) -> ApiMode {
    if is_node_wrapper_executable(Path::new(executable)) {
        ApiMode::AsyncJsonRpcStdio
    } else {
        ApiMode::SyncMsgpackStdio
    }
}

fn is_node_wrapper_executable(path: &Path) -> bool {
    if path.extension().and_then(|extension| extension.to_str()) == Some("js") {
        return true;
    }

    if path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        == Some(".bin")
    {
        return true;
    }

    let Some(parent) = path.parent() else {
        return false;
    };
    let Some(grandparent) = parent.parent() else {
        return false;
    };

    parent.file_name().and_then(|name| name.to_str()) == Some("bin")
        && grandparent.file_name().and_then(|name| name.to_str()) == Some("native-preview")
}

pub(super) fn uri_document_identifier(uri: &str) -> DocumentIdentifier {
    if let Some(path) = file_uri_to_path(uri) {
        let path = path.to_string_lossy();
        return DocumentIdentifier::FileName(CompactString::from(path.as_ref()));
    }

    DocumentIdentifier::Uri {
        uri: CompactString::from(uri),
    }
}

impl CorsaProjectClient {
    pub(super) fn has_project_session(&self) -> bool {
        self.session.is_some()
    }

    pub(super) fn project_session(&self) -> Result<&ProjectSession, String> {
        self.session
            .as_ref()
            .ok_or_else(|| cstr!("Corsa project-session API is unavailable"))
    }

    pub(super) fn project_session_mut(&mut self) -> Result<&mut ProjectSession, String> {
        self.session
            .as_mut()
            .ok_or_else(|| cstr!("Corsa project-session API is unavailable"))
    }

    fn trusts_capabilities(&self) -> bool {
        self.capabilities.runtime.capability_endpoint
    }

    pub(super) fn supports_overlay_api(&self) -> bool {
        if !self.has_project_session() {
            return true;
        }
        !self.overlay_api_disabled
            && (!self.trusts_capabilities()
                || self.capabilities.overlay.update_snapshot_overlay_changes)
    }

    pub(super) fn supports_project_diagnostics_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.diagnostics.project)
    }

    pub(super) fn supports_file_diagnostics_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.diagnostics.file)
    }

    pub(super) fn supports_hover_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.editor.hover)
    }

    pub(super) fn supports_definition_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.editor.definition)
    }

    pub(super) fn supports_references_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.editor.references)
    }

    pub(super) fn supports_rename_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.editor.rename)
    }

    pub(super) fn supports_completion_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.editor.completion)
    }

    pub(super) fn can_use_api_for_uri(&self, uri: &str) -> bool {
        !self.document_texts.contains_key(uri) || self.supports_overlay_api()
    }

    pub(super) fn sync_overlay_document(&mut self, uri: &str, content: &str) -> Result<(), String> {
        let previous = self.document_texts.insert(uri.into(), content.into());
        if previous.as_deref() != Some(content) {
            self.editor_lsp_documents_dirty = true;
        }

        if self.materialized_project_session {
            return self.sync_materialized_overlay_document(uri, content);
        }

        if !self.has_project_session() {
            return Ok(());
        }

        let document_uri = self.session_document_uri(uri);
        if previous.as_deref() == Some(content) {
            return Ok(());
        }

        if document_uri != uri || !self.supports_overlay_api() {
            return self.sync_materialized_overlay_document(uri, content);
        }

        let file_changes = materialize_session_document(uri, document_uri.as_str(), content)
            .or_else(|| {
                virtual_overlay::upsert_file_changes(
                    uri,
                    document_uri.as_str(),
                    &self.project_root,
                    previous.is_some(),
                )
            });
        let version = next_overlay_version(&mut self.overlay_versions, uri);
        match block_on(self.project_session_mut()?.refresh_with_overlay_changes(
            file_changes,
            Some(OverlayChanges {
                upsert: vec![OverlayUpdate {
                    document: uri_document_identifier(document_uri.as_str()),
                    text: content.into(),
                    version: Some(version),
                    language_id: Some(super::language_id::for_uri(document_uri.as_str()).into()),
                }],
                delete: Vec::new(),
            }),
        )) {
            Ok(()) => Ok(()),
            Err(error) if overlay_changes_error_is_unsupported(&error) => {
                self.overlay_api_disabled = true;
                self.sync_materialized_overlay_document(uri, content)
            }
            Err(error) => Err(cstr!("Failed to sync Corsa overlay: {error}")),
        }
    }

    fn sync_materialized_overlay_document(
        &mut self,
        uri: &str,
        content: &str,
    ) -> Result<(), String> {
        self.activate_materialized_project_session()?;
        let document_uri = self
            .materialized_session_document_uri(uri)
            .ok_or_else(|| cstr!("Failed to derive materialized Corsa overlay path for {uri}"))?;
        let file_changes = materialize_session_document(uri, document_uri.as_str(), content);
        if !self.has_project_session() {
            return Ok(());
        }
        block_on(self.project_session_mut()?.refresh(file_changes))
            .map_err(|error| cstr!("Failed to refresh Corsa snapshot: {error}"))
    }

    pub(super) fn delete_overlay_document(&mut self, uri: &str) -> Result<(), String> {
        if self.document_texts.remove(uri).is_some() {
            self.editor_lsp_documents_dirty = true;
        }
        self.overlay_versions.remove(uri);
        let document_uri = self
            .session_document_uris
            .remove(uri)
            .unwrap_or_else(|| self.session_document_uri(uri));
        self.external_document_uris.remove(document_uri.as_str());
        let file_changes = remove_session_document(uri, document_uri.as_str()).or_else(|| {
            virtual_overlay::delete_file_changes(uri, document_uri.as_str(), &self.project_root)
        });
        if !self.has_project_session() {
            return Ok(());
        }
        if document_uri != uri {
            return block_on(self.project_session_mut()?.refresh(file_changes))
                .map_err(|error| cstr!("Failed to refresh Corsa snapshot: {error}"));
        }

        if !self.supports_overlay_api() {
            return Ok(());
        }

        block_on(self.project_session_mut()?.refresh_with_overlay_changes(
            file_changes,
            Some(OverlayChanges {
                upsert: Vec::new(),
                delete: vec![uri_document_identifier(document_uri.as_str())],
            }),
        ))
        .map_err(|error| cstr!("Failed to remove Corsa overlay: {error}"))
    }

    pub(super) fn utf16_offset_for(&self, uri: &str, line: u32, character: u32) -> Option<u32> {
        self.document_texts
            .get(uri)
            .map(|content| line_character_to_utf16_offset(content.as_str(), line, character))
            .or_else(|| {
                load_file_text(uri)
                    .as_deref()
                    .map(|content| line_character_to_utf16_offset(content, line, character))
            })
    }

    pub(super) fn session_document_uri(&mut self, uri: &str) -> String {
        if let Some(mapped) = self.session_document_uris.get(uri) {
            return mapped.clone();
        }

        let mapped =
            build_session_document_uri(uri, &self.project_root, self.supports_overlay_api());
        self.remember_session_document_uri(uri, mapped)
    }

    fn materialized_session_document_uri(&mut self, uri: &str) -> Option<String> {
        let mapped = build_materialized_session_document_uri(uri, &self.project_root)?;
        Some(self.remember_session_document_uri(uri, mapped))
    }

    pub(super) fn remember_session_document_uri(&mut self, uri: &str, mapped: String) -> String {
        if let Some(previous) = self
            .session_document_uris
            .insert(uri.into(), mapped.clone())
        {
            self.external_document_uris.remove(previous.as_str());
        }
        self.external_document_uris
            .insert(mapped.clone(), uri.into());
        mapped
    }

    pub(super) fn remap_diagnostics(&self, diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
        remap_serialized_uris(diagnostics.clone(), &self.external_document_uris)
            .unwrap_or(diagnostics)
    }

    pub(super) fn remap_result_uris(&self, value: &mut Value) {
        super::utils::remap_json_uris(value, &self.external_document_uris);
    }
}

fn next_overlay_version(versions: &mut vize_carton::FxHashMap<String, i32>, uri: &str) -> i32 {
    let next = versions.get(uri).copied().unwrap_or(0).saturating_add(1);
    versions.insert(uri.into(), next);
    next
}

fn load_file_text(uri: &str) -> Option<String> {
    let path = file_uri_to_path(uri)?;
    std::fs::read_to_string(path).ok().map(Into::into)
}

/// The runtime advertised overlay support through `describeCapabilities`, but
/// rejected the overlay write at request time. corsa-bind surfaces this as the
/// typed `CorsaError::Unsupported` variant (also for runtimes that lack the
/// `updateSnapshot.overlayChanges` method entirely, normalized from the RPC
/// "method not found" error), so we gate the materialized fallback on the
/// variant rather than sniffing the human-readable message text.
fn overlay_changes_error_is_unsupported(error: &CorsaError) -> bool {
    matches!(error, CorsaError::Unsupported(_))
}

pub(super) fn materialize_session_document(
    external_uri: &str,
    document_uri: &str,
    content: &str,
) -> Option<FileChanges> {
    if document_uri == external_uri {
        return None;
    }

    let path = file_uri_to_path(document_uri)?;
    let path = path.as_path();
    let parent = path.parent()?;
    let existed = path.exists();
    let _ = std::fs::create_dir_all(parent);
    let _ = std::fs::write(path, content);

    Some(FileChanges::Summary(FileChangeSummary {
        changed: if existed {
            vec![uri_document_identifier(document_uri)]
        } else {
            Vec::new()
        },
        created: if existed {
            Vec::new()
        } else {
            vec![uri_document_identifier(document_uri)]
        },
        deleted: Vec::new(),
    }))
}

fn remove_session_document(external_uri: &str, document_uri: &str) -> Option<FileChanges> {
    if document_uri == external_uri {
        return None;
    }

    let path = file_uri_to_path(document_uri)?;
    let path = path.as_path();
    if !path.exists() {
        return None;
    }

    let _ = std::fs::remove_file(path);
    Some(FileChanges::Summary(FileChangeSummary {
        changed: Vec::new(),
        created: Vec::new(),
        deleted: vec![uri_document_identifier(document_uri)],
    }))
}

pub(super) fn line_character_to_utf16_offset(text: &str, line: u32, character: u32) -> u32 {
    let mut offset = 0u32;
    let mut lines = text.split_inclusive('\n');

    for _ in 0..line {
        let Some(segment) = lines.next() else {
            return text.encode_utf16().count() as u32;
        };
        offset += segment.encode_utf16().count() as u32;
    }

    let Some(segment) = lines.next() else {
        return text.encode_utf16().count() as u32;
    };
    let line_without_break = segment.strip_suffix('\n').unwrap_or(segment);
    let line_len = line_without_break.encode_utf16().count() as u32;
    offset + character.min(line_len)
}

#[cfg(test)]
mod tests;
