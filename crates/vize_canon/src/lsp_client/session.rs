#![allow(clippy::disallowed_types)]

use super::{utils::remap_serialized_uris, CorsaProjectClient};
use crate::file_uri::{file_uri_to_path, path_to_file_uri};
use corsa::{
    api::{
        ApiMode, ApiSpawnConfig, CapabilitiesResponse, DocumentIdentifier, FileChangeSummary,
        FileChanges, OverlayChanges, OverlayUpdate, ProjectSession,
    },
    error::TsgoError,
    fast::CompactString,
    runtime::block_on,
};
use lsp_types::Diagnostic;
use serde_json::Value;
use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};
use vize_carton::{cstr, String};

pub(super) fn spawn_project_session(
    executable: &str,
    cwd: &Path,
    project_root: &Path,
) -> Result<(ProjectSession, Arc<CapabilitiesResponse>), String> {
    let config_path = project_root.join("tsconfig.json");
    let config_path_wire = config_path.to_string_lossy();
    let mode = api_mode_for_executable(executable);
    let session = match block_on(spawn_project_session_with_mode(
        executable,
        cwd,
        config_path_wire.as_ref(),
        mode,
    )) {
        Ok(session) => session,
        Err(error) if should_retry_json_rpc(mode, &error) => block_on(
            spawn_project_session_with_mode(
                executable,
                cwd,
                config_path_wire.as_ref(),
                ApiMode::AsyncJsonRpcStdio,
            ),
        )
        .map_err(|fallback| {
            cstr!("Failed to start Corsa API session: {fallback} (after msgpack error: {error})")
        })?,
        Err(error) => {
            return Err(cstr!("Failed to start Corsa API session: {error}"));
        }
    };
    let capabilities = block_on(session.describe_capabilities())
        .unwrap_or_else(|_| Arc::new(CapabilitiesResponse::default()));
    Ok((session, capabilities))
}

async fn spawn_project_session_with_mode(
    executable: &str,
    cwd: &Path,
    config_path: &str,
    mode: ApiMode,
) -> Result<ProjectSession, TsgoError> {
    ProjectSession::spawn(
        ApiSpawnConfig::new(executable)
            .with_mode(mode)
            .with_cwd(cwd),
        config_path,
        None,
    )
    .await
}

