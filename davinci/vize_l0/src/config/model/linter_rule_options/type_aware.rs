//! Options for the typescript-eslint type-aware rules.
//!
//! Each struct mirrors the option object of the matching
//! `@typescript-eslint/<rule>` rule, with the same names and defaults, so a
//! project can carry its typescript-eslint configuration over unchanged.
//! Omitted keys take typescript-eslint's default.

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

/// Flattened options for the type-aware rules.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TypeAwareRuleOptions {
    /// `type/no-confusing-void-expression`.
    #[serde(
        rename = "type/no-confusing-void-expression",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_confusing_void_expression: Option<NoConfusingVoidExpressionOptions>,
    /// `type/no-unnecessary-condition`.
    #[serde(
        rename = "type/no-unnecessary-condition",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_unnecessary_condition: Option<NoUnnecessaryConditionOptions>,
    /// `type/restrict-plus-operands`.
    #[serde(
        rename = "type/restrict-plus-operands",
        skip_serializing_if = "Option::is_none"
    )]
    pub restrict_plus_operands: Option<RestrictPlusOperandsOptions>,
    /// `type/restrict-template-expressions`.
    #[serde(
        rename = "type/restrict-template-expressions",
        skip_serializing_if = "Option::is_none"
    )]
    pub restrict_template_expressions: Option<RestrictTemplateExpressionsOptions>,
    /// `type/return-await`.
    #[serde(rename = "type/return-await", skip_serializing_if = "Option::is_none")]
    pub return_await: Option<ReturnAwaitOption>,
}

impl TypeAwareRuleOptions {
    /// Whether no type-aware rule has options.
    pub fn is_empty(&self) -> bool {
        self.no_confusing_void_expression.is_none()
            && self.no_unnecessary_condition.is_none()
            && self.restrict_plus_operands.is_none()
            && self.restrict_template_expressions.is_none()
            && self.return_await.is_none()
    }

    /// Apply a later config layer.
    pub fn merge_from(&mut self, overlay: &Self) {
        if overlay.no_confusing_void_expression.is_some() {
            self.no_confusing_void_expression = overlay.no_confusing_void_expression;
        }
        if overlay.no_unnecessary_condition.is_some() {
            self.no_unnecessary_condition = overlay.no_unnecessary_condition;
        }
        if overlay.restrict_plus_operands.is_some() {
            self.restrict_plus_operands = overlay.restrict_plus_operands;
        }
        if overlay.restrict_template_expressions.is_some() {
            self.restrict_template_expressions = overlay.restrict_template_expressions.clone();
        }
        if overlay.return_await.is_some() {
            self.return_await = overlay.return_await;
        }
    }
}

impl super::ConfigLintRuleOptions {
    /// Options for the typescript-eslint type-aware rules.
    #[inline]
    pub fn type_aware(&self) -> &TypeAwareRuleOptions {
        &self.type_aware
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConstantLoopConditionMode, RestrictPlusOperandsOptions, ReturnAwaitOption,
        TypeAwareRuleOptions,
    };
    use crate::config::ConfigLintRuleOptions;

    #[test]
    fn reads_typescript_eslint_strict_type_checked_overrides() {
        let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
            "type/no-confusing-void-expression": { "ignoreArrowShorthand": true },
            "type/no-unnecessary-condition": {
                "allowConstantLoopConditions": "only-allowed-literals"
            },
            "type/restrict-plus-operands": {
                "allowAny": false,
                "allowBoolean": false,
                "allowNullish": false,
                "allowNumberAndString": true,
                "allowRegExp": false
            },
            "type/restrict-template-expressions": {
                "allowBoolean": true,
                "allowNullish": false,
                "allowNumber": true
            },
            "type/return-await": "error-handling-correctness-only"
        }))
        .expect("typescript-eslint option shapes parse");
        let typed = options.type_aware();
        assert_eq!(
            typed
                .no_confusing_void_expression
                .map(|value| value.ignore_arrow_shorthand),
            Some(true)
        );
        assert_eq!(
            typed
                .no_unnecessary_condition
                .map(|value| value.constant_loop_mode()),
            Some(ConstantLoopConditionMode::OnlyAllowedLiterals)
        );
        assert_eq!(
            typed.restrict_plus_operands,
            Some(RestrictPlusOperandsOptions {
                allow_any: false,
                allow_boolean: false,
                allow_nullish: false,
                allow_number_and_string: true,
                allow_regexp: false,
                skip_compound_assignments: false,
            })
        );
        let template = typed
            .restrict_template_expressions
            .clone()
            .expect("template options");
        assert!(!template.allow_nullish);
        assert!(
            template.allow_any,
            "omitted keys keep typescript-eslint defaults"
        );
        assert_eq!(template.allow.len(), 1, "default allow list is kept");
        assert_eq!(
            typed.return_await,
            Some(ReturnAwaitOption::ErrorHandlingCorrectnessOnly)
        );
        assert!(!options.is_empty());
    }

    #[test]
    fn boolean_loop_conditions_resolve_to_modes() {
        let typed: TypeAwareRuleOptions = serde_json::from_value(serde_json::json!({
            "type/no-unnecessary-condition": { "allowConstantLoopConditions": true }
        }))
        .expect("boolean form parses");
        assert_eq!(
            typed
                .no_unnecessary_condition
                .map(|value| value.constant_loop_mode()),
            Some(ConstantLoopConditionMode::Always)
        );
    }

    #[test]
    fn rejects_unknown_keys_and_modes() {
        for invalid in [
            serde_json::json!({ "type/restrict-plus-operands": { "allowStrings": true } }),
            serde_json::json!({ "type/return-await": "sometimes" }),
            serde_json::json!({
                "type/no-unnecessary-condition": { "allowConstantLoopConditions": "maybe" }
            }),
        ] {
            assert!(serde_json::from_value::<TypeAwareRuleOptions>(invalid).is_err());
        }
    }

    #[test]
    fn merges_later_layers_per_rule() {
        let mut base: TypeAwareRuleOptions = serde_json::from_value(serde_json::json!({
            "type/return-await": "always",
            "type/no-confusing-void-expression": { "ignoreVoidOperator": true }
        }))
        .expect("base");
        let overlay: TypeAwareRuleOptions = serde_json::from_value(serde_json::json!({
            "type/return-await": "never"
        }))
        .expect("overlay");
        base.merge_from(&overlay);
        assert_eq!(base.return_await, Some(ReturnAwaitOption::Never));
        assert!(base.no_confusing_void_expression.is_some());
    }
}
