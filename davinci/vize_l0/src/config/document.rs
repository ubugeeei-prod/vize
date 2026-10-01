//! An in-memory configuration document, independent of host discovery or evaluation.

mod entries;
mod linter;
#[cfg(test)]
mod tests;

use serde::Deserialize;

use super::model::{
    ConfigExperimentalVueFlags, ConfigFeatureFlags, ConfigLintRuleOptions,
    LanguageServerUnstableFlags, LibConfig, RawVizeConfig, VizeConfig, VueVersion,
};

/// One deserialized configuration value and its pure projections.
///
/// The raw representation stays private. Hosts supply JSON bytes after reading
/// or evaluating their input; projections preserve the existing alias and
/// declaration-order rules without another parse or serialization roundtrip.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct ConfigDocument(RawVizeConfig);

impl ConfigDocument {
    /// Normalize the effective model and its feature flags once.
    pub fn into_config_and_features(self) -> (VizeConfig, ConfigFeatureFlags) {
        self.0.into_config_and_features()
    }

    /// Stable compiler template syntax when explicitly supplied.
    pub fn compiler_template_syntax(&self) -> Option<&'static str> {
        self.0
            .compiler
            .template_syntax
            .map(|template_syntax| template_syntax.as_str())
    }

    /// Stable compiler compatibility dialect, before legacy fallback.
    pub fn compiler_compatibility_vue_version(&self) -> Option<VueVersion> {
        self.0.compiler.compatibility.vue_version
    }

    /// Legacy top-level compatibility dialect.
    pub fn legacy_compatibility_vue_version(&self) -> Option<VueVersion> {
        self.0.compatibility.vue_version
    }

    /// Stable host compiler selection when explicitly supplied.
    pub fn compiler_host_compiler(&self) -> Option<bool> {
        self.0.compiler.compatibility.host_compiler
    }

    /// Stable Vapor selection takes precedence over the experimental alias.
    pub fn compiler_vapor(&self) -> Option<bool> {
        self.0
            .compiler
            .vapor
            .or_else(|| self.0.experimentals.vapor_enabled().then_some(true))
    }

    /// Consume the declared custom-element patterns without cloning them.
    pub fn into_compiler_custom_elements(self) -> Vec<crate::String> {
        self.0.compiler.custom_elements
    }

    /// Experimental Vue switches not represented by stable model fields.
    pub fn experimental_vue_flags(&self) -> ConfigExperimentalVueFlags {
        self.0.experimental_vue_flags()
    }

    /// Editor timeout after the existing default and minimum are applied.
    pub fn lsp_request_timeout_ms(&self) -> u64 {
        self.0.lsp_request_timeout_ms()
    }

    /// Editor switches after the existing stable/legacy precedence is applied.
    pub fn language_server_unstable_flags(&self) -> LanguageServerUnstableFlags {
        self.0.language_server_unstable_flags()
    }

    /// Borrow typed rule options from this same deserialization.
    pub fn lint_rule_options(&self) -> &ConfigLintRuleOptions {
        self.0.linter.rule_options()
    }

    /// Consume the library section without cloning it.
    pub fn into_lib_config(self) -> LibConfig {
        self.0.lib
    }
}
