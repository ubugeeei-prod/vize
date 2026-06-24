//! Convert a JSX element/fragment subtree into a Vue template string.
//!
//! The conversion is intentionally conservative: anything that cannot be mapped
//! confidently returns `None` so the caller emits a manual-port TODO instead of
//! silently-wrong markup. Expression source is recovered by slicing the original
//! file by oxc byte spans.

use super::text::escape_attr;
use oxc_ast::ast::{
    Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement,
    JSXElementName, JSXExpression, JSXFragment, JSXMemberExpression, JSXMemberExpressionObject,
};
use vize_carton::{String, append};

/// Convert a JSX expression (element or fragment) to Vue template markup.
///
/// `source` is the full original file text; spans index into it.
pub(super) fn convert_render(expr: &Expression<'_>, source: &str) -> Option<String> {
    match expr {
        Expression::JSXElement(element) => convert_element(element, source),
        Expression::JSXFragment(fragment) => convert_fragment(fragment, source),
        _ => None,
    }
}

/// A fragment `<>...</>` emits only its children (no wrapper element).
fn convert_fragment(fragment: &JSXFragment<'_>, source: &str) -> Option<String> {
    convert_children(&fragment.children, source)
}

fn convert_element(element: &JSXElement<'_>, source: &str) -> Option<String> {
    let tag = element_name(&element.opening_element.name)?;

    let mut attributes = String::default();
    for item in &element.opening_element.attributes {
        attributes.push(' ');
        attributes.push_str(&convert_attribute(item, source)?);
    }

    let children = convert_children(&element.children, source)?;

    let mut out = String::default();
    if children.is_empty() {
        append!(out, "<{tag}{attributes} />");
    } else {
        append!(out, "<{tag}{attributes}>{children}</{tag}>");
    }
    Some(out)
}

fn convert_children(children: &[JSXChild<'_>], source: &str) -> Option<String> {
    let mut out = String::default();
    for child in children {
        match child {
            JSXChild::Text(text) => {
                let trimmed = text.value.as_str().trim();
                if !trimmed.is_empty() {
                    out.push_str(trimmed);
                }
            }
            JSXChild::Element(element) => out.push_str(&convert_element(element, source)?),
            JSXChild::Fragment(fragment) => out.push_str(&convert_fragment(fragment, source)?),
            JSXChild::ExpressionContainer(container) => match &container.expression {
                JSXExpression::EmptyExpression(_) => {}
                expression => {
                    let text = jsx_expression_source(expression, source)?;
                    append!(out, "{{{{ {text} }}}}");
                }
            },
            // `{...spread}` children and anything unexpected: bail out.
            JSXChild::Spread(_) => return None,
        }
    }
    Some(out)
}

fn convert_attribute(item: &JSXAttributeItem<'_>, source: &str) -> Option<String> {
    match item {
        JSXAttributeItem::Attribute(attribute) => {
            let name = attribute_name(&attribute.name)?;
            match attribute.value.as_ref() {
                None => Some(name.into()),
                Some(JSXAttributeValue::StringLiteral(literal)) => {
                    let mut out = String::default();
                    append!(out, "{name}=\"{}\"", escape_attr(literal.value.as_str()));
                    Some(out)
                }
                Some(JSXAttributeValue::ExpressionContainer(container)) => {
                    match &container.expression {
                        JSXExpression::EmptyExpression(_) => None,
                        expression => {
                            let text = jsx_expression_source(expression, source)?;
                            let mut out = String::default();
                            append!(out, ":{name}=\"{}\"", escape_attr(text.as_str()));
                            Some(out)
                        }
                    }
                }
                // `prop=<Element/>` and `prop=<></>` are not convertible.
                Some(_) => None,
            }
        }
        JSXAttributeItem::SpreadAttribute(spread) => {
            let text = expression_source(&spread.argument, source)?;
            let mut out = String::default();
            append!(out, "v-bind=\"{}\"", escape_attr(text.as_str()));
            Some(out)
        }
    }
}

/// Element tag name. Supports identifiers and dotted member names.
fn element_name(name: &JSXElementName<'_>) -> Option<String> {
    match name {
        JSXElementName::Identifier(ident) => Some(ident.name.as_str().into()),
        JSXElementName::IdentifierReference(ident) => Some(ident.name.as_str().into()),
        JSXElementName::MemberExpression(member) => member_name(member),
        JSXElementName::NamespacedName(_) | JSXElementName::ThisExpression(_) => None,
    }
}

fn member_name(member: &JSXMemberExpression<'_>) -> Option<String> {
    let mut out = match &member.object {
        JSXMemberExpressionObject::IdentifierReference(ident) => String::from(ident.name.as_str()),
        JSXMemberExpressionObject::MemberExpression(inner) => member_name(inner)?,
        JSXMemberExpressionObject::ThisExpression(_) => return None,
    };
    out.push('.');
    out.push_str(member.property.name.as_str());
    Some(out)
}

fn attribute_name<'a>(name: &'a JSXAttributeName<'a>) -> Option<&'a str> {
    match name {
        JSXAttributeName::Identifier(ident) => Some(ident.name.as_str()),
        // Namespaced attribute names (`foo:bar`) have no clean Vue mapping.
        JSXAttributeName::NamespacedName(_) => None,
    }
}

/// Source text of a `JSXExpression` (the expression inside `{...}`).
fn jsx_expression_source(expression: &JSXExpression<'_>, source: &str) -> Option<String> {
    use oxc_span::GetSpan;
    let span = expression.span();
    Some(span.source_text(source).into())
}

/// Source text of an `Expression`.
fn expression_source(expression: &Expression<'_>, source: &str) -> Option<String> {
    use oxc_span::GetSpan;
    let span = expression.span();
    Some(span.source_text(source).into())
}

#[cfg(test)]
mod tests;
