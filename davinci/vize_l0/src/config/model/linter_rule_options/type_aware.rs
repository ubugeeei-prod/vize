//! Options for the typescript-eslint type-aware rules.
//!
//! Each struct mirrors the option object of the matching
//! `@typescript-eslint/<rule>` rule, with the same names and defaults, so a
//! project can carry its typescript-eslint configuration over unchanged.
//! Omitted keys take typescript-eslint's default.

mod misused_promises;

pub use misused_promises::{
    ChecksConditionalsOptions, ChecksVoidReturnOptions, NoMeaninglessVoidOperatorOptions,
    NoMisusedPromisesOptions, PromiseUnionFlag, Switch,
};
use serde::{Deserialize, Serialize};

use crate::String;

/// `type/no-confusing-void-expression`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoConfusingVoidExpressionOptions {
    /// Ignore shorthand `() => expr` arrow functions.
    pub ignore_arrow_shorthand: bool,
    /// Ignore returns that start with the `void` operator.
    pub ignore_void_operator: bool,
    /// Ignore returns from functions whose return type is `void`.
    pub ignore_void_returning_functions: bool,
}

/// When `type/no-unnecessary-condition` ignores a constant loop condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConstantLoopConditions {
    /// `true` is `"always"`, `false` is `"never"`.
    Flag(bool),
    /// One of the named modes.
    Mode(ConstantLoopConditionMode),
}

/// Named modes of `allowConstantLoopConditions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConstantLoopConditionMode {
    /// Ignore every constant loop condition.
    Always,
    /// Report every constant loop condition.
    Never,
    /// Ignore `true`, `false`, `1` and `0` as loop conditions.
    OnlyAllowedLiterals,
}

impl ConstantLoopConditions {
    /// The mode after resolving `true`, `false` and the default.
    pub fn mode(self) -> ConstantLoopConditionMode {
        match self {
            Self::Flag(true) => ConstantLoopConditionMode::Always,
            Self::Flag(false) => ConstantLoopConditionMode::Never,
            Self::Mode(mode) => mode,
        }
    }
}

/// `type/no-unnecessary-condition`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoUnnecessaryConditionOptions {
    /// Whether constant loop conditions such as `while (true)` are allowed.
    /// Absent means typescript-eslint's default, `"never"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_constant_loop_conditions: Option<ConstantLoopConditions>,
    /// Check the asserted argument of a type predicate call.
    pub check_type_predicates: bool,
    /// Accepted for typescript-eslint compatibility; Vize always runs the rule.
    #[serde(rename = "allowRuleToRunWithoutStrictNullChecksIKnowWhatIAmDoing")]
    pub allow_without_strict_null_checks: bool,
}

impl NoUnnecessaryConditionOptions {
    /// The resolved `allowConstantLoopConditions` mode.
    pub fn constant_loop_mode(&self) -> ConstantLoopConditionMode {
        self.allow_constant_loop_conditions.map_or(
            ConstantLoopConditionMode::Never,
            ConstantLoopConditions::mode,
        )
    }
}

/// `type/restrict-plus-operands`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct RestrictPlusOperandsOptions {
    /// Allow `any` operands.
    pub allow_any: bool,
    /// Allow `boolean` operands.
    pub allow_boolean: bool,
    /// Allow operands that may be `null` or `undefined`.
    pub allow_nullish: bool,
    /// Allow adding a `number` or `bigint` to a `string`.
    pub allow_number_and_string: bool,
    /// Allow `RegExp` operands.
    #[serde(rename = "allowRegExp")]
    pub allow_regexp: bool,
    /// Skip `+=` compound assignments.
    pub skip_compound_assignments: bool,
}

impl Default for RestrictPlusOperandsOptions {
    fn default() -> Self {
        Self {
            allow_any: true,
            allow_boolean: true,
            allow_nullish: true,
            allow_number_and_string: true,
            allow_regexp: true,
            skip_compound_assignments: false,
        }
    }
}

/// One or more type names in a type specifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypeSpecifierNames {
    /// One name.
    One(String),
    /// Several names.
    Many(Vec<String>),
}

/// Where a type named by a specifier is declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TypeSpecifierSource {
    /// A file of the project.
    File,
    /// TypeScript's default library.
    Lib,
    /// An installed package.
    Package,
}

/// A typescript-eslint `TypeOrValueSpecifier`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypeSpecifier {
    /// A bare type name, matched wherever it is declared.
    Name(String),
    /// A type name with its declaration source.
    Sourced {
        /// Declaration source.
        from: TypeSpecifierSource,
        /// Type names.
        name: TypeSpecifierNames,
        /// File path, for `from: "file"`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        /// Package name, for `from: "package"`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        package: Option<String>,
    },
}

