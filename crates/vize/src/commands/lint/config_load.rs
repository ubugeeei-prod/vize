//! Load shared lint configuration once for the CLI.

use super::LintArgs;
use crate::config::{
    LinterConfigPlanWithConfigRuleOptions, LinterFeatureFlags, LoadedConfigWithFeatures,
};

pub(super) fn load(
    args: &LintArgs,
) -> (
    LoadedConfigWithFeatures,
    LinterConfigPlanWithConfigRuleOptions,
    LinterFeatureFlags,
) {
    if args.no_config {
        (
            crate::config::LoadedConfigWithFeatures::default(),
            crate::config::LinterConfigPlanWithConfigRuleOptions::default(),
            crate::config::LinterFeatureFlags::default(),
        )
    } else {
        crate::config::load_config_and_linter_plan_with_config_rule_options_and_lint_features_and_source(
            args.config.as_deref(),
        )
    }
}
