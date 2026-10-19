//! nuxt/no-nuxt-config-test-key
//!
//! Nuxt detects its test environment automatically. This reproduces
//! `@nuxt/eslint-plugin` 1.16.0's deliberately narrow syntax contract: inspect
//! only a default-exported object (or the first object argument of any
//! default-exported call), and reject direct identifier keys named `test` when
//! their value is a boolean literal.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "nuxt/no-nuxt-config-test-key",
    description: "Disallow setting `test` key in Nuxt config",
    default_severity: Severity::Error,
};

const MESSAGE: &str =
    "Do not set `test` key in Nuxt config. The test environment is automatically detected.";

/// Disallow a boolean `test` key in a directly exported Nuxt config object.
pub struct NoNuxtConfigTestKey;

impl ScriptRule for NoNuxtConfigTestKey {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        for statement in &program.body {
            let Statement::ExportDefaultDeclaration(export) = statement else {
                continue;
            };
            let Some(object) = exported_object(&export.declaration) else {
                continue;
            };
            report_boolean_test_properties(object, offset, result);
        }
    }
}

fn exported_object<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => first_object_argument(call),
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            object_from_expression(&parenthesized.expression)
        }
        _ => None,
    }
}

fn object_from_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => first_object_argument(call),
        Expression::ParenthesizedExpression(parenthesized) => {
            object_from_expression(&parenthesized.expression)
        }
        _ => None,
    }
}

fn first_object_argument<'a>(call: &'a CallExpression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object),
        Argument::ParenthesizedExpression(parenthesized) => {
            object_from_expression(&parenthesized.expression)
        }
        _ => None,
    }
}

fn report_boolean_test_properties(
    object: &ObjectExpression<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    for entry in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = entry else {
            continue;
        };
        if identifier_key_name(&property.key) != Some("test") || !is_boolean(&property.value) {
            continue;
        }
        result.add_diagnostic(LintDiagnostic::error(
            META.name,
            MESSAGE,
            offset as u32 + property.span.start,
            offset as u32 + property.span.end,
        ));
    }
}

fn identifier_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(key) => Some(key.name.as_str()),
        PropertyKey::Identifier(key) => Some(key.name.as_str()),
        PropertyKey::ParenthesizedExpression(parenthesized) => {
            identifier_expression_name(&parenthesized.expression)
        }
        _ => None,
    }
}

fn identifier_expression_name<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::ParenthesizedExpression(parenthesized) => {
            identifier_expression_name(&parenthesized.expression)
        }
        _ => None,
    }
}

