//! Type checker config model.

use serde::{Deserialize, Serialize};

use crate::String;

/// Type checking settings shared by CLI and IDE diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
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
    pub tsgo_path: Option<String>,
    pub globals_file: Option<String>,
    pub servers: Option<usize>,
}

impl TypeCheckerConfig {
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
