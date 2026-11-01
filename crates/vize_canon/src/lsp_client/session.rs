#![expect(clippy::disallowed_types, reason = "dependency API uses std String")]

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
use vize_l0::{String, cstr};

mod bootstrap;
mod canon_document;
mod capabilities;
mod close;
mod logical;
mod replay;
pub(super) use bootstrap::{
    ProjectSessionSpawnError, api_mode_for_executable, spawn_project_session,
    spawn_project_session_for_document, validate_materialized_project_selection,
};
#[cfg(test)]
use bootstrap::{classify_project_session_error, should_retry_json_rpc};

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

    pub(super) fn project_session(&mut self) -> Result<&ProjectSession, String> {
        self.ensure_project_transport()?;
        self.require_current_logical_api_revision()?;
        self.session
            .as_ref()
            .ok_or_else(|| cstr!("Corsa project-session API is unavailable"))
    }

    pub(super) fn project_session_mut(&mut self) -> Result<&mut ProjectSession, String> {
        self.ensure_project_transport()?;
        self.require_current_logical_api_revision()?;
        self.session
            .as_mut()
            .ok_or_else(|| cstr!("Corsa project-session API is unavailable"))
    }

    pub(super) fn sync_overlay_document(&mut self, uri: &str, content: &str) -> Result<(), String> {
        let previous = self.retain_document_text(uri, content);
        if previous.changed() {
            self.editor_lsp_documents_dirty = true;
        }

        if self.logical_workspace_overlays {
            return self.retain_logical_workspace_revision();
        }
        if self.materialized_project_session {
            return self.sync_materialized_overlay_document(uri, content);
        }

        if !self.has_project_session() {
            self.try_logical_workspace_overlays()?;
            return Ok(());
        }

        let document_uri = self.session_document_uri(uri);
        if !previous.changed() {
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

        let file_changes = materialize_session_document(
            uri,
            document_uri.as_str(),
            content,
            self.overlay_storage.root(),
        )?
        .or_else(|| {
            virtual_overlay::upsert_file_changes(
                uri,
                document_uri.as_str(),
                &self.project_root,
                previous.was_present(),
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
        if self.try_logical_workspace_overlays()? {
            return Ok(());
        }
        self.activate_materialized_project_session()?;
        let document_uri = self
            .materialized_session_document_uri(uri)
            .ok_or_else(|| cstr!("Failed to derive materialized Corsa overlay path for {uri}"))?;
        let file_changes = materialize_session_document(
            uri,
            document_uri.as_str(),
            content,
            self.overlay_storage.root(),
        )?;
        self.refresh_materialized_overlay_snapshot(file_changes)
    }

    pub(super) fn delete_overlay_document(&mut self, uri: &str) -> Result<(), String> {
        if self.logical_workspace_overlays {
            return self.close_logical_workspace_document(uri);
        }
        let document_uri = self
            .session_document_uris
            .get(uri)
            .cloned()
            .unwrap_or_else(|| self.session_document_uri(uri));
        let file_changes = if document_uri != uri {
            close::close_materialized_document(
                uri,
                document_uri.as_str(),
                &self.project_root,
                self.overlay_storage.root(),
            )?
        } else {
            virtual_overlay::delete_file_changes(uri, document_uri.as_str(), &self.project_root)
        };
        let restored = matches!(&file_changes, Some(FileChanges::Summary(summary))
            if !summary.changed.is_empty() || !summary.created.is_empty());
        // A closed disk-backed member still belongs to the materialized
        // project. Preserve its external identity for navigation results.
        if !restored {
            self.session_document_uris.remove(uri);
            self.external_document_uris.remove(document_uri.as_str());
        }
        if self.document_texts.remove(uri).is_some() {
            self.editor_lsp_documents_dirty = true;
        }
        self.overlay_versions.remove(uri);
        if document_uri != uri {
            if let Some(editor) = self.editor_lsp.as_mut()
                && let Some(FileChanges::Summary(summary)) = &file_changes
                && let Some(path) = file_uri_to_path(document_uri.as_str())
            {
                if !summary.created.is_empty() {
                    editor.refresh_materialized_files(&[], &[path], &[])?;
                } else if restored {
                    editor.refresh_materialized_files(&[path], &[], &[])?;
                } else {
                    editor.refresh_materialized_files(&[], &[], &[path])?;
                }
            }
            return self.refresh_materialized_overlay_snapshot(file_changes);
        }
        if !self.has_project_session() {
            return Ok(());
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
        if self.logical_workspace_overlays {
            return self.remember_session_document_uri(uri, uri.into());
        }
        if let Some(mapped) = self.session_document_uris.get(uri) {
            return mapped.clone();
        }

        let mapped = build_session_document_uri(
            uri,
            &self.project_root,
            self.overlay_storage.root(),
            self.supports_overlay_api(),
        );
        self.remember_session_document_uri(uri, mapped)
    }

    fn materialized_session_document_uri(&mut self, uri: &str) -> Option<String> {
        let mapped = build_materialized_session_document_uri(
            uri,
            &self.project_root,
            self.overlay_storage.root(),
        )?;
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

    pub(super) fn remap_reference_result_uris(&self, value: &mut Value) {
        // The native editor can return the same owned materialized identities
        // as the API. Only Location.uri is protocol identity here; retain all
        // coordinates, unknown entries and opaque payloads without rewriting.
        for location in value.as_array_mut().into_iter().flatten() {
            if let Some(Value::String(uri)) = location.get_mut("uri")
                && let Some(mapped) = self.external_document_uris.get(uri.as_str())
            {
                *uri = mapped.as_str().into();
            }
        }
    }
}

fn next_overlay_version(versions: &mut vize_l0::FxHashMap<String, i32>, uri: &str) -> i32 {
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
    overlay_root: &std::path::Path,
) -> Result<Option<FileChanges>, String> {
    if document_uri == external_uri {
        return Ok(None);
    }
    let path = file_uri_to_path(document_uri)
        .ok_or_else(|| cstr!("Invalid private Corsa document URI {document_uri}"))?;
    let existed =
        super::session_storage::write_owned_file(overlay_root, &path, content.as_bytes())?;
    Ok(Some(FileChanges::Summary(FileChangeSummary {
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
    })))
}

pub(super) fn line_character_to_utf16_offset(text: &str, line: u32, character: u32) -> u32 {
    let start = vize_l0::line_index::LineBreaks::Lsp
        .line_starts(text)
        .nth(line as usize)
        .unwrap_or(text.len());
    let (before, after) = text.split_at_checked(start).unwrap_or((text, ""));
    let offset = before.encode_utf16().count() as u32;
    let line_len = after
        .split(['\r', '\n'])
        .next()
        .unwrap_or_default()
        .encode_utf16()
        .count() as u32;
    offset + character.min(line_len)
}

#[cfg(test)]
mod tests;
