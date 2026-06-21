use std::path::Path;

use super::{LoadedConfigWithFeatures, load_linter_from_raw_config, load_raw_config_with_source};
use crate::config::{LinterConfig, LinterFeatureFlags};

/// Load configuration, feature flags, linter settings, and lint-only compatibility in one pass.
pub fn load_config_and_linter_with_lint_features_and_source(
    path: Option<&Path>,
) -> (LoadedConfigWithFeatures, LinterConfig, LinterFeatureFlags) {
    let loaded = load_raw_config_with_source(path);
    let compiler_compatibility_vue_version = loaded.config.compiler.compatibility.vue_version;
    let compiler_vapor = loaded.config.compiler.vapor;
    let linter = load_linter_from_raw_config(&loaded.config);
    let (config, features) = loaded.config.into_config_and_features();
    let linter_features = LinterFeatureFlags::from_config_features(
        features,
        compiler_compatibility_vue_version,
        compiler_vapor,
    );

    (
        LoadedConfigWithFeatures {
            config,
            source_path: loaded.source_path,
            features,
        },
        linter,
        linter_features,
    )
}
