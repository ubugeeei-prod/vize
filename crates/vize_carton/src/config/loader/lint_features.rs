use std::path::Path;

use super::{
    LoadedConfigWithFeatures, RawVizeConfig, load_linter_from_raw_config,
    load_raw_config_with_source,
};
use crate::config::{
    ConfigEntryIgnore, LinterConfig, LinterConfigEntry, LinterConfigPlan,
    LinterConfigPlanWithRuleOptions, LinterFeatureFlags,
};

fn load_linter_plan_from_raw_config(config: &RawVizeConfig) -> LinterConfigPlan {
    let entries = config
        .entries
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|entry| {
            let linter = LinterConfig::from(entry.linter.clone());
            (!linter.rules.is_empty()).then(|| LinterConfigEntry {
                base_path: entry.base_path.clone(),
                files: entry.files.clone(),
                ignores: entry.ignores.clone().unwrap_or_default(),
                rules: linter.rules,
            })
        })
        .collect();
    LinterConfigPlan {
        base: load_linter_from_raw_config(config),
        entries,
        global_ignores: global_ignores_from_raw_config(config),
        rule_options: config.linter.rule_options().clone(),
    }
}

fn load_linter_plan_with_rule_options_from_raw_config(
    config: &RawVizeConfig,
) -> LinterConfigPlanWithRuleOptions {
    let mut entries = Vec::new();
    let mut entry_rule_options = Vec::new();
    for entry in config.entries.as_deref().unwrap_or_default() {
        let rule_options = entry.linter.rule_options().clone();
        let linter = LinterConfig::from(entry.linter.clone());
        if linter.rules.is_empty() && rule_options.is_empty() {
            continue;
        }
        entries.push(LinterConfigEntry {
            base_path: entry.base_path.clone(),
            files: entry.files.clone(),
            ignores: entry.ignores.clone().unwrap_or_default(),
            rules: linter.rules,
        });
        entry_rule_options.push(rule_options);
    }
    LinterConfigPlanWithRuleOptions {
        plan: LinterConfigPlan {
            base: load_linter_from_raw_config(config),
            entries,
            global_ignores: global_ignores_from_raw_config(config),
            rule_options: config.linter.rule_options().clone(),
        },
        entry_rule_options,
    }
}

fn global_ignores_from_raw_config(config: &RawVizeConfig) -> Vec<ConfigEntryIgnore> {
    config
        .ignores
        .as_deref()
        .unwrap_or_default()
        .iter()
        .cloned()
        .map(|pattern| ConfigEntryIgnore {
            // Preserve the established top-level ignore contract: patterns are
            // resolved from the config directory, independently of `basePath`.
            base_path: None,
            pattern,
        })
        .collect()
}

/// Load configuration, feature flags, linter settings, and lint-only compatibility in one pass.
pub fn load_config_and_linter_with_lint_features_and_source(
    path: Option<&Path>,
) -> (LoadedConfigWithFeatures, LinterConfig, LinterFeatureFlags) {
    let loaded = load_raw_config_with_source(path);
    let compiler_compatibility_vue_version = loaded.config.compiler.compatibility.vue_version;
    let compiler_vapor = loaded
        .config
        .compiler
        .vapor
        .or_else(|| loaded.config.experimentals.vapor_enabled().then_some(true));
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

/// Load the declaration-ordered linter plan without reparsing the config.
pub fn load_config_and_linter_plan_with_lint_features_and_source(
    path: Option<&Path>,
) -> (
    LoadedConfigWithFeatures,
    LinterConfigPlan,
    LinterFeatureFlags,
) {
    let loaded = load_raw_config_with_source(path);
    let compiler_compatibility_vue_version = loaded.config.compiler.compatibility.vue_version;
    let compiler_vapor = loaded
        .config
        .compiler
        .vapor
        .or_else(|| loaded.config.experimentals.vapor_enabled().then_some(true));
    let linter = load_linter_plan_from_raw_config(&loaded.config);
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
    let compiler_compatibility_vue_version = loaded.config.compiler.compatibility.vue_version;
    let compiler_vapor = loaded
        .config
        .compiler
        .vapor
        .or_else(|| loaded.config.experimentals.vapor_enabled().then_some(true));
    let linter = load_linter_plan_with_rule_options_from_raw_config(&loaded.config);
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
