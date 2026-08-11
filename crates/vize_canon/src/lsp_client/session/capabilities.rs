//! Capability policy for project-session and materialized Canon documents.

use crate::file_uri::file_uri_to_path;

use super::CorsaProjectClient;

impl CorsaProjectClient {
    fn trusts_capabilities(&self) -> bool {
        self.capabilities.runtime.capability_endpoint
    }

    pub(in crate::lsp_client) fn supports_overlay_api(&self) -> bool {
        if !self.has_project_session() {
            return true;
        }
        !self.overlay_api_disabled
            && (!self.trusts_capabilities()
                || self.capabilities.overlay.update_snapshot_overlay_changes)
    }

    pub(in crate::lsp_client) fn supports_project_diagnostics_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.diagnostics.project)
    }

    pub(in crate::lsp_client) fn supports_file_diagnostics_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.diagnostics.file)
    }

    pub(in crate::lsp_client) fn supports_hover_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.editor.hover)
    }

    pub(in crate::lsp_client) fn supports_definition_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.editor.definition)
    }

    pub(in crate::lsp_client) fn supports_references_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.editor.references)
    }

    pub(in crate::lsp_client) fn supports_rename_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.editor.rename)
    }

    pub(in crate::lsp_client) fn supports_completion_api(&self) -> bool {
        self.has_project_session()
            && (!self.trusts_capabilities() || self.capabilities.editor.completion)
    }

    pub(in crate::lsp_client) fn can_use_api_for_uri(&self, uri: &str) -> bool {
        !self.document_texts.contains_key(uri)
            || self.supports_overlay_api()
            || (self.project_root.join("__vize_vue_modules.d.ts").is_file()
                && file_uri_to_path(uri)
                    .is_some_and(|path| path.starts_with(&self.project_root) && path.is_file()))
    }
}
