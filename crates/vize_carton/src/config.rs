//! Shared Vize configuration loading.

mod loader;
mod model;
mod normalize;

pub use loader::{
    LoadedConfig, LoadedConfigEntryFiles, LoadedConfigEntryIgnores, LoadedConfigWithFeatures,
    load_compiler_host_compiler, load_compiler_jsx_mode, load_compiler_template_syntax,
    load_compiler_vue_version, load_config, load_config_and_linter_with_features_and_source,
    load_config_and_linter_with_lint_features_and_source, load_config_and_linter_with_source,
    load_config_entry_files_with_source, load_config_entry_ignores_with_source,
    load_config_with_features_and_source, load_config_with_source, load_linter_config,
    load_linter_rule_options, validate_explicit_config_path,
};
pub use model::{
    ArrowParens, AttributeSortOrder, ConfigEntryFiles, ConfigEntryIgnore, ConfigFeatureFlags,
    EndOfLine, FormatterConfig, GlobalTypeDeclaration, GlobalTypesConfig, JsxMode,
    LanguageServerConfig, LintRuleOptions, LintRuleSeverity, LinterConfig, LinterFeatureFlags,
    LspConfig, NoRestrictedGlobalsOptions, NoRestrictedMembersOptions, ParseVueVersionError,
    QuoteProps, RestrictedGlobal, RestrictedMember, TrailingComma, TypeCheckerConfig, VizeConfig,
    VueVersion,
};
pub use normalize::normalize_public_config_value;

pub use crate::dialect::VueDialect;
