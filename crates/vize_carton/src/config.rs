//! Host discovery and evaluation of Vize configuration.

#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "host config evaluation retains its existing std error and process strings"
)]
mod loader;
pub use loader::{
    LoadedConfig, LoadedConfigEntryFiles, LoadedConfigEntryIgnores,
    LoadedConfigExperimentalVueFlags, LoadedConfigWithFeatures, LoadedLibConfig, LoadedLspConfig,
    load_compiler_custom_elements, load_compiler_host_compiler, load_compiler_jsx_compat,
    load_compiler_jsx_mode, load_compiler_template_syntax, load_compiler_vapor,
    load_compiler_vue_version, load_config,
    load_config_and_linter_plan_with_config_rule_options_and_lint_features_and_source,
    load_config_and_linter_plan_with_lint_features_and_source,
    load_config_and_linter_plan_with_rule_options_and_lint_features_and_source,
    load_config_and_linter_with_features_and_source,
    load_config_and_linter_with_lint_features_and_source, load_config_and_linter_with_source,
    load_config_entry_files_with_source, load_config_entry_ignores_with_source,
    load_config_experimental_vue_flags_with_source, load_config_lint_rule_options,
    load_config_with_features_and_source, load_config_with_source,
    load_language_server_unstable_flags, load_lib_config_with_source, load_linter_config,
    load_linter_rule_options, load_lsp_config_snapshot, validate_explicit_config_path,
};
pub use vize_l0::config::*;
