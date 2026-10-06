//! Actual configuration read helpers, with unchanged public visibility.
use super::super::{LspFeatureConfig, ServerState};

impl ServerState {
    /// Get the enabled LSP feature set.
    #[inline]
    pub(crate) fn lsp_features(&self) -> LspFeatureConfig {
        *self.lsp_features.read()
    }

    #[inline]
    pub(crate) fn legacy_vue2_enabled(&self) -> bool {
        *self.type_checker_legacy_vue2.read() || self.lsp_features().legacy_vue2
    }
    /// Resolve Vue 3 Options API template bindings. Implied by legacy mode.
    #[inline]
    pub(crate) fn options_api_enabled(&self) -> bool {
        *self.type_checker_options_api.read()
            || self.lsp_features().options_api
            || self.legacy_vue2_enabled()
    }
}
