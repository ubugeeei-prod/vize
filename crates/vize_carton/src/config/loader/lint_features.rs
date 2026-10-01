use std::path::Path;

use super::{LoadedConfigWithFeatures, load_raw_config_with_source};
use crate::config::{
    LinterConfig, LinterConfigPlan, LinterConfigPlanWithConfigRuleOptions,
    LinterConfigPlanWithRuleOptions, LinterFeatureFlags,
};

/// Load configuration, feature flags, linter settings, and lint-only compatibility in one pass.
pub fn load_config_and_linter_with_lint_features_and_source(
    path: Option<&Path>,
) -> (LoadedConfigWithFeatures, LinterConfig, LinterFeatureFlags) {
    let loaded = load_raw_config_with_source(path);
    let compiler_compatibility_vue_version = loaded
        .config
        .compiler_compatibility_vue_version()
        .or(loaded.config.legacy_compatibility_vue_version());
    let compiler_vapor = loaded.config.compiler_vapor();
    let linter = loaded.config.linter();
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

/// Load the declaration-ordered linter plan without reparsing the config.
pub fn load_config_and_linter_plan_with_lint_features_and_source(
    path: Option<&Path>,
) -> (
    LoadedConfigWithFeatures,
    LinterConfigPlan,
    LinterFeatureFlags,
) {
    let loaded = load_raw_config_with_source(path);
    let compiler_compatibility_vue_version = loaded
        .config
        .compiler_compatibility_vue_version()
        .or(loaded.config.legacy_compatibility_vue_version());
    let compiler_vapor = loaded.config.compiler_vapor();
    let linter = loaded.config.linter_plan();
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

/// Load the declaration-ordered linter plan with entry-local rule options.
pub fn load_config_and_linter_plan_with_rule_options_and_lint_features_and_source(
    path: Option<&Path>,
) -> (
    LoadedConfigWithFeatures,
    LinterConfigPlanWithRuleOptions,
    LinterFeatureFlags,
) {
    let loaded = load_raw_config_with_source(path);
    let compiler_compatibility_vue_version = loaded
        .config
        .compiler_compatibility_vue_version()
        .or(loaded.config.legacy_compatibility_vue_version());
    let compiler_vapor = loaded.config.compiler_vapor();
    let linter = loaded.config.linter_plan_with_rule_options();
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

/// Load the declaration-ordered linter plan with full entry-local rule options.
pub fn load_config_and_linter_plan_with_config_rule_options_and_lint_features_and_source(
    path: Option<&Path>,
) -> (
    LoadedConfigWithFeatures,
    LinterConfigPlanWithConfigRuleOptions,
    LinterFeatureFlags,
) {
    let loaded = load_raw_config_with_source(path);
    let compiler_compatibility_vue_version = loaded.config.compiler_compatibility_vue_version();
    let compiler_vapor = loaded.config.compiler_vapor();
    let linter = loaded.config.linter_plan_with_config_rule_options();
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
