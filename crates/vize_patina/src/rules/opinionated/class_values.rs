use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey, TemplateElement};
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
            for (index, quasi) in template.quasis.iter().enumerate() {
                let text = template_element_text(quasi);
                if let Some(text) = complete_template_segment(
                    text,
                    index == 0 || starts_with_ascii_whitespace(text),
                    index == template.expressions.len() || ends_with_ascii_whitespace(text),
                ) {
                    parts.push(StaticClassPart {
                        text,
                        span: quasi.span,
                        unconditional,
                        shared_group: group,
                    });
                }
            }
            for (index, expression) in template.expressions.iter().enumerate() {
                let left = template_element_text(&template.quasis[index]);
                let right = template_element_text(&template.quasis[index + 1]);
                let left_bounded =
                    (index == 0 && left.is_empty()) || ends_with_ascii_whitespace(left);
                let right_bounded = (index + 1 == template.quasis.len() - 1 && right.is_empty())
                    || starts_with_ascii_whitespace(right);
                if left_bounded && right_bounded {
                    collect_possible_static_parts(expression, unconditional, group, parts);
                }
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
            for (index, quasi) in template.quasis.iter().enumerate() {
                let text = template_element_text(quasi);
                if let Some(text) = complete_template_segment(
                    text,
                    index == 0 || starts_with_ascii_whitespace(text),
                    index == template.expressions.len() || ends_with_ascii_whitespace(text),
                ) {
                    parts.push(StaticClassPart {
                        text,
                        span: quasi.span,
                        unconditional: true,
                        shared_group: None,
                    });
                }
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

fn template_element_text<'a>(element: &'a TemplateElement<'a>) -> &'a str {
    element
        .value
        .cooked
        .as_ref()
        .map_or_else(|| element.value.raw.as_str(), |value| value.as_str())
}

fn starts_with_ascii_whitespace(text: &str) -> bool {
    text.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_whitespace())
}

fn ends_with_ascii_whitespace(text: &str) -> bool {
    text.as_bytes()
        .last()
        .is_some_and(|byte| byte.is_ascii_whitespace())
}

/// Return only the class tokens whose two boundaries are authored whitespace
/// (or the ends of the template literal). A fragment such as `badge--` in
/// `` `badge--${variant}` `` is not a class by itself and cannot be moved to a
/// static attribute or compared with another dynamic prefix.
fn complete_template_segment(text: &str, left_bounded: bool, right_bounded: bool) -> Option<&str> {
    let mut start = 0;
    let mut end = text.len();
    if !left_bounded {
        start = text
            .find(|character: char| character.is_ascii_whitespace())
            .map(|index| index + 1)?;
    }
    if !right_bounded {
        end = text[..end].rfind(|character: char| character.is_ascii_whitespace())?;
    }
    if start >= end {
        return None;
    }
    let segment = text[start..end].trim_matches(|character: char| character.is_ascii_whitespace());
    (!segment.is_empty()).then_some(segment)
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
