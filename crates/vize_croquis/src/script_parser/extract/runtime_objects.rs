use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};

use crate::macros::{EmitDefinition, PropDefinition};
use vize_carton::CompactString;

use super::super::{RuntimeObjectLiteral, ScriptParseResult};
use super::{emits, props};

pub(in crate::script_parser) fn record_static_runtime_object_literal(
    result: &mut ScriptParseResult,
    name: &str,
    expr: &Expression<'_>,
    source: &str,
) {
    let Some(object) = unwrap_runtime_object_expression(expr) else {
        return;
    };

    let literal = collect_runtime_object_literal(result, object, source);
    result
        .runtime_object_literals
        .insert(CompactString::new(name), literal);
}

fn collect_runtime_object_literal(
    result: &ScriptParseResult,
    object: &ObjectExpression<'_>,
    source: &str,
) -> RuntimeObjectLiteral {
    let mut literal = RuntimeObjectLiteral::default();

    for property in object.properties.iter() {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                let Some(name) = props::runtime_object_property_name(&property.key) else {
                    continue;
                };
                literal.props.push(PropDefinition {
                    name: CompactString::new(name),
                    required: props::detect_required_prop(&property.value),
                    prop_type: props::extract_runtime_prop_type(&property.value, source),
                    default_value: props::extract_runtime_prop_default(&property.value, source),
                });
                literal.emits.push(EmitDefinition {
                    name: CompactString::new(name),
                    payload_type: emits::extract_runtime_emit_payload_type(&property.value, source),
                });
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                let Expression::Identifier(identifier) = &spread.argument else {
                    continue;
                };
                let Some(spread_literal) =
                    result.runtime_object_literals.get(identifier.name.as_str())
                else {
                    continue;
                };
                literal.props.extend(spread_literal.props.iter().cloned());
                literal.emits.extend(spread_literal.emits.iter().cloned());
            }
        }
    }

    literal
}

fn unwrap_runtime_object_expression<'a>(
    expr: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expr {
        Expression::ObjectExpression(object) => Some(object),
        Expression::TSAsExpression(ts_as) => unwrap_runtime_object_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            unwrap_runtime_object_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            unwrap_runtime_object_expression(&ts_non_null.expression)
        }
        Expression::ParenthesizedExpression(paren) => {
            unwrap_runtime_object_expression(&paren.expression)
        }
        _ => None,
    }
}
