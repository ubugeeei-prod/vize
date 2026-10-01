//! Oxfmt-compatible import sorting configuration.
use crate::String;
use serde::{Deserialize, Serialize};

/// Import sorting is disabled with `false` or configured with an object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum SortImportsSetting {
    Disabled(bool),
    Config(SortImportsConfig),
}

/// Import sorting controls passed to the native script formatter.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct SortImportsConfig {
    pub partition_by_newline: Option<bool>,
    pub partition_by_comment: Option<bool>,
    pub sort_side_effects: Option<bool>,
    pub order: Option<String>,
    pub ignore_case: Option<bool>,
    pub newlines_between: Option<bool>,
    pub internal_pattern: Option<Vec<String>>,
    pub groups: Option<Vec<ImportSortGroup>>,
    pub custom_groups: Option<Vec<ImportSortCustomGroup>>,
}

/// A group name, combined group, or explicit newline boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ImportSortGroup {
    Name(String),
    Names(Vec<String>),
    Boundary {
        #[serde(rename = "newlinesBetween")]
        newlines_between: bool,
    },
}

/// Custom import category matched by module name, selector, and modifiers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImportSortCustomGroup {
    pub group_name: String,
    #[serde(default)]
    pub element_name_pattern: Vec<String>,
    pub selector: Option<String>,
    #[serde(default)]
    pub modifiers: Vec<String>,
}
