//! Shared Vize configuration loading.

mod loader;
mod model;

pub use loader::{LoadedConfig, load_config, load_config_with_source, load_linter_config};
pub use model::{
    ArrowParens, AttributeSortOrder, EndOfLine, FormatterConfig, GlobalTypeDeclaration,
    GlobalTypesConfig, LanguageServerConfig, LintRuleSeverity, LinterConfig, LspConfig, QuoteProps,
    TrailingComma, TypeCheckerConfig, VizeConfig,
};
