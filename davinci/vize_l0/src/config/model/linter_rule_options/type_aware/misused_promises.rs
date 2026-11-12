//! Options of the promise-misuse rules: no-meaningless-void-operator and
//! no-misused-promises.

use serde::{Deserialize, Serialize};

/// `type/no-meaningless-void-operator`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoMeaninglessVoidOperatorOptions {
    /// Also report `void` on `never`-typed expressions.
    pub check_never: bool,
}

/// Which union types `checksConditionals` reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromiseUnionFlag {
    /// Report any union with a promise constituent.
    All,
    /// Report only unions whose constituents are all promises.
    Strict,
    /// Never report unions.
    None,
}

/// Object form of `checksConditionals`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ChecksConditionalsOptions {
    /// Which unions to report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag_unions: Option<PromiseUnionFlag>,
}

/// Object form of `checksVoidReturn`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ChecksVoidReturnOptions {
    /// Promise-returning functions passed as `void`-returning arguments.
    pub arguments: bool,
    /// Promise-returning functions passed as `void`-returning JSX attributes.
    pub attributes: bool,
    /// Promise-returning methods overriding `void`-returning ones.
    pub inherited_methods: bool,
    /// Promise-returning functions assigned to `void`-returning properties.
    pub properties: bool,
    /// Promise-returning functions returned where `void` is expected.
    pub returns: bool,
    /// Promise-returning functions assigned to `void`-returning variables.
    pub variables: bool,
}

impl Default for ChecksVoidReturnOptions {
    fn default() -> Self {
        Self {
            arguments: true,
            attributes: true,
            inherited_methods: true,
            properties: true,
            returns: true,
            variables: true,
        }
    }
}

/// A boolean switch, or an object with finer settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Switch<T> {
    /// On or off.
    Flag(bool),
    /// On, with settings.
    Settings(T),
}

/// `type/no-misused-promises`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoMisusedPromisesOptions {
    /// Check promises used as conditions.
    pub checks_conditionals: Switch<ChecksConditionalsOptions>,
    /// Check spreading promises into objects.
    pub checks_spreads: bool,
    /// Check promise-returning functions where `void` is expected.
    pub checks_void_return: Switch<ChecksVoidReturnOptions>,
}

impl Default for NoMisusedPromisesOptions {
    fn default() -> Self {
        Self {
            checks_conditionals: Switch::Flag(true),
            checks_spreads: true,
            checks_void_return: Switch::Flag(true),
        }
    }
}
