//! Same-file Options API mixin and extends resolution.

use oxc_ast::ast::{ArrayExpression, Expression, ObjectExpression};
use vize_carton::{FxHashMap, FxHashSet};

use super::{
    ScriptParseResult, collect_options_object_template_bindings, option_expression_property,
};

/// Merges template bindings contributed by same-file `mixins` entries.
///
/// Only same-file targets are resolved: inline object literals and
/// identifiers whose `const` initializer is an object literal in this module.
/// Imported mixins are deliberately ignored because they require cross-file analysis.
pub(super) fn collect_mixins_bindings<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    seen_mixins: &mut FxHashSet<&'a str>,
    legacy_vue2: bool,
) {
    let Some(expression) = option_expression_property(options, "mixins") else {
        return;
    };
    let Some(array) = unwrap_array_expression(expression) else {
        return;
    };

    for element in &array.elements {
        let Some(expression) = element.as_expression() else {
            continue;
        };
        collect_mixin_target_bindings(
            result,
            expression,
            object_bindings,
            seen_mixins,
            legacy_vue2,
        );
    }
}

fn unwrap_array_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ArrayExpression<'a>> {
    match expression {
        Expression::ArrayExpression(array) => Some(array),
        Expression::ParenthesizedExpression(parenthesized) => {
            unwrap_array_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => unwrap_array_expression(&ts_as.expression),
        Expression::TSTypeAssertion(ts_assertion) => {
            unwrap_array_expression(&ts_assertion.expression)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            unwrap_array_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            unwrap_array_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

/// Merges bindings from a same-file `extends` target using the mixin rules.
pub(super) fn collect_extends_bindings<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    seen_mixins: &mut FxHashSet<&'a str>,
    legacy_vue2: bool,
) {
    let Some(expression) = option_expression_property(options, "extends") else {
        return;
    };
    collect_mixin_target_bindings(
        result,
        expression,
        object_bindings,
        seen_mixins,
        legacy_vue2,
    );
}

/// Resolves one mixin/extends target and guards recursive identifier cycles.
fn collect_mixin_target_bindings<'a>(
    result: &mut ScriptParseResult,
    expression: &'a Expression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    seen_mixins: &mut FxHashSet<&'a str>,
    legacy_vue2: bool,
) {
    match expression {
        Expression::ObjectExpression(object) => collect_options_object_template_bindings(
            result,
            object,
            object_bindings,
            seen_mixins,
            legacy_vue2,
        ),
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if !seen_mixins.insert(name) {
                return;
            }
            if let Some(object) = object_bindings.get(name).copied() {
                collect_options_object_template_bindings(
                    result,
                    object,
                    object_bindings,
                    seen_mixins,
                    legacy_vue2,
                );
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => collect_mixin_target_bindings(
            result,
            &parenthesized.expression,
            object_bindings,
            seen_mixins,
            legacy_vue2,
        ),
        Expression::TSAsExpression(ts_as) => collect_mixin_target_bindings(
            result,
            &ts_as.expression,
            object_bindings,
            seen_mixins,
            legacy_vue2,
        ),
        Expression::TSSatisfiesExpression(ts_satisfies) => collect_mixin_target_bindings(
            result,
            &ts_satisfies.expression,
            object_bindings,
            seen_mixins,
            legacy_vue2,
        ),
        Expression::TSNonNullExpression(ts_non_null) => collect_mixin_target_bindings(
            result,
            &ts_non_null.expression,
            object_bindings,
            seen_mixins,
            legacy_vue2,
        ),
        // Imported mixins, call expressions, etc. — deferred.
        _ => {}
    }
}
