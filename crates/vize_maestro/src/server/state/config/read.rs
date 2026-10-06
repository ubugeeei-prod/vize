//! Actual configuration read helpers, with unchanged public visibility.
use super::super::{LspFeatureConfig, ServerState};

impl ServerState {
    pub(super) fn apply_linter_features(&self, features: vize_l0::config::ConfigFeatureFlags) {
        self.linter_config.write().1 =
            vize_l0::config::LinterFeatureFlags::from_config_features(features, None, None);
    }

    pub(crate) fn get_linter_config_and_features(
        &self,
    ) -> (
        vize_l0::config::LinterConfig,
        vize_l0::config::LinterFeatureFlags,
    ) {
        let snapshot = self.linter_config.read();
        (snapshot.0.clone(), snapshot.1)
    }

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
