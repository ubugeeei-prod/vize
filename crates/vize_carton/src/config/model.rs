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

/// Feature flags parsed from config keys that are not exposed as stable Rust
/// model fields.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfigFeatureFlags {
    pub type_checker_legacy_vue2: bool,
    pub language_server_legacy_vue2: Option<bool>,
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
    type_checker: RawTypeCheckerConfig,
    #[serde(rename = "languageServer")]
    language_server: RawLanguageServerConfig,
    #[serde(rename = "globalTypes")]
    pub global_types: RawGlobalTypesConfig,
    #[serde(rename = "check")]
    legacy_check: Option<LegacyCheckConfig>,
    #[serde(rename = "fmt")]
    legacy_formatter: Option<FormatterConfig>,
    #[serde(rename = "lsp")]
    legacy_lsp: Option<RawLanguageServerConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawTypeCheckerConfig {
    #[serde(flatten)]
    config: TypeCheckerConfig,
    legacy_vue2: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawLanguageServerConfig {
    #[serde(flatten)]
    config: LanguageServerConfig,
    legacy_vue2: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct LegacyCheckConfig {
    globals: Option<String>,
    servers: Option<usize>,
}

impl RawVizeConfig {
    pub(crate) fn into_config_and_features(self) -> (VizeConfig, ConfigFeatureFlags) {
        let RawVizeConfig {
            schema,
            formatter,
            linter: _,
            type_checker: raw_type_checker,
            language_server: raw_language_server,
            global_types,
            legacy_check,
            legacy_formatter,
            legacy_lsp,
        } = self;

        let type_checker_legacy_vue2 = raw_type_checker.legacy_vue2;
        let mut type_checker = raw_type_checker.config;
        if let Some(legacy_check) = legacy_check {
            if type_checker.globals_file.is_none() {
                type_checker.globals_file = legacy_check.globals;
            }
            if type_checker.servers.is_none() {
                type_checker.servers = legacy_check.servers;
            }
        }

        let formatter = if formatter == FormatterConfig::default() {
            legacy_formatter.unwrap_or(formatter)
        } else {
            formatter
        };

        let language_server_raw = if raw_language_server.config == LanguageServerConfig::default() {
            legacy_lsp.unwrap_or(raw_language_server)
        } else {
            raw_language_server
        };
        let language_server = language_server_raw.config;
        let features = ConfigFeatureFlags {
            type_checker_legacy_vue2,
            language_server_legacy_vue2: language_server_raw.legacy_vue2,
        };

        let config = VizeConfig {
            schema,
            formatter,
            type_checker,
            language_server,
            global_types: global_types.into(),
        };

        (config, features)
    }
}

impl From<RawVizeConfig> for VizeConfig {
    fn from(raw: RawVizeConfig) -> Self {
        raw.into_config_and_features().0
    }
}
