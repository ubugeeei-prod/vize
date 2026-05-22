//! Shared config model.

mod formatter;
mod global_types;
mod language_server;
mod linter;
mod type_checker;

use serde::{Deserialize, Serialize};

use crate::String;

pub use formatter::{
    ArrowParens, AttributeSortOrder, EndOfLine, FormatterConfig, QuoteProps, TrailingComma,
};
pub use global_types::{GlobalTypeDeclaration, GlobalTypesConfig, RawGlobalTypesConfig};
pub use language_server::{LanguageServerConfig, LspConfig};
#[allow(unused_imports)]
pub use linter::{LintRuleSeverity, LinterConfig};
pub use type_checker::TypeCheckerConfig;

/// Effective shared configuration.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(default)]
pub struct VizeConfig {
    /// JSON Schema reference for legacy JSON editor autocompletion.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// Formatter settings shared by CLI and IDE formatting.
    #[serde(skip_serializing_if = "FormatterConfig::is_default")]
    pub formatter: FormatterConfig,
    /// Linter settings shared by CLI and IDE linting.
    #[serde(skip_serializing_if = "LinterConfig::is_default")]
    pub linter: LinterConfig,
    /// Type checker settings shared by CLI and IDE diagnostics.
    #[serde(
        rename = "typeChecker",
        skip_serializing_if = "TypeCheckerConfig::is_default"
    )]
    pub type_checker: TypeCheckerConfig,
    /// IDE language server feature flags.
    #[serde(
        rename = "languageServer",
        skip_serializing_if = "LanguageServerConfig::is_default"
    )]
    pub language_server: LanguageServerConfig,
    /// Template global declarations.
    #[serde(
        rename = "globalTypes",
        skip_serializing_if = "GlobalTypesConfig::is_empty"
    )]
    pub global_types: GlobalTypesConfig,
}

/// Raw config representation with legacy aliases preserved for migration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(crate) struct RawVizeConfig {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub formatter: FormatterConfig,
    pub linter: LinterConfig,
    #[serde(rename = "typeChecker")]
    pub type_checker: TypeCheckerConfig,
    #[serde(rename = "languageServer")]
    pub language_server: LanguageServerConfig,
    #[serde(rename = "globalTypes")]
    pub global_types: RawGlobalTypesConfig,
    #[serde(rename = "check")]
    legacy_check: Option<LegacyCheckConfig>,
    #[serde(rename = "fmt")]
    legacy_formatter: Option<FormatterConfig>,
    #[serde(rename = "lsp")]
    legacy_lsp: Option<LanguageServerConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct LegacyCheckConfig {
    globals: Option<String>,
    servers: Option<usize>,
}

impl From<RawVizeConfig> for VizeConfig {
    fn from(raw: RawVizeConfig) -> Self {
        let mut type_checker = raw.type_checker;
        if let Some(legacy_check) = raw.legacy_check {
            if type_checker.globals_file.is_none() {
                type_checker.globals_file = legacy_check.globals;
            }
            if type_checker.servers.is_none() {
                type_checker.servers = legacy_check.servers;
            }
        }

        let formatter = if raw.formatter == FormatterConfig::default() {
            raw.legacy_formatter.unwrap_or(raw.formatter)
        } else {
            raw.formatter
        };

        let language_server = if raw.language_server == LanguageServerConfig::default() {
            raw.legacy_lsp.unwrap_or(raw.language_server)
        } else {
            raw.language_server
        };

        Self {
            schema: raw.schema,
            formatter,
            linter: raw.linter,
            type_checker,
            language_server,
            global_types: raw.global_types.into(),
        }
    }
}
