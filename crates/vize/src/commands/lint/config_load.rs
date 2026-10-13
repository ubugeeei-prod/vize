//! Load shared lint configuration once for the CLI.

use super::LintArgs;
use crate::config::{
    LinterConfigPlanWithConfigRuleOptions, LinterFeatureFlags, LoadedConfigWithFeatures,
};
use std::path::PathBuf;
use vize_relief::options::CustomElementMatcher;

mod rule_names;

#[derive(Default)]
pub(super) struct LoadedLintConfig {
    pub(super) config: LoadedConfigWithFeatures,
    pub(super) plan: LinterConfigPlanWithConfigRuleOptions,
    pub(super) features: LinterFeatureFlags,
    pub(super) project_root: Option<PathBuf>,
    pub(super) custom_elements: CustomElementMatcher,
}

pub(super) fn load(args: &mut LintArgs) -> LoadedLintConfig {
    if args.no_config {
        return LoadedLintConfig::default();
    }
    let loaded = crate::config::try_load_project_config_with_source(args.config.as_deref())
        .unwrap_or_else(|error| {
            eprintln!("\x1b[31mError:\x1b[0m {error}");
            std::process::exit(2);
        });
    let document = loaded.document;
    let compatibility = document.compiler_compatibility_vue_version();
    let vapor = document.compiler_vapor();
    let plan = document.linter_plan_with_config_rule_options();
    let execution = document.linter_execution();
    let custom_elements =
        CustomElementMatcher::from_patterns(document.clone().into_compiler_custom_elements());
    let (config, flags) = document.into_config_and_features();
    let features = LinterFeatureFlags::from_config_features(flags, compatibility, vapor);
    rule_names::validate(&plan.plan).unwrap_or_else(|error| {
        eprintln!("\x1b[31mError:\x1b[0m {error}");
        std::process::exit(2);
    });
    args.cross_file |= execution.cross_file;
    args.cross_file_tree |= execution.cross_file_tree;
    args.cross_file_complexity |= execution.cross_file_complexity;
    args.max_warnings = args.max_warnings.or(execution.max_warnings);
    LoadedLintConfig {
        config: LoadedConfigWithFeatures {
            config,
            source_path: loaded.source_path,
            features: flags,
        },
        plan,
        features,
        project_root: loaded.project_root,
        custom_elements,
    }
}
