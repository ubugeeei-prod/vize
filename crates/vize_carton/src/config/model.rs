//! Shared config model.

mod compiler;
mod formatter;
mod global_types;
mod language_server;
mod linter;
mod linter_rule_options;
mod type_checker;
mod vue;

use serde::{Deserialize, Serialize};

use compiler::RawCompilerConfig;
use vue::RawVueConfig;

pub use compiler::JsxMode;

use crate::String;
use crate::dialect::VueDialect;
pub use formatter::{
    ArrowParens, AttributeSortOrder, EndOfLine, FormatterConfig, QuoteProps, TrailingComma,
};
pub use global_types::{GlobalTypeDeclaration, GlobalTypesConfig, RawGlobalTypesConfig};
pub use language_server::{LanguageServerConfig, LspConfig};
#[allow(unused_imports)]
pub(crate) use linter::RawLinterConfig;
pub use linter::{LintRuleSeverity, LinterConfig};
pub use linter_rule_options::{
    LintRuleOptions, NoRestrictedGlobalsOptions, NoRestrictedMembersOptions, RestrictedGlobal,
    RestrictedMember,
};
pub use type_checker::TypeCheckerConfig;
pub use vue::{ParseVueVersionError, VueVersion};

/// Effective shared configuration.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(default)]
pub struct VizeConfig {
    /// JSON Schema reference for legacy JSON editor autocompletion.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// Vue dialect profile for standalone HTML documents (`"vue"` or
    /// `"petite-vue"`). When absent, the dialect is detected structurally per
    /// document (see [`crate::dialect::standalone_html_dialect`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dialect: Option<VueDialect>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigFeatureFlags {
    /// Resolve Vue 3 Options API template bindings during type checking.
    /// Default-on (matches vue-tsc): an Options API SFC's template bindings
    /// (`data`/`computed`/`methods`/`props`) resolve without configuration.
    /// Set `typeChecker.optionsApi: false` to opt out. Available in the standard
    /// build (not a legacy feature).
    pub type_checker_options_api: bool,
    pub type_checker_legacy_vue2: bool,
    /// Opt-in type-checking of `.jsx`/`.tsx` Vue components (#1497). Default-off
    /// so mixed Vue/React repositories do not accidentally route React `.tsx`
    /// through the Vue JSX checker. Set `typeChecker.jsxTypecheck: true` to route
    /// `.jsx`/`.tsx` through the Vize JSX virtual-TS path instead of the verbatim
    /// passthrough.
    pub type_checker_jsx_typecheck: bool,
    pub language_server_legacy_vue2: Option<bool>,
    /// Dialect selected by `vue.version`; `None` when the key is absent
    /// (modern Vue 3). Validated at parse time — unknown or ambiguous values
    /// fail config loading instead of silently picking a line. Groundwork for
    /// legacy Vue support (#1392): consumers thread this into parser and
    /// transform options in follow-ups.
    pub vue_version: Option<VueVersion>,
    /// Default JSX/TSX output backend selected by `compiler.jsxMode` (#1496);
    /// `None` when the key is absent (treated as VDOM). The JS plugins and the
    /// native `compileJsx` binding thread this into the per-component
    /// mode-selection logic so a single module can still mix VDOM and Vapor via
    /// `"use vue:*"` directives.
    pub jsx_mode: Option<JsxMode>,
}

impl Default for ConfigFeatureFlags {
    fn default() -> Self {
        Self {
            // Options API resolution is default-on (matches vue-tsc).
            type_checker_options_api: true,
            type_checker_legacy_vue2: false,
            type_checker_jsx_typecheck: false,
            language_server_legacy_vue2: None,
            vue_version: None,
            jsx_mode: None,
        }
    }
}

/// Lint-only feature switches derived from config compatibility keys.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LinterFeatureFlags {
    pub vue_version: Option<VueVersion>,
    pub vapor: Option<bool>,
}

