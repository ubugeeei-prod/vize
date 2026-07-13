//! Persistent script projections for open SFC revisions.

use tower_lsp::lsp_types::Url;
use vize_atlas::Shared;

use super::ServerState;

impl ServerState {
    pub(crate) fn sfc_script_syntax(
        &self,
        uri: &Url,
    ) -> Option<Shared<vize_atelier_sfc::SfcScriptSyntaxSnapshot>> {
        self.artifact::<vize_atelier_sfc::SfcScriptSyntaxProduct>(uri)
    }

    /// Return the parse-once neutral script modules for an open SFC.
    pub(crate) fn sfc_modules(&self, uri: &Url) -> Option<Shared<vize_module::ModuleDocument>> {
        self.artifact::<vize_module::ModuleSyntaxProduct>(uri)
    }
}
