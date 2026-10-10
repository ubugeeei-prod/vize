//! Workspace/LSP config loading and feature application.

#[cfg(feature = "glyph")]
mod formatting;
mod read;
#[cfg(feature = "glyph")]
use formatting::format_options_from_config;

use std::path::Path;
use std::sync::atomic::Ordering;

use vize_l0::config::{GlobalTypesConfig, LinterConfig, TypeCheckerConfig};

use super::ServerState;
use super::features::LspConfigSection;

impl ServerState {
    /// Apply LSP initialization options sent by an editor client.
    pub fn apply_lsp_initialization_options(&self, options: Option<&serde_json::Value>) {
        #[cfg(feature = "native")]
        self.retain_project_initialization_options(options);
        let Some(options) = options else {
            return;
        };

        match serde_json::from_value::<LspConfigSection>(options.clone()) {
            Ok(config) => self.apply_lsp_config(config, "initializationOptions"),
            Err(error) => {
                tracing::warn!(
                    "Failed to parse LSP initializationOptions: {}. Keeping current LSP options.",
                    error
                );
            }
        }
    }
    /// Get a clone of the current type checker config.
    #[inline]
    pub fn get_type_checker_config(&self) -> TypeCheckerConfig {
        self.type_checker_config.read().0.clone()
    }
    /// Effective editor Corsa request bound in milliseconds.
    #[inline]
    pub fn lsp_request_timeout_ms(&self) -> u64 {
        self.type_checker_config.read().1
    }

    /// Build the shared virtual TypeScript options from workspace config.
    #[cfg(feature = "native")]
    pub(crate) fn virtual_ts_options(&self) -> vize_canon::virtual_ts::VirtualTsOptions {
        let mut options = vize_canon::virtual_ts::VirtualTsOptions::default();
        options
            .template_globals
            .extend(self.global_types.read().iter().map(|(name, declaration)| {
                vize_canon::virtual_ts::TemplateGlobal {
                    name: name.clone(),
                    type_annotation: declaration.type_annotation.clone(),
                    default_value: declaration.template_default_value(),
                }
            }));
        options
    }

    /// Get a clone of the current linter config.
    #[inline]
    pub fn get_linter_config(&self) -> LinterConfig {
        self.linter_config.read().0.clone()
    }

    /// Get a clone of the current per-rule lint options.
    #[inline]
    pub fn get_linter_rule_options(&self) -> vize_l0::config::ConfigLintRuleOptions {
        self.linter_rule_options.read().clone()
    }

    /// Get the configured Vue dialect override, if any.
    #[inline]
    pub fn get_dialect_config(&self) -> Option<vize_l0::dialect::VueDialect> {
        *self.dialect_config.read()
    }

    /// Set the Vue dialect override (`None` re-enables structural detection).
    #[inline]
    pub fn set_dialect_config(&self, dialect: Option<vize_l0::dialect::VueDialect>) {
        self.update_names_configuration(|| self.set_dialect_config_locked(dialect));
    }

    fn set_dialect_config_locked(&self, dialect: Option<vize_l0::dialect::VueDialect>) {
        *self.dialect_config.write() = dialect;
    }

    fn update_names_configuration<T>(&self, apply: impl FnOnce() -> T) -> T {
        #[cfg(feature = "native")]
        let _change = self.corsa_environment_change();
        #[cfg(feature = "experimental-source-navigation")]
        {
            self.update_native_names_configuration(apply)
        }
        #[cfg(not(feature = "experimental-source-navigation"))]
        {
            apply()
        }
    }

    fn apply_type_checker_config(&self, config: TypeCheckerConfig, timeout_ms: u64, source: &Path) {
        #[cfg(feature = "native")]
        let _change = self.corsa_environment_change();
        #[cfg(feature = "experimental-source-navigation")]
        self.update_module_link_context(Some(source.to_path_buf()), || {
            self.install_type_checker_snapshot(config, timeout_ms, source);
        });
        #[cfg(not(feature = "experimental-source-navigation"))]
        {
            self.install_type_checker_snapshot(config, timeout_ms, source);
        }
        self.invalidate_component_interfaces();
        // The tsconfig and runtime this selects decide which project the
        // overlays are layered onto, so a reload retargets them even though no
        // document changed (#3442).
        #[cfg(feature = "native")]
        {
            self.invalidate_corsa_overlays();
            self.retire_corsa_configuration();
        }
        tracing::info!("Loaded type checker config from {}", source.display());
    }

