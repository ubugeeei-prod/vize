//! Actual configuration read helpers, with unchanged public visibility.
use super::{
    super::{LspFeatureConfig, ServerState},
    host::ProjectModel,
};

#[cfg(all(test, feature = "native"))]
mod tests;

impl ServerState {
    /// Effective project Vue version used by type checking and formatting.
    #[cfg(any(
        feature = "native",
        feature = "glyph",
        feature = "experimental-source-navigation"
    ))]
    pub(crate) fn type_checker_vue_version(&self) -> vize_l0::config::VueVersion {
        *self.type_checker_vue_version.read()
    }

    pub(super) fn install_type_checker_snapshot(
        &self,
        root: &std::path::Path,
        config: vize_l0::config::TypeCheckerConfig,
        timeout_ms: u64,
        source: Option<&std::path::Path>,
    ) {
        let project = ProjectModel::new(Some(root), source, &config);
        let mut settings = self.type_checker_config.write();
        *settings = (config, timeout_ms, project);
    }

    #[cfg(feature = "native")]
    pub(in crate::server::state) fn retarget_type_checker_project(&self, root: &std::path::Path) {
        let mut settings = self.type_checker_config.write();
        let source = settings.2.config_source().map(std::path::Path::to_path_buf);
        let config = settings.0.clone();
        settings.2 = ProjectModel::new(Some(root), source.as_deref(), &config);
    }

    #[cfg(feature = "native")]
    pub(in crate::server::state) fn native_checker_settings(
        &self,
    ) -> (vize_l0::config::TypeCheckerConfig, u64, ProjectModel) {
        let settings = self.type_checker_config.read();
        (settings.0.clone(), settings.1, settings.2.clone())
    }

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
