use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;

#[derive(Clone, Copy)]
pub(crate) struct StaticClassPart<'a> {
    pub(crate) text: &'a str,
    pub(crate) span: Span,
    pub(crate) unconditional: bool,
    pub(crate) shared_group: Option<Span>,
}

/// Collect every statically spelled class name that can contribute to a
/// dynamic class binding. Conditional branches are included because the
/// duplicate-class rule checks each possible rendered branch.
pub(crate) fn possible_static_parts<'a>(
    expression: &'a Expression<'a>,
    parts: &mut Vec<StaticClassPart<'a>>,
) {
    collect_possible_static_parts(expression, true, None, parts);
}

fn collect_possible_static_parts<'a>(
    expression: &'a Expression<'a>,
    unconditional: bool,
    shared_group: Option<Span>,
    parts: &mut Vec<StaticClassPart<'a>>,
) {
    match expression.get_inner_expression() {
        Expression::StringLiteral(literal) => parts.push(StaticClassPart {
            text: literal.value.as_str(),
            span: literal.span,
            unconditional,
            shared_group,
        }),
        Expression::TemplateLiteral(template) => {
            let group = Some(template.span());
            for quasi in &template.quasis {
                parts.push(StaticClassPart {
                    text: quasi
                        .value
                        .cooked
                        .as_ref()
                        .map_or_else(|| quasi.value.raw.as_str(), |value| value.as_str()),
                    span: quasi.span,
                    unconditional,
                    shared_group: group,
                });
            }
            for expression in &template.expressions {
                collect_possible_static_parts(expression, unconditional, group, parts);
            }
        }
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                if let Some(expression) = element.as_expression() {
                    collect_possible_static_parts(expression, unconditional, None, parts);
                }
            }
        }
        Expression::ObjectExpression(object) => {
            for entry in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = entry else {
                    continue;
                };
                if let Some(mut part) = static_property_key(&property.key, property.computed) {
                    part.unconditional = false;
                    parts.push(part);
                }
            }
        }
        Expression::ConditionalExpression(conditional) => {
            collect_possible_static_parts(&conditional.consequent, false, None, parts);
            collect_possible_static_parts(&conditional.alternate, false, None, parts);
        }
        Expression::LogicalExpression(logical) => {
            collect_possible_static_parts(&logical.left, unconditional, None, parts);
            collect_possible_static_parts(&logical.right, false, None, parts);
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let group = Some(binary.span);
            collect_possible_static_parts(&binary.left, unconditional, group, parts);
            collect_possible_static_parts(&binary.right, unconditional, group, parts);
        }
        _ => {}
    }
}

/// Collect class fragments that are present regardless of runtime state and
/// can therefore move to a static `class` attribute.
pub(crate) fn unconditional_static_parts<'a>(
    expression: &'a Expression<'a>,
    parts: &mut Vec<StaticClassPart<'a>>,
) {
    match expression.get_inner_expression() {
        Expression::StringLiteral(literal) => parts.push(StaticClassPart {
            text: literal.value.as_str(),
            span: literal.span,
            unconditional: true,
            shared_group: None,
        }),
        Expression::TemplateLiteral(template) => {
            for quasi in &template.quasis {
                parts.push(StaticClassPart {
                    text: quasi
                        .value
                        .cooked
                        .as_ref()
                        .map_or_else(|| quasi.value.raw.as_str(), |value| value.as_str()),
                    span: quasi.span,
                    unconditional: true,
                    shared_group: None,
                });
            }
        }
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                if let Some(expression) = element.as_expression() {
                    unconditional_static_parts(expression, parts);
                }
            }
        }
        Expression::ObjectExpression(object) => {
            for entry in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = entry else {
                    continue;
                };
                let Expression::BooleanLiteral(value) = property.value.get_inner_expression()
                else {
                    continue;
                };
                if value.value
                    && let Some(mut part) = static_property_key(&property.key, property.computed)
                {
                    part.unconditional = true;
                    parts.push(part);
                }
            }
        }
        _ => {}
    }
}

fn static_property_key<'a>(
    key: &'a PropertyKey<'a>,
    computed: bool,
) -> Option<StaticClassPart<'a>> {
    if computed {
        return None;
    }
    let (text, span) = match key {
        PropertyKey::StaticIdentifier(identifier) => (identifier.name.as_str(), identifier.span),
        PropertyKey::StringLiteral(literal) => (literal.value.as_str(), literal.span),
        PropertyKey::TemplateLiteral(template) => {
            (template.single_quasi()?.as_str(), template.span())
        }
        _ => return None,
    };
    Some(StaticClassPart {
        text,
        span,
        unconditional: false,
        shared_group: None,
    })
}
