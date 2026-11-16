use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, VariableDeclarationKind};

use crate::macros::{EmitDefinition, PropDefinition};
use vize_carton::CompactString;

use super::super::{RuntimeObjectLiteral, ScriptParseResult};
use super::{emits, props};

pub(in crate::script_parser) fn record_static_runtime_object_literal(
    result: &mut ScriptParseResult,
    name: &str,
    expr: &Expression<'_>,
    kind: VariableDeclarationKind,
    source: &str,
) {
    super::with_defaults::record_object_binding(result, name, expr, kind, source);
    let Some(literal) = collect_runtime_object_expression(result, expr, source) else {
        return;
    };
    result
        .runtime_object_literals
        .insert(CompactString::new(name), literal);
}

pub(super) fn collect_runtime_object_expression(
    result: &ScriptParseResult,
    expression: &Expression<'_>,
    source: &str,
) -> Option<RuntimeObjectLiteral> {
    let mut literal = match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => {
            collect_runtime_object_literal(result, object, source)
        }
        Expression::Identifier(identifier) => result
            .runtime_object_literals
            .get(identifier.name.as_str())?
            .clone(),
        _ => return None,
    };
    let annotations = emits::extract_runtime_emit_type_annotations(expression, source);
    if has_runtime_type_assertion(expression) {
        for emit in &mut literal.emits {
            emit.payload_type = None;
        }
    }
    if !annotations.is_empty() {
        let names: vize_carton::FxHashSet<_> =
            literal.emits.iter().map(|emit| emit.name.clone()).collect();
        for name in names {
            let existing = literal
                .emit_validator_type_annotations
                .entry(name)
                .or_default();
            let mut inherited = annotations.clone();
            inherited.append(existing);
            *existing = inherited;
        }
    }
    Some(literal)
}

pub(super) fn has_runtime_type_assertion(mut expression: &Expression<'_>) -> bool {
    loop {
        expression = match expression {
            Expression::TSAsExpression(wrapper)
                if is_const_type_assertion(&wrapper.type_annotation) =>
            {
                &wrapper.expression
            }
            Expression::TSTypeAssertion(wrapper)
                if is_const_type_assertion(&wrapper.type_annotation) =>
            {
                &wrapper.expression
            }
            Expression::TSAsExpression(_) | Expression::TSTypeAssertion(_) => return true,
            Expression::TSSatisfiesExpression(wrapper) => &wrapper.expression,
            Expression::TSNonNullExpression(wrapper) => &wrapper.expression,
            Expression::ParenthesizedExpression(wrapper) => &wrapper.expression,
            _ => return false,
        };
    }
}

// `as const`/`<const>` preserve the literal's callable annotations. They
// cannot replace them with an asserted function or object type.
pub(super) use crate::script_parser::is_const_type_assertion;

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
                if property.kind == oxc_ast::ast::PropertyKind::Init
                    && let Some(signature) =
                        emits::extract_runtime_emit_signature(&property.value, source)
                {
                    literal
                        .emit_validator_signatures
                        .entry(CompactString::new(name))
                        .or_default()
                        .push(signature);
                }
                if property.kind == oxc_ast::ast::PropertyKind::Init {
                    let annotations =
                        emits::extract_runtime_emit_type_annotations(&property.value, source);
                    if !annotations.is_empty() {
                        literal
                            .emit_validator_type_annotations
                            .entry(CompactString::new(name))
                            .or_default()
                            .extend(annotations);
                    }
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                let Some(spread_literal) =
                    collect_runtime_object_expression(result, &spread.argument, source)
                else {
                    continue;
                };
                literal.props.extend(spread_literal.props.iter().cloned());
                literal.emits.extend(spread_literal.emits.iter().cloned());
                for (name, signatures) in &spread_literal.emit_validator_signatures {
                    literal
                        .emit_validator_signatures
                        .entry(name.clone())
                        .or_default()
                        .extend(signatures.iter().cloned());
                }
                for (name, annotations) in &spread_literal.emit_validator_type_annotations {
                    literal
                        .emit_validator_type_annotations
                        .entry(name.clone())
                        .or_default()
                        .extend(annotations.iter().cloned());
                }
            }
        }
    }

    literal
}