fn is_boolean(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::BooleanLiteral(_) => true,
        Expression::ParenthesizedExpression(parenthesized) => is_boolean(&parenthesized.expression),
        _ => false,
    }
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests {
    use super::{META, NoNuxtConfigTestKey};
    use crate::diagnostic::Severity;
    use crate::rules::script::{ScriptLintResult, ScriptLinter, ScriptRule};
    use serde_json::Value;

    const CORPUS: &str = include_str!(
        "../../../../../npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/fixtures/corpus.json"
    );
    const RECORDING: &str = include_str!(
        "../../../../../npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/fixtures/nuxt-eslint-output.json"
    );

    fn lint(source: &str) -> ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoNuxtConfigTestKey));
        linter.lint(source, 0)
    }

    fn line_column(source: &str, offset: usize) -> (u64, u64) {
        let prefix = &source[..offset];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u64 + 1;
        let column = prefix
            .rsplit_once('\n')
            .map_or(prefix, |(_, tail)| tail)
            .len() as u64
            + 1;
        (line, column)
    }

    #[test]
    fn metadata_matches_the_nuxt_rule_contract() {
        let meta = NoNuxtConfigTestKey.meta();
        assert_eq!(meta.name, "nuxt/no-nuxt-config-test-key");
        assert_eq!(
            meta.description,
            "Disallow setting `test` key in Nuxt config"
        );
        assert_eq!(meta.default_severity, Severity::Error);
    }

    #[test]
    fn exact_diagnostic_contract_is_non_fixable() {
        let source = "export default defineNuxtConfig({ test: true })";
        let result = lint(source);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.warning_count, 0);

        let diagnostic = &result.diagnostics[0];
        assert_eq!(diagnostic.rule_name, META.name);
        assert_eq!(diagnostic.severity, Severity::Error);
        assert_eq!(
            diagnostic.message,
            "Do not set `test` key in Nuxt config. The test environment is automatically detected."
        );
        assert_eq!(
            &source[diagnostic.start as usize..diagnostic.end as usize],
            "test: true"
        );
        assert!(diagnostic.help.is_none());
        assert!(diagnostic.fix.is_none());
    }

    #[test]
    fn diagnostic_offsets_include_the_sfc_block_offset() {
        let source = "export default { test: false }";
        let mut result = ScriptLintResult::default();
        NoNuxtConfigTestKey.check(source, 41, &mut result);
        let property_start = source.find("test").unwrap() as u32;
        assert_eq!(result.diagnostics[0].start, 41 + property_start);
        assert_eq!(
            result.diagnostics[0].end,
            41 + source.find('}').unwrap() as u32 - 1
        );
    }

    #[test]
    fn matches_the_recorded_nuxt_eslint_plugin_oracle() {
        let corpus: Value = serde_json::from_str(CORPUS).unwrap();
        let recording: Value = serde_json::from_str(RECORDING).unwrap();
        let cases = corpus["noNuxtConfigTestKeyCases"].as_array().unwrap();
        let recorded = recording["noNuxtConfigTestKeyCases"].as_object().unwrap();

        for case in cases {
            let id = case["id"].as_str().unwrap();
            let source = case["source"].as_str().unwrap();
            let upstream = &recorded[id];
            let expected_messages = upstream["messages"].as_array().unwrap();
            let result = lint(source);

            assert_eq!(result.diagnostics.len(), expected_messages.len(), "{id}");
            for (diagnostic, expected) in result.diagnostics.iter().zip(expected_messages) {
                assert_eq!(
                    diagnostic.rule_name,
                    expected["ruleId"].as_str().unwrap(),
                    "{id}"
                );
                assert_eq!(diagnostic.severity, Severity::Error, "{id}");
                assert_eq!(
                    diagnostic.message,
                    expected["message"].as_str().unwrap(),
                    "{id}"
                );
                assert!(diagnostic.fix.is_none(), "{id}");

                let range = expected["range"].as_array().unwrap();
                let start = range[0].as_u64().unwrap();
                let end = range[1].as_u64().unwrap();
                assert_eq!(
                    (u64::from(diagnostic.start), u64::from(diagnostic.end)),
                    (start, end),
                    "diagnostic range for {id}"
                );
                assert_eq!(
                    line_column(source, start as usize),
                    (
                        expected["line"].as_u64().unwrap(),
                        expected["column"].as_u64().unwrap()
                    ),
                    "diagnostic start location for {id}"
                );
                assert_eq!(
                    line_column(source, end as usize),
                    (
                        expected["endLine"].as_u64().unwrap(),
                        expected["endColumn"].as_u64().unwrap()
                    ),
                    "diagnostic end location for {id}"
                );
            }

            assert_eq!(upstream["fixed"], false, "{id}");
            assert_eq!(upstream["output"], source, "{id}");
            assert_eq!(upstream["secondPassFixed"], false, "{id}");
            assert_eq!(upstream["secondPassOutput"], source, "{id}");
            assert_eq!(
                upstream["secondPassMessageCount"],
                expected_messages.len(),
                "{id}"
            );
            assert_eq!(upstream["secondPassMessagesMatch"], true, "{id}");
        }
    }

    #[test]
    fn uses_utf8_byte_offsets() {
        let source = "const cafe = 'é'\nexport default { test: true }";
        let result = lint(source);
        let start = source.find("test").unwrap() as u32;
        assert_eq!(result.diagnostics[0].start, start);
        assert_eq!(
            &source[result.diagnostics[0].start as usize..result.diagnostics[0].end as usize],
            "test: true"
        );
    }
}
