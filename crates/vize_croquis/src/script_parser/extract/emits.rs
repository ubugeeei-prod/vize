use oxc_ast::ast::{
    Argument, Expression, FormalParameters, ObjectPropertyKind, PropertyKey, TSType,
};
use oxc_span::{GetSpan, Span};

use crate::macros::EmitDefinition;
use vize_carton::{CompactString, String};

use super::super::ScriptParseResult;

pub fn extract_emits_from_type(
    result: &mut ScriptParseResult,
    type_params: &oxc_allocator::Vec<'_, TSType<'_>>,
    _source: &str,
) {
    for tp in type_params.iter() {
        if let TSType::TSTypeLiteral(lit) = tp {
            // Handle call signatures like { (e: 'update', value: string): void }
            for member in lit.members.iter() {
                if let oxc_ast::ast::TSSignature::TSCallSignatureDeclaration(call_sig) = member {
                    // First parameter is usually the event name: (e: 'eventName', ...)
                    if let Some(first_param) = call_sig.params.items.first()
                        && let Some(type_ann) = &first_param.type_annotation
                        && let TSType::TSLiteralType(lit_type) = &type_ann.type_annotation
                        && let oxc_ast::ast::TSLiteral::StringLiteral(s) = &lit_type.literal
                    {
                        result.macros.add_emit(EmitDefinition {
                            name: CompactString::new(s.value.as_str()),
                            payload_type: None,
                        });
                    }
                }
            }
        }
    }
}

/// Extract emits from runtime arguments (array)
pub fn extract_emits_from_runtime(
    result: &mut ScriptParseResult,
    arg: &Argument<'_>,
    source: &str,
) {
    match arg {
        Argument::ArrayExpression(arr) => extract_emits_from_array(result, arr),
        Argument::ObjectExpression(obj) => extract_emits_from_object(result, obj, source),
        Argument::TSAsExpression(ts_as) => {
            extract_emits_from_runtime_expression(result, &ts_as.expression, source);
        }
        Argument::TSSatisfiesExpression(ts_satisfies) => {
            extract_emits_from_runtime_expression(result, &ts_satisfies.expression, source);
        }
        Argument::TSNonNullExpression(ts_non_null) => {
            extract_emits_from_runtime_expression(result, &ts_non_null.expression, source);
        }
        Argument::ParenthesizedExpression(paren) => {
            extract_emits_from_runtime_expression(result, &paren.expression, source);
        }
        _ => {}
    }
}

fn extract_emits_from_runtime_expression(
    result: &mut ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) {
    match expr {
        Expression::ArrayExpression(arr) => extract_emits_from_array(result, arr),
        Expression::ObjectExpression(obj) => extract_emits_from_object(result, obj, source),
        Expression::TSAsExpression(ts_as) => {
            extract_emits_from_runtime_expression(result, &ts_as.expression, source);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_emits_from_runtime_expression(result, &ts_satisfies.expression, source);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_emits_from_runtime_expression(result, &ts_non_null.expression, source);
        }
        Expression::ParenthesizedExpression(paren) => {
            extract_emits_from_runtime_expression(result, &paren.expression, source);
        }
        _ => {}
    }
}

fn extract_emits_from_array(
    result: &mut ScriptParseResult,
    arr: &oxc_ast::ast::ArrayExpression<'_>,
) {
    for elem in arr.elements.iter() {
        if let oxc_ast::ast::ArrayExpressionElement::StringLiteral(s) = elem {
            result.macros.add_emit(EmitDefinition {
                name: CompactString::new(s.value.as_str()),
                payload_type: None,
            });
        }
    }
}

fn extract_emits_from_object(
    result: &mut ScriptParseResult,
    obj: &oxc_ast::ast::ObjectExpression<'_>,
    source: &str,
) {
    for prop in obj.properties.iter() {
        match prop {
            ObjectPropertyKind::ObjectProperty(prop) => {
                let name = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                    PropertyKey::StringLiteral(s) => s.value.as_str(),
                    _ => continue,
                };

                result.macros.add_emit(EmitDefinition {
                    name: CompactString::new(name),
                    payload_type: extract_runtime_emit_payload_type(&prop.value, source),
                });
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                let Expression::Identifier(identifier) = &spread.argument else {
                    continue;
                };
                let Some(emits) = result
                    .runtime_object_literals
                    .get(identifier.name.as_str())
                    .map(|literal| literal.emits.clone())
                else {
                    continue;
                };
                for emit in emits {
                    result.macros.add_emit(emit);
                }
            }
        }
    }
}

pub(super) fn extract_runtime_emit_payload_type(
    value: &Expression<'_>,
    source: &str,
) -> Option<CompactString> {
    match value {
        Expression::ArrowFunctionExpression(func) => {
            extract_emit_payload_tuple(&func.params, source)
        }
        Expression::FunctionExpression(func) => extract_emit_payload_tuple(&func.params, source),
        Expression::TSAsExpression(ts_as) => {
            extract_runtime_emit_payload_type(&ts_as.expression, source)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_runtime_emit_payload_type(&ts_satisfies.expression, source)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_runtime_emit_payload_type(&ts_non_null.expression, source)
        }
        Expression::ParenthesizedExpression(paren) => {
            extract_runtime_emit_payload_type(&paren.expression, source)
        }
        _ => None,
    }
}

fn extract_emit_payload_tuple(
    params: &FormalParameters<'_>,
    source: &str,
) -> Option<CompactString> {
    let mut payload = String::from("[");
    let mut first = true;

    for param in params.items.iter() {
        let type_annotation = param.type_annotation.as_ref()?;
        let ty = type_annotation_source(source, type_annotation.span)?;

        if !first {
            payload.push_str(", ");
        }
        first = false;

        if let Some(label) = simple_parameter_label(source, param.pattern.span()) {
            payload.push_str(label.as_str());
            if param.optional {
                payload.push('?');
            }
            payload.push_str(": ");
        }
        payload.push_str(ty);
    }

    if let Some(rest) = params.rest.as_ref() {
        let type_annotation = rest.type_annotation.as_ref()?;
        let ty = type_annotation_source(source, type_annotation.span)?;

        if !first {
            payload.push_str(", ");
        }

        if let Some(label) = simple_parameter_label(source, rest.rest.argument.span()) {
            payload.push_str("...");
            payload.push_str(label.as_str());
            payload.push_str(": ");
        } else {
            payload.push_str("...");
        }
        payload.push_str(ty);
    }

    payload.push(']');
    Some(CompactString::new(payload.as_str()))
}

fn type_annotation_source(source: &str, span: Span) -> Option<&str> {
    let ty = source
        .get(span.start as usize..span.end as usize)?
        .trim()
        .trim_start_matches(':')
        .trim();
    (!ty.is_empty()).then_some(ty)
}

fn simple_parameter_label(source: &str, span: Span) -> Option<CompactString> {
    let label = source.get(span.start as usize..span.end as usize)?.trim();
    let mut chars = label.chars();
    let first = chars.next()?;
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return None;
    }
    if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$') {
        return None;
    }
    Some(CompactString::new(label))
}
