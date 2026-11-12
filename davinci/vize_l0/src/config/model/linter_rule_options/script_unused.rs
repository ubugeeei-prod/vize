//! Options for `vue/no-unused-script-vars` and `vue/prefer-const`.
//!
//! The keys follow typescript-eslint `no-unused-vars` and ESLint
//! `prefer-const`. An empty pattern string turns a default pattern off.

use serde::{Deserialize, Serialize};

use crate::String;

/// Which unused variables `vue/no-unused-script-vars` checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnusedVarsScope {
    /// Every variable, including module-level ones.
    All,
    /// Only variables declared inside functions and blocks.
    Local,
}

/// Which unused parameters `vue/no-unused-script-vars` checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnusedArgs {
    /// Every parameter.
    All,
    /// Parameters after the last used one, and destructured ones.
    AfterUsed,
    /// No parameter.
    None,
}

/// Whether `vue/no-unused-script-vars` checks `catch` bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnusedCaughtErrors {
    /// Every caught error.
    All,
    /// No caught error.
    None,
}

/// `vue/no-unused-script-vars`, in the typescript-eslint `no-unused-vars` shape.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoUnusedScriptVarsOptions {
    /// `all` (default) or `local`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vars: Option<UnusedVarsScope>,
    /// Unused variables to allow. Defaults to `^_`; `""` allows none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vars_ignore_pattern: Option<String>,
    /// `after-used` (default), `all`, or `none`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<UnusedArgs>,
    /// Unused parameters to allow. Defaults to `^_`; `""` allows none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args_ignore_pattern: Option<String>,
    /// `all` (default) or `none`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caught_errors: Option<UnusedCaughtErrors>,
    /// Unused `catch` bindings to allow. No default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caught_errors_ignore_pattern: Option<String>,
    /// Unused array-destructuring elements to allow. No default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructured_array_ignore_pattern: Option<String>,
    /// Allow the siblings of a used object rest element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_rest_siblings: Option<bool>,
    /// Allow an unused class that has a `static {}` block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_class_with_static_init_block: Option<bool>,
    /// Report a used binding that matches an ignore pattern.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_used_ignore_pattern: Option<bool>,
}

/// How `vue/prefer-const` treats destructuring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreferConstDestructuring {
    /// Report each binding that could be `const`.
    Any,
    /// Report only when every binding of the destructuring could be `const`.
    All,
}

/// `vue/prefer-const`, in the ESLint `prefer-const` shape.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct PreferConstOptions {
    /// `any` (default) or `all`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructuring: Option<PreferConstDestructuring>,
    /// Skip a `let` that is read before its only assignment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_read_before_assign: Option<bool>,
}