    fn apply_global_types_config(&self, config: GlobalTypesConfig, source: &str) {
        #[cfg(feature = "native")]
        let _change = self.corsa_environment_change();
        *self.global_types.write() = config;
        #[cfg(feature = "native")]
        self.batch_cache.invalidate();
        tracing::info!("Loaded global types config from {}", source);
    }

    fn apply_config_features(&self, features: vize_l0::config::ConfigFeatureFlags) {
        #[cfg(feature = "native")]
        let _change = self.corsa_environment_change();
        self.apply_linter_features(features);
        self.experimental_patterned_template
            .store(features.experimental_patterned_template, Ordering::SeqCst);
        *self.type_checker_options_api.write() = features.type_checker_options_api;
        // `type_checker_legacy_vue2` already folds in a Vue 2 / 2.7 dialect
        // (`ConfigFeatureFlags::from`), so the LSP and `vize check` agree on
        // slot-scope and filter lowering for `vue.version: "2.7"` alone (#3297).
        *self.type_checker_legacy_vue2.write() = features.type_checker_legacy_vue2;
        *self.type_checker_vue_version.write() = features.vue_version.unwrap_or_default();
        *self.type_checker_jsx_typecheck.write() = features.type_checker_jsx_typecheck;

        let mut lsp_features = self.lsp_features.write();
        lsp_features.options_api = features.type_checker_options_api;
        if let Some(enabled) = features.language_server_legacy_vue2 {
            lsp_features.legacy_vue2 = enabled;
        }
        lsp_features.apply_effective_compatibility();
    }

    pub(crate) fn patterned_template_enabled(&self) -> bool {
        self.experimental_patterned_template.load(Ordering::SeqCst)
    }

    /// Effective JSX policy; dedicated configurations retain their opt-in default.
    pub(crate) fn jsx_typecheck_enabled(&self) -> bool {
        *self.type_checker_jsx_typecheck.read()
    }

    /// Build the config-file LSP section, folding in the `languageServer`
    /// switches that are not stable carton model fields. They ride on
    /// [`LspConfigSection`] rather than being applied afterwards so the
    /// `editor` bundle switch still loses to an explicit per-feature toggle.
    fn lsp_config_section_from_file(
        config: vize_l0::config::LanguageServerConfig,
        unstable: vize_l0::config::LanguageServerUnstableFlags,
    ) -> LspConfigSection {
        LspConfigSection::from(config).with_signature_help(unstable.signature_help)
    }

    fn apply_linter_config(&self, config: LinterConfig, source: &str) {
        self.linter_config.write().0 = config;
        tracing::info!("Loaded linter config from {}", source);
    }

    fn apply_lsp_config(&self, config: LspConfigSection, source: &str) {
        let features = self.update_names_configuration(|| self.apply_lsp_config_locked(config));
        tracing::info!("Loaded LSP config from {}: {:?}", source, features);
    }

    fn apply_lsp_config_locked(&self, config: LspConfigSection) -> super::LspFeatureConfig {
        #[cfg(feature = "experimental-source-navigation")]
        if let Some(enabled) = config.native_linked_editing {
            self.native_linked_editing.store(enabled, Ordering::SeqCst);
        }
        let mut features = self.lsp_features.write();
        config.apply_to(&mut features);
        features.apply_effective_compatibility();
        self.lsp_typecheck_enabled
            .store(features.typecheck, Ordering::SeqCst);
        *features
    }

    fn apply_names_configuration(
        &self,
        features: vize_l0::config::ConfigFeatureFlags,
        lsp: LspConfigSection,
        dialect: Option<vize_l0::dialect::VueDialect>,
        source: &str,
    ) {
        let features = self.update_names_configuration(|| {
            self.apply_config_features(features);
            let lsp = self.apply_lsp_config_locked(lsp);
            self.set_dialect_config_locked(dialect);
            lsp
        });
        tracing::info!("Loaded LSP config from {}: {:?}", source, features);
    }

