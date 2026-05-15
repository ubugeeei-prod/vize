//! Shared Vize configuration loading.

mod loader;
mod model;

pub use loader::{load_config, load_config_with_source, LoadedConfig};
pub use model::{
    ArrowParens, AttributeSortOrder, EndOfLine, FormatterConfig, GlobalTypeDeclaration,
    GlobalTypesConfig, LanguageServerConfig, LspConfig, QuoteProps, TrailingComma,
    TypeCheckerConfig, VizeConfig,
};