/// `type/restrict-template-expressions`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct RestrictTemplateExpressionsOptions {
    /// Allow `any` values.
    pub allow_any: bool,
    /// Allow array values.
    pub allow_array: bool,
    /// Allow `boolean` values.
    pub allow_boolean: bool,
    /// Allow `null` and `undefined` values.
    pub allow_nullish: bool,
    /// Allow `number` and `bigint` values.
    pub allow_number: bool,
    /// Allow `RegExp` values.
    #[serde(rename = "allowRegExp")]
    pub allow_regexp: bool,
    /// Allow `never` values.
    pub allow_never: bool,
    /// Further types to allow. The default allows `Error`, `URL` and
    /// `URLSearchParams` from the default library.
    pub allow: Vec<TypeSpecifier>,
}

impl Default for RestrictTemplateExpressionsOptions {
    fn default() -> Self {
        Self {
            allow_any: true,
            allow_array: false,
            allow_boolean: true,
            allow_nullish: true,
            allow_number: true,
            allow_regexp: true,
            allow_never: false,
            allow: vec![TypeSpecifier::Sourced {
                from: TypeSpecifierSource::Lib,
                name: TypeSpecifierNames::Many(vec![
                    "Error".into(),
                    "URL".into(),
                    "URLSearchParams".into(),
                ]),
                path: None,
                package: None,
            }],
        }
    }
}

/// `type/return-await`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReturnAwaitOption {
    /// Await returned promises in error-handling contexts and nowhere else.
    #[default]
    InTryCatch,
    /// Await every returned promise.
    Always,
    /// Await returned promises in error-handling contexts; allow both elsewhere.
    ErrorHandlingCorrectnessOnly,
    /// Never await a returned promise.
    Never,
}

/// A list of typescript-eslint `TypeOrValueSpecifier`s.
pub type TypeSpecifiers = Vec<TypeSpecifier>;

/// `type/await-thenable`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct AwaitThenableOptions {
    /// Allow `await Promise.all([...])`-style arrays that mix promises and values.
    pub allow_mixed_promise_arrays: bool,
}

/// `type/no-base-to-string`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoBaseToStringOptions {
    /// Also report values typed `unknown`.
    pub check_unknown: bool,
    /// Type names whose `toString()` is known to be meaningful.
    pub ignored_type_names: Vec<String>,
}

impl Default for NoBaseToStringOptions {
    fn default() -> Self {
        Self {
            check_unknown: false,
            ignored_type_names: vec![
                "Error".into(),
                "RegExp".into(),
                "URL".into(),
                "URLSearchParams".into(),
            ],
        }
    }
}

/// A rule whose only option is a list of allowed types or values.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct AllowListOptions {
    /// Types or values the rule accepts.
    pub allow: TypeSpecifiers,
}

/// `type/no-duplicate-type-constituents`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoDuplicateTypeConstituentsOptions {
    /// Skip intersection types.
    pub ignore_intersections: bool,
    /// Skip union types.
    pub ignore_unions: bool,
}

/// `type/no-floating-promises`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoFloatingPromisesOptions {
    /// Calls whose returned promise may float.
    pub allow_for_known_safe_calls: TypeSpecifiers,
    /// Promise types that may float.
    pub allow_for_known_safe_promises: TypeSpecifiers,
    /// Also check thenables that are not `Promise`s.
    pub check_thenables: bool,
    /// Ignore immediately invoked async functions.
    #[serde(rename = "ignoreIIFE")]
    pub ignore_iife: bool,
    /// Accept `void promise` as handling.
    pub ignore_void: bool,
}

impl Default for NoFloatingPromisesOptions {
    fn default() -> Self {
        Self {
            allow_for_known_safe_calls: Vec::new(),
            allow_for_known_safe_promises: Vec::new(),
            check_thenables: false,
            ignore_iife: false,
            ignore_void: true,
        }
    }
}

/// `type/no-unnecessary-boolean-literal-compare`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoUnnecessaryBooleanLiteralCompareOptions {
    /// Allow `nullableBoolean === false`.
    pub allow_comparing_nullable_booleans_to_false: bool,
    /// Allow `nullableBoolean === true`.
    pub allow_comparing_nullable_booleans_to_true: bool,
    /// Accepted for typescript-eslint compatibility; Vize always runs the rule.
    #[serde(rename = "allowRuleToRunWithoutStrictNullChecksIKnowWhatIAmDoing")]
    pub allow_without_strict_null_checks: bool,
}

impl Default for NoUnnecessaryBooleanLiteralCompareOptions {
    fn default() -> Self {
        Self {
            allow_comparing_nullable_booleans_to_false: true,
            allow_comparing_nullable_booleans_to_true: true,
            allow_without_strict_null_checks: false,
        }
    }
}

/// `type/no-unnecessary-type-assertion`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoUnnecessaryTypeAssertionOptions {
    /// Also report `as const` on literals whose type is already literal.
    pub check_literal_const_assertions: bool,
    /// Asserted type names to skip.
    pub types_to_ignore: Vec<String>,
}

/// `type/no-unsafe-member-access`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoUnsafeMemberAccessOptions {
    /// Allow `anyValue?.member`.
    pub allow_optional_chaining: bool,
}

