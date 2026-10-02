//! Load shared lint configuration once for the CLI.

use super::LintArgs;
use crate::config::{
    LinterConfigPlanWithConfigRuleOptions, LinterFeatureFlags, LoadedConfigWithFeatures,
};

pub(super) fn load(
    args: &mut LintArgs,
) -> (
    LoadedConfigWithFeatures,
    LinterConfigPlanWithConfigRuleOptions,
    LinterFeatureFlags,
) {
    let (config, plan, features, execution) = if args.no_config {
        (
            crate::config::LoadedConfigWithFeatures::default(),
            crate::config::LinterConfigPlanWithConfigRuleOptions::default(),
            crate::config::LinterFeatureFlags::default(),
            crate::config::LinterExecutionOptions::default(),
        )
    } else {
        crate::config::try_load_linter_execution_with_source(args.config.as_deref()).unwrap_or_else(
            |error| {
                eprintln!("\x1b[31mError:\x1b[0m {error}");
                std::process::exit(2);
            },
        )
    };
    args.cross_file |= execution.cross_file;
    args.cross_file_tree |= execution.cross_file_tree;
    args.cross_file_complexity |= execution.cross_file_complexity;
    args.max_warnings = args.max_warnings.or(execution.max_warnings);
    (config, plan, features)
}
