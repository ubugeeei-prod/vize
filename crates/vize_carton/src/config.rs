//! Shared Vize configuration loading.

mod loader;
mod model;
mod normalize;

pub use loader::{
    LoadedConfig, LoadedConfigWithFeatures, load_compiler_template_syntax, load_config,
    load_config_and_linter_with_features_and_source, load_config_and_linter_with_source,
    load_config_with_features_and_source, load_config_with_source, load_linter_config,
    validate_explicit_config_path,
};
pub use model::{
    ArrowParens, AttributeSortOrder, ConfigFeatureFlags, EndOfLine, FormatterConfig,
    GlobalTypeDeclaration, GlobalTypesConfig, LanguageServerConfig, LintRuleSeverity, LinterConfig,
    LspConfig, QuoteProps, TrailingComma, TypeCheckerConfig, VizeConfig,
};
pub use normalize::normalize_public_config_value;

pub use crate::dialect::VueDialect;
