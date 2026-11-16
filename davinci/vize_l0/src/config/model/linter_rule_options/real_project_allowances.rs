//! Typed allowances for real-project Vue and accessibility conventions.

use serde::{Deserialize, Serialize};

use crate::String;

/// Options for `vue/warn-custom-directive`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct WarnCustomDirectiveOptions {
    /// Exact directive names without the `v-` prefix.
    pub ignore: Vec<String>,
}

/// Options for `vue/no-bare-strings-in-template`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoBareStringsInTemplateOptions {
    /// Literal substrings and `/pattern/flags` entries removed before checking.
    /// `None` preserves eslint-plugin-vue's default punctuation list, while an
    /// authored empty list intentionally clears it.
    pub allowlist: Option<Vec<String>>,
}

/// Options for `a11y/anchor-has-content` and `a11y/heading-has-content`, in
/// eslint-plugin-vuejs-accessibility's shape.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct AnchorHasContentOptions {
    /// Component names checked as if they were the native element.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
    /// Component names whose rendered subtree always provides an accessible name.
    pub accessible_children: Vec<String>,
    /// Directive names, without `v-`, that fill the element they sit on.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub accessible_directives: Vec<String>,
}

/// Options for `a11y/heading-has-content`, in the same shape.
pub type HeadingHasContentOptions = AnchorHasContentOptions;

/// Options for `a11y/label-has-for`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct LabelHasForOptions {
    /// Component names that render a native form control.
    pub control_components: Vec<String>,
}
