//! Types for variant auto-generation.

#![allow(clippy::disallowed_types)]

use serde::{Deserialize, Serialize};
use vize_carton::String;

/// Configuration for variant auto-generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutogenConfig {
    /// Maximum number of variants to generate.
    #[serde(default = "default_max_variants")]
    pub max_variants: usize,

    /// Include a "Default" variant with all default values.
    #[serde(default = "default_true")]
    pub include_default: bool,

    /// Include boolean toggle variants.
    #[serde(default = "default_true")]
    pub include_boolean_toggles: bool,

    /// Include enum/union variants (one per value).
    #[serde(default = "default_true")]
    pub include_enum_variants: bool,

    /// Include boundary value variants for numbers.
    #[serde(default)]
    pub include_boundary_values: bool,

    /// Include empty string variants for optional strings.
    #[serde(default)]
    pub include_empty_strings: bool,
}

impl Default for AutogenConfig {
    fn default() -> Self {
        Self {
            max_variants: 20,
            include_default: true,
            include_boolean_toggles: true,
            include_enum_variants: true,
            include_boundary_values: false,
            include_empty_strings: false,
        }
    }
}

fn default_max_variants() -> usize {
    20
}

fn default_true() -> bool {
    true
}

/// A prop definition used as input for variant generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropDefinition {
    /// Prop name.
    pub name: String,
    /// Prop type string (e.g., "string", "number", "'primary' | 'secondary'").
    pub prop_type: String,
    /// Whether the prop is required.
    pub required: bool,
    /// Default value as JSON.
    pub default_value: Option<serde_json::Value>,
}

/// A generated variant definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::disallowed_types)]
pub struct GeneratedVariant {
    /// Variant name (e.g., "Default", "Primary", "Disabled").
    pub name: String,
    /// Whether this should be the default variant.
    pub is_default: bool,
    /// Props to apply to the component.
    pub props: serde_json::Map<std::string::String, serde_json::Value>,
    /// Description of what this variant tests.
    pub description: Option<String>,
}

/// Output of variant auto-generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutogenOutput {
    /// Generated variants.
    pub variants: Vec<GeneratedVariant>,
    /// Generated `.art.vue` file content.
    pub art_file_content: String,
    /// Component name extracted from path.
    pub component_name: String,
}
