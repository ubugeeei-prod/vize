//! Lint-only feature switches derived from config compatibility keys.

use super::{ConfigFeatureFlags, VueVersion};

/// Lint-only feature switches derived from config compatibility keys.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LinterFeatureFlags {
    pub vue_version: Option<VueVersion>,
    pub vapor: Option<bool>,
}

impl LinterFeatureFlags {
    pub(crate) fn from_config_features(
        features: ConfigFeatureFlags,
        compiler_compatibility_vue_version: Option<VueVersion>,
        compiler_vapor: Option<bool>,
    ) -> Self {
        let vue_version = features
            .vue_version
            .or(compiler_compatibility_vue_version)
            .or_else(|| {
                (features.type_checker_legacy_vue2
                    || features.language_server_legacy_vue2 == Some(true))
                .then_some(VueVersion::V2_7)
            });
        Self {
            vue_version,
            vapor: compiler_vapor,
        }
    }
}