    /// Load all workspace-scoped options from `vize.config.pkl` (preferred) or JSON.
    pub fn load_workspace_config(&self, dir: &Path) {
        let loaded = vize_carton::config::load_lsp_config_snapshot(Some(dir));
        if !loaded.valid {
            return;
        }
        self.install_project_linter_context(dir, &loaded.project);
        #[cfg(feature = "native")]
        self.apply_project_path_identity(dir, &loaded.project);
        self.apply_project_formatting_default(loaded.source_path.as_deref());
        if loaded.source_path.is_none() {
            self.apply_config_features(loaded.features);
        }
        if let Some(source_path) = loaded.source_path {
            let source = source_path.display().to_string();
            let config = loaded.config;
            #[cfg(feature = "glyph")]
            {
                *self.format_options.write() = (
                    format_options_from_config(&config.formatter),
                    source_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("vite.config."))
                        && loaded.compiler_whitespace == Some("preserve"),
                );
                tracing::info!("Loaded format config from {}", source);
            }
            self.apply_linter_config(loaded.linter, &source);
            *self.linter_rule_options.write() = loaded.lint_rule_options;
            self.apply_global_types_config(config.global_types, &source);
            self.apply_type_checker_config(
                config.type_checker,
                loaded.request_timeout_ms,
                &source_path,
            );
            self.apply_names_configuration(
                loaded.features,
                Self::lsp_config_section_from_file(
                    config.language_server,
                    loaded.language_server_unstable_flags,
                ),
                config.dialect,
                &source,
            );
        }
    }

    /// Load LSP options from `vize.config.pkl` (preferred) or `vize.config.json`.
    pub fn load_lsp_config(&self, dir: &Path) {
        let loaded = vize_carton::config::load_lsp_config_snapshot(Some(dir));
        if !loaded.valid {
            return;
        }
        self.apply_project_formatting_default(loaded.source_path.as_deref());
        if loaded.source_path.is_none() {
            self.apply_config_features(loaded.features);
        }
        if let Some(source_path) = loaded.source_path {
            let source = source_path.display().to_string();
            let config = loaded.config;
            self.apply_linter_config(loaded.linter, &source);
            *self.linter_rule_options.write() = loaded.lint_rule_options;
            self.apply_global_types_config(config.global_types, &source);
            self.apply_type_checker_config(
                config.type_checker,
                loaded.request_timeout_ms,
                &source_path,
            );
            self.apply_names_configuration(
                loaded.features,
                Self::lsp_config_section_from_file(
                    config.language_server,
                    loaded.language_server_unstable_flags,
                ),
                config.dialect,
                &source,
            );
        }
    }

    // Keep dedicated-config users' opt-in formatting behavior. Fresh projects
    // expose the formatter alongside the existing recommended editor profile;
    // authored project and editor switches are applied afterwards.
    fn apply_project_formatting_default(&self, source: Option<&Path>) {
        let dedicated = source
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("vize.config."));
        if !dedicated {
            self.apply_lsp_config(
                LspConfigSection::from(vize_l0::config::LanguageServerConfig {
                    formatting: Some(true),
                    ..Default::default()
                }),
                "project defaults",
            );
        }
    }

    /// Get a clone of the current format options.
    #[cfg(feature = "glyph")]
    #[inline]
    pub fn get_format_options(&self) -> vize_glyph::FormatOptions {
        self.format_options.read().0.clone()
    }

    /// Native options and Vite template-whitespace policy from one snapshot.
    #[cfg(feature = "glyph")]
    pub(crate) fn get_formatter_context(&self) -> (vize_glyph::FormatOptions, bool) {
        self.format_options.read().clone()
    }

    /// Load format options from `vize.config.json` in the given directory.
    #[cfg(feature = "glyph")]
    pub fn load_format_config(&self, dir: &Path) {
        let loaded = vize_carton::config::load_config_with_source(Some(dir));
        if let Some(source_path) = loaded.source_path {
            *self.format_options.write() =
                (format_options_from_config(&loaded.config.formatter), false);
            tracing::info!("Loaded format config from {}", source_path.display());
        }
    }
}