/// `type/only-throw-error`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct OnlyThrowErrorOptions {
    /// Types that may be thrown.
    pub allow: TypeSpecifiers,
    /// Allow rethrowing a caught value.
    pub allow_rethrowing: bool,
    /// Allow throwing `any`.
    pub allow_throwing_any: bool,
    /// Allow throwing `unknown`.
    pub allow_throwing_unknown: bool,
}

impl Default for OnlyThrowErrorOptions {
    fn default() -> Self {
        Self {
            allow: Vec::new(),
            allow_rethrowing: true,
            allow_throwing_any: true,
            allow_throwing_unknown: true,
        }
    }
}

/// `type/prefer-promise-reject-errors`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct PreferPromiseRejectErrorsOptions {
    /// Types that may be rejection reasons.
    pub allow: TypeSpecifiers,
    /// Allow `reject()` with no argument.
    pub allow_empty_reject: bool,
    /// Allow rejecting with `any`.
    pub allow_throwing_any: bool,
    /// Allow rejecting with `unknown`.
    pub allow_throwing_unknown: bool,
}

/// `type/unbound-method`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct UnboundMethodOptions {
    /// Skip static methods.
    pub ignore_static: bool,
}

/// Declares [`TypeAwareRuleOptions`] from one list of `(field, rule, type)`.
macro_rules! type_aware_rule_options {
    ($($field:ident: $rule:literal => $ty:ty,)+) => {
        /// Flattened options for the type-aware rules, keyed by rule name.
        #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(default)]
        pub struct TypeAwareRuleOptions {
            $(
                #[doc = concat!("`", $rule, "`.")]
                #[serde(rename = $rule, skip_serializing_if = "Option::is_none")]
                pub $field: Option<$ty>,
            )+
        }

        impl TypeAwareRuleOptions {
            /// Rule names that accept options, in declaration order.
            pub const RULES: &'static [&'static str] = &[$($rule),+];

            /// Whether no type-aware rule has options.
            pub fn is_empty(&self) -> bool {
                true $(&& self.$field.is_none())+
            }

            /// Apply a later config layer; a configured rule replaces the earlier entry.
            pub fn merge_from(&mut self, overlay: &Self) {
                $(
                    if overlay.$field.is_some() {
                        self.$field = overlay.$field.clone();
                    }
                )+
            }

            /// The configured options of `rule` as the typescript-eslint option
            /// value (the first element of ESLint's option list), with every
            /// default filled in.
            pub fn rule_value(&self, rule: &str) -> Option<serde_json::Value> {
                match rule {
                    $($rule => serde_json::to_value(self.$field.as_ref()?).ok(),)+
                    _ => None,
                }
            }
        }
    };
}

type_aware_rule_options! {
    await_thenable: "type/await-thenable" => AwaitThenableOptions,
    no_base_to_string: "type/no-base-to-string" => NoBaseToStringOptions,
    no_confusing_void_expression: "type/no-confusing-void-expression" => NoConfusingVoidExpressionOptions,
    no_deprecated: "type/no-deprecated" => AllowListOptions,
    no_duplicate_type_constituents: "type/no-duplicate-type-constituents" => NoDuplicateTypeConstituentsOptions,
    no_floating_promises: "type/no-floating-promises" => NoFloatingPromisesOptions,
    no_meaningless_void_operator: "type/no-meaningless-void-operator" => NoMeaninglessVoidOperatorOptions,
    no_misused_promises: "type/no-misused-promises" => NoMisusedPromisesOptions,
    no_misused_spread: "type/no-misused-spread" => AllowListOptions,
    no_unnecessary_boolean_literal_compare: "type/no-unnecessary-boolean-literal-compare" => NoUnnecessaryBooleanLiteralCompareOptions,
    no_unnecessary_condition: "type/no-unnecessary-condition" => NoUnnecessaryConditionOptions,
    no_unnecessary_type_assertion: "type/no-unnecessary-type-assertion" => NoUnnecessaryTypeAssertionOptions,
    no_unsafe_member_access: "type/no-unsafe-member-access" => NoUnsafeMemberAccessOptions,
    only_throw_error: "type/only-throw-error" => OnlyThrowErrorOptions,
    prefer_promise_reject_errors: "type/prefer-promise-reject-errors" => PreferPromiseRejectErrorsOptions,
    restrict_plus_operands: "type/restrict-plus-operands" => RestrictPlusOperandsOptions,
    restrict_template_expressions: "type/restrict-template-expressions" => RestrictTemplateExpressionsOptions,
    return_await: "type/return-await" => ReturnAwaitOption,
    unbound_method: "type/unbound-method" => UnboundMethodOptions,
}

impl super::ConfigLintRuleOptions {
    /// Options for the typescript-eslint type-aware rules.
    #[inline]
    pub fn type_aware(&self) -> &TypeAwareRuleOptions {
        &self.type_aware
    }
}

#[cfg(test)]
mod tests;