impl LinterFeatureFlags {
    pub(crate) fn from_config_features(
        features: ConfigFeatureFlags,
        compiler_compatibility_vue_version: Option<VueVersion>,
        compiler_vapor: Option<bool>,
    ) -> Self {
        let vue_version = features
            .vue_version
            .or(compiler_compatibility_vue_version)
            .or_else(|| {
                (features.type_checker_legacy_vue2
                    || features.language_server_legacy_vue2 == Some(true))
                .then_some(VueVersion::V2_7)
            });
        Self {
            vue_version,
            vapor: compiler_vapor,
        }
    }
}

/// Raw config representation with legacy aliases preserved for migration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(crate) struct RawVizeConfig {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    #[serde(rename = "basePath")]
    pub base_path: Option<String>,
    pub files: Option<Vec<String>>,
    pub dialect: Option<VueDialect>,
    pub formatter: FormatterConfig,
    pub(crate) compiler: RawCompilerConfig,
    pub(crate) vue: RawVueConfig,
    pub linter: RawLinterConfig,
    #[serde(rename = "typeChecker")]
    type_checker: RawTypeCheckerConfig,
    #[serde(rename = "languageServer")]
    language_server: RawLanguageServerConfig,
    #[serde(rename = "globalTypes")]
    pub global_types: RawGlobalTypesConfig,
    pub ignores: Option<Vec<String>>,
    pub entries: Option<Vec<RawConfigEntry>>,
    #[serde(rename = "check")]
    legacy_check: Option<LegacyCheckConfig>,
    #[serde(rename = "fmt")]
    legacy_formatter: Option<FormatterConfig>,
    #[serde(rename = "lsp")]
    legacy_lsp: Option<RawLanguageServerConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct RawConfigEntry {
    pub base_path: Option<String>,
    pub files: Option<Vec<String>>,
    pub ignores: Option<Vec<String>>,
    pub linter: RawLinterConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntryIgnore {
    pub base_path: Option<String>,
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntryFiles {
    pub base_path: Option<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawTypeCheckerConfig {
    #[serde(flatten)]
    config: TypeCheckerConfig,
    /// `None` when `typeChecker.optionsApi` is absent — defaults to enabled
    /// (matches vue-tsc). Set `false` to opt out.
    options_api: Option<bool>,
    legacy_vue2: bool,
    /// Opt-in type-checking of JSX/TSX Vue components (#1497). Default-off.
    jsx_typecheck: bool,
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
    /// Normalize raw config and derive auxiliary feature flags once.
    ///
    /// Legacy aliases (`check`, `fmt`, `lsp`) are folded here while the raw
    /// object is still owned. Callers that also need linter settings clone them
    /// before this conversion, which avoids a second deserialization pass.
    pub(crate) fn into_config_and_features(self) -> (VizeConfig, ConfigFeatureFlags) {
        let RawVizeConfig {
            schema,
            base_path: _,
            files: _,
            dialect,
            formatter,
            compiler,
            vue,
            linter: _,
            type_checker: raw_type_checker,
            language_server: raw_language_server,
            global_types,
            ignores: _,
            entries: _,
            legacy_check,
            legacy_formatter,
            legacy_lsp,
        } = self;

        // Default-on (matches vue-tsc); explicit `false` opts out.
        let type_checker_options_api = raw_type_checker.options_api.unwrap_or(true);
        let type_checker_legacy_vue2 = raw_type_checker.legacy_vue2;
        let type_checker_jsx_typecheck = raw_type_checker.jsx_typecheck;
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
            type_checker_options_api,
            type_checker_legacy_vue2,
            type_checker_jsx_typecheck,
            language_server_legacy_vue2: language_server_raw.legacy_vue2,
            vue_version: vue.version.or(compiler.compatibility.vue_version),
            jsx_mode: compiler.jsx_mode,
        };

        let config = VizeConfig {
            schema,
            dialect,
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
