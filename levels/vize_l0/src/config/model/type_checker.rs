//! Type checker config model.

use serde::{Deserialize, Deserializer, Serialize};

use crate::String;

/// Type checking settings shared by CLI and IDE diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeCheckerConfig {
    pub enabled: bool,
    pub strict: bool,
    pub check_props: bool,
    pub check_emits: bool,
    pub check_template_bindings: bool,
    pub check_reactivity: bool,
    pub check_setup_context: bool,
    pub check_invalid_exports: bool,
    pub check_fallthrough_attrs: bool,
    pub tsconfig: Option<String>,
    /// Path to the Corsa executable, serialized as `corsaPath`.
    ///
    /// The Rust field name stays `tsgo_path` to preserve the public crate API.
    #[serde(rename = "corsaPath")]
    pub tsgo_path: Option<String>,
    pub globals_file: Option<String>,
    pub servers: Option<usize>,
}

impl TypeCheckerConfig {
    /// Canonical Corsa executable path, with the legacy `tsgoPath` key as a fallback.
    pub fn runtime_path(&self) -> Option<&str> {
        self.tsgo_path.as_deref()
    }

    /// Returns true when the config matches the built-in defaults.
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

impl Default for TypeCheckerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strict: false,
            check_props: true,
            check_emits: true,
            check_template_bindings: true,
            check_reactivity: true,
            check_setup_context: true,
            check_invalid_exports: true,
            check_fallthrough_attrs: true,
            tsconfig: None,
            tsgo_path: None,
            globals_file: None,
            servers: None,
        }
    }
}

impl<'de> Deserialize<'de> for TypeCheckerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = TypeCheckerConfigDeserialize::deserialize(deserializer)?;

        Ok(Self {
            enabled: helper.enabled,
            strict: helper.strict,
            check_props: helper.check_props,
            check_emits: helper.check_emits,
            check_template_bindings: helper.check_template_bindings,
            check_reactivity: helper.check_reactivity,
            check_setup_context: helper.check_setup_context,
            check_invalid_exports: helper.check_invalid_exports,
            check_fallthrough_attrs: helper.check_fallthrough_attrs,
            tsconfig: helper.tsconfig,
            tsgo_path: helper.corsa_path.or(helper.tsgo_path),
            globals_file: helper.globals_file,
            servers: helper.servers,
        })
    }
}

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct TypeCheckerConfigDeserialize {
    enabled: bool,
    strict: bool,
    check_props: bool,
    check_emits: bool,
    check_template_bindings: bool,
    check_reactivity: bool,
    check_setup_context: bool,
    check_invalid_exports: bool,
    check_fallthrough_attrs: bool,
    tsconfig: Option<String>,
    corsa_path: Option<String>,
    tsgo_path: Option<String>,
    globals_file: Option<String>,
    servers: Option<usize>,
}

impl Default for TypeCheckerConfigDeserialize {
    fn default() -> Self {
        let config = TypeCheckerConfig::default();
        Self {
            enabled: config.enabled,
            strict: config.strict,
            check_props: config.check_props,
            check_emits: config.check_emits,
            check_template_bindings: config.check_template_bindings,
            check_reactivity: config.check_reactivity,
            check_setup_context: config.check_setup_context,
            check_invalid_exports: config.check_invalid_exports,
            check_fallthrough_attrs: config.check_fallthrough_attrs,
            tsconfig: config.tsconfig,
            corsa_path: None,
            tsgo_path: config.tsgo_path,
            globals_file: config.globals_file,
            servers: config.servers,
        }
    }
}
