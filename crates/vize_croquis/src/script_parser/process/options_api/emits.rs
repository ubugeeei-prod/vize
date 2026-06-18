use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, Expression, ObjectExpression, ObjectPropertyKind,
};
use vize_carton::{CompactString, FxHashMap, FxHashSet};

use crate::macros::EmitDefinition;
use crate::script_parser::{ScriptParseResult, extract::extract_runtime_emit_payload_type};

use super::{option_expression_property, property_key_name};

pub(super) fn collect_options_api_emits_from_options<'a>(
    result: &mut ScriptParseResult,
    options: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    source: &str,
) {
    let Some(emits) = option_expression_property(options, "emits") else {
        return;
    };
    collect_from_expression(
        result,
        emits,
        object_bindings,
        source,
        &mut FxHashSet::default(),
    );
}

fn collect_from_expression<'a>(
    result: &mut ScriptParseResult,
    expression: &'a Expression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    source: &str,
    seen_bindings: &mut FxHashSet<&'a str>,
) {
    match expression {
        Expression::ArrayExpression(array) => collect_from_array(result, array),
        Expression::ObjectExpression(object) => {
            collect_from_object(result, object, object_bindings, source, seen_bindings);
        }
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if !seen_bindings.insert(name) {
                return;
            }
            let Some(object) = object_bindings.get(name) else {
                return;
            };
            collect_from_object(result, object, object_bindings, source, seen_bindings);
        }
        Expression::TSAsExpression(ts_as) => collect_from_expression(
            result,
            &ts_as.expression,
            object_bindings,
            source,
            seen_bindings,
        ),
        Expression::TSSatisfiesExpression(ts_satisfies) => collect_from_expression(
            result,
            &ts_satisfies.expression,
            object_bindings,
            source,
            seen_bindings,
        ),
        Expression::TSNonNullExpression(ts_non_null) => collect_from_expression(
            result,
            &ts_non_null.expression,
            object_bindings,
            source,
            seen_bindings,
        ),
        Expression::ParenthesizedExpression(parenthesized) => collect_from_expression(
            result,
            &parenthesized.expression,
            object_bindings,
            source,
            seen_bindings,
        ),
        _ => {}
    }
}

fn collect_from_array(result: &mut ScriptParseResult, array: &ArrayExpression<'_>) {
    for element in &array.elements {
        let ArrayExpressionElement::StringLiteral(literal) = element else {
            continue;
        };
        result.macros.add_emit(EmitDefinition {
            name: CompactString::new(literal.value.as_str()),
            payload_type: None,
        });
    }
}

fn collect_from_object<'a>(
    result: &mut ScriptParseResult,
    object: &'a ObjectExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
    source: &str,
    seen_bindings: &mut FxHashSet<&'a str>,
) {
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                let Some(name) = property_key_name(&property.key) else {
                    continue;
                };
                result.macros.add_emit(EmitDefinition {
                    name: CompactString::new(name),
                    payload_type: extract_runtime_emit_payload_type(&property.value, source),
                });
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                let Expression::Identifier(identifier) = &spread.argument else {
                    continue;
                };
                collect_from_expression(
                    result,
                    &spread.argument,
                    object_bindings,
                    source,
                    seen_bindings,
                );
                seen_bindings.remove(identifier.name.as_str());
            }
        }
    }
}