fn should_retry_json_rpc(mode: ApiMode, error: &TsgoError) -> bool {
    if mode != ApiMode::SyncMsgpackStdio {
        return false;
    }

    let TsgoError::Protocol(message) = error else {
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
    fn trusts_capabilities(&self) -> bool {
        self.capabilities.runtime.capability_endpoint
    }

    pub(super) fn supports_overlay_api(&self) -> bool {
        !self.trusts_capabilities() || self.capabilities.overlay.update_snapshot_overlay_changes
    }

    pub(super) fn supports_project_diagnostics_api(&self) -> bool {
        !self.trusts_capabilities() || self.capabilities.diagnostics.project
    }

    pub(super) fn supports_file_diagnostics_api(&self) -> bool {
        !self.trusts_capabilities() || self.capabilities.diagnostics.file
    }

    pub(super) fn supports_hover_api(&self) -> bool {
        !self.trusts_capabilities() || self.capabilities.editor.hover
    }

    pub(super) fn supports_definition_api(&self) -> bool {
        !self.trusts_capabilities() || self.capabilities.editor.definition
    }

    pub(super) fn supports_references_api(&self) -> bool {
        !self.trusts_capabilities() || self.capabilities.editor.references
    }

    pub(super) fn supports_rename_api(&self) -> bool {
        !self.trusts_capabilities() || self.capabilities.editor.rename
    }

    pub(super) fn supports_completion_api(&self) -> bool {
        !self.trusts_capabilities() || self.capabilities.editor.completion
    }

    pub(super) fn can_use_api_for_uri(&self, uri: &str) -> bool {
        !self.document_texts.contains_key(uri) || self.supports_overlay_api()
    }

    pub(super) fn sync_overlay_document(&mut self, uri: &str, content: &str) -> Result<(), String> {
        self.document_texts.insert(uri.into(), content.into());
        if !self.supports_overlay_api() {
            return Err(
                "Corsa overlay changes are unavailable for this project-session runtime".into(),
            );
        }

        let document_uri = self.session_document_uri(uri);
        let file_changes = materialize_session_document(uri, document_uri.as_str(), content);
        if document_uri != uri {
            return block_on(self.session.refresh(file_changes))
                .map_err(|error| cstr!("Failed to refresh Corsa snapshot: {error}"));
        }

        let version = next_overlay_version(&mut self.overlay_versions, uri);
        block_on(self.session.refresh_with_overlay_changes(
            file_changes,
            Some(OverlayChanges {
                upsert: vec![OverlayUpdate {
                    document: uri_document_identifier(document_uri.as_str()),
                    text: content.into(),
                    version: Some(version),
                    language_id: Some("typescript".into()),
                }],
                delete: Vec::new(),
            }),
        ))
        .map_err(|error| cstr!("Failed to sync Corsa overlay: {error}"))
    }

    pub(super) fn delete_overlay_document(&mut self, uri: &str) -> Result<(), String> {
        self.document_texts.remove(uri);
        self.overlay_versions.remove(uri);
        let document_uri = self
            .session_document_uris
            .remove(uri)
            .unwrap_or_else(|| self.session_document_uri(uri));
        self.external_document_uris.remove(document_uri.as_str());
        let file_changes = remove_session_document(uri, document_uri.as_str());
        if document_uri != uri {
            return block_on(self.session.refresh(file_changes))
                .map_err(|error| cstr!("Failed to refresh Corsa snapshot: {error}"));
        }

        if !self.supports_overlay_api() {
            return Ok(());
        }

        block_on(self.session.refresh_with_overlay_changes(
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

        let mapped = build_session_document_uri(uri, &self.project_root);
        self.session_document_uris
            .insert(uri.into(), mapped.clone());
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

fn build_session_document_uri(uri: &str, project_root: &Path) -> String {
    let Some(external_path) = external_document_path(uri) else {
        return uri.into();
    };

    if external_path.starts_with(project_root) && external_path.exists() {
        return path_to_file_uri(&external_path);
    }

    let mut session_path = project_root.join("__agent_only").join("vize-corsa-overlay");
    for component in external_path.components() {
        match component {
            Component::Prefix(prefix) => session_path.push(prefix.as_os_str()),
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => session_path.push("__parent__"),
            Component::Normal(part) => session_path.push(part),
        }
    }

    path_to_file_uri(&session_path)
}

fn external_document_path(uri: &str) -> Option<PathBuf> {
    if let Some(path) = file_uri_to_path(uri) {
        return Some(path);
    }

    let (scheme, path) = uri.split_once("://")?;
    let mut session_path = PathBuf::from("__scheme__");
    session_path.push(scheme);
    session_path.push(path.trim_start_matches('/'));
    Some(session_path)
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
mod tests {
    use super::{
        api_mode_for_executable, line_character_to_utf16_offset, should_retry_json_rpc,
        uri_document_identifier,
    };
    use corsa::api::{ApiMode, DocumentIdentifier};
    use corsa::error::TsgoError;

    #[test]
    fn uses_async_json_rpc_for_node_modules_bin_wrappers() {
        assert_eq!(
            api_mode_for_executable("/tmp/project/node_modules/.bin/tsgo"),
            ApiMode::AsyncJsonRpcStdio
        );
    }

    #[test]
    fn uses_async_json_rpc_for_native_preview_js_entrypoints() {
        assert_eq!(
            api_mode_for_executable(
                "/tmp/project/node_modules/@typescript/native-preview/bin/tsgo.js"
            ),
            ApiMode::AsyncJsonRpcStdio
        );
    }

    #[test]
    fn keeps_native_binaries_on_sync_msgpack() {
        assert_eq!(
            api_mode_for_executable("/tmp/project/corsa-bind/.cache/tsgo"),
            ApiMode::SyncMsgpackStdio
        );
    }

    #[test]
    fn retries_json_rpc_after_msgpack_shape_mismatch() {
        let error = TsgoError::Protocol("expected tuple marker, got 61".into());

        assert!(should_retry_json_rpc(ApiMode::SyncMsgpackStdio, &error));
        assert!(!should_retry_json_rpc(ApiMode::AsyncJsonRpcStdio, &error));
    }

    #[test]
    fn utf16_offsets_clamp_to_line_boundaries() {
        assert_eq!(line_character_to_utf16_offset("alpha\nbeta", 0, 99), 5);
        assert_eq!(line_character_to_utf16_offset("a😀b", 0, 3), 3);
        assert_eq!(line_character_to_utf16_offset("a\nb", 9, 0), 3);
    }

    #[test]
    fn api_queries_use_uri_document_identifiers() {
        assert!(matches!(
            uri_document_identifier("file:///workspace/App.vue.ts"),
            DocumentIdentifier::FileName(_)
        ));
        assert!(matches!(
            uri_document_identifier("corsa://overlay/App.vue.ts"),
            DocumentIdentifier::Uri { .. }
        ));
    }
}
