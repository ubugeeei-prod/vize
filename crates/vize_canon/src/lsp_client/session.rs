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
        DocumentIdentifier, FileChangeSummary, FileChanges, OverlayChanges, OverlayUpdate,
        ProjectSession,
    },
    fast::CompactString,
    runtime::block_on,
};
use lsp_types::Diagnostic;
use serde_json::Value;
use vize_s0::{String, cstr};

mod bootstrap;
mod canon_document;
mod capabilities;
pub(super) use bootstrap::{ProjectSessionSpawnError, spawn_project_session};
#[cfg(test)]
use bootstrap::{api_mode_for_executable, classify_project_session_error, should_retry_json_rpc};

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

        // A Canon query document is already an exact file inside the active
        // materialized project. Keep that disk identity authoritative even
        // when the runtime advertises overlays: applying a second overlay
        // snapshot after the materialized-file refresh can detach the file
        // from its configured project on the next edit. `document_texts`
        // remains populated above for editor-LSP fallback queries.
        if self.sync_existing_canon_document(uri, content)? {
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
                if self.sync_existing_canon_document(uri, content)? {
                    return Ok(());
                }
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

fn next_overlay_version(versions: &mut vize_s0::FxHashMap<String, i32>, uri: &str) -> i32 {
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
