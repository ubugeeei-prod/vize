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
use vize_l0::{String, append};

/// Convert a JSX expression (element or fragment) to Vue template markup.
///
/// `source` is the full original file text; spans index into it.
pub(super) fn convert_render(
    expr: &Expression<'_>,
    source: &str,
    render_args_name: Option<&str>,
) -> Option<String> {
    match expr {
        Expression::JSXElement(element) => convert_element(element, source, render_args_name),
        Expression::JSXFragment(fragment) => convert_fragment(fragment, source, render_args_name),
        _ => None,
    }
}

/// A fragment `<>...</>` emits only its children (no wrapper element).
fn convert_fragment(
    fragment: &JSXFragment<'_>,
    source: &str,
    render_args_name: Option<&str>,
) -> Option<String> {
    convert_children(&fragment.children, source, render_args_name)
}

fn convert_element(
    element: &JSXElement<'_>,
    source: &str,
    render_args_name: Option<&str>,
) -> Option<String> {
    let tag = element_name(&element.opening_element.name)?;

    let mut attributes = String::default();
    for item in &element.opening_element.attributes {
        attributes.push(' ');
        attributes.push_str(&convert_attribute(item, source, render_args_name)?);
    }

    let children = convert_children(&element.children, source, render_args_name)?;

    let mut out = String::default();
    if children.is_empty() {
        append!(out, "<{tag}{attributes} />");
    } else {
        append!(out, "<{tag}{attributes}>{children}</{tag}>");
    }
    Some(out)
}

fn convert_children(
    children: &[JSXChild<'_>],
    source: &str,
    render_args_name: Option<&str>,
) -> Option<String> {
    let mut out = String::default();
    for child in children {
        match child {
            JSXChild::Text(text) => {
                let trimmed = text.value.as_str().trim();
                if !trimmed.is_empty() {
                    out.push_str(trimmed);
                }
            }
            JSXChild::Element(element) => {
                out.push_str(&convert_element(element, source, render_args_name)?);
            }
            JSXChild::Fragment(fragment) => {
                out.push_str(&convert_fragment(fragment, source, render_args_name)?);
            }
            JSXChild::ExpressionContainer(container) => match &container.expression {
                JSXExpression::EmptyExpression(_) => {}
                expression => {
                    if child_expression_requires_binding(expression) {
                        return None;
                    }
                    let text = jsx_expression_source(expression, source)?;
                    if references_render_args(text.as_str(), render_args_name) {
                        return None;
                    }
                    append!(out, "{{{{ {text} }}}}");
                }
            },
            // `{...spread}` children and anything unexpected: bail out.
            JSXChild::Spread(_) => return None,
        }
    }
    Some(out)
}

fn convert_attribute(
    item: &JSXAttributeItem<'_>,
    source: &str,
    render_args_name: Option<&str>,
) -> Option<String> {
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
                            if references_render_args(text.as_str(), render_args_name)
                                || attribute_expression_requires_binding(expression)
                            {
                                return None;
                            }
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
            if !is_render_args_spread(text.as_str(), render_args_name) {
                return None;
            }
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

fn child_expression_requires_binding(expression: &JSXExpression<'_>) -> bool {
    match expression {
        JSXExpression::ObjectExpression(_)
        | JSXExpression::ArrowFunctionExpression(_)
        | JSXExpression::FunctionExpression(_)
        | JSXExpression::JSXElement(_)
        | JSXExpression::JSXFragment(_) => true,
        JSXExpression::ParenthesizedExpression(inner) => {
            child_value_requires_binding(&inner.expression)
        }
        JSXExpression::TSAsExpression(inner) => child_value_requires_binding(&inner.expression),
        JSXExpression::TSSatisfiesExpression(inner) => {
            child_value_requires_binding(&inner.expression)
        }
        JSXExpression::TSNonNullExpression(inner) => {
            child_value_requires_binding(&inner.expression)
        }
        _ => false,
    }
}

fn child_value_requires_binding(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ObjectExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_) => true,
        Expression::ParenthesizedExpression(inner) => {
            child_value_requires_binding(&inner.expression)
        }
        Expression::TSAsExpression(inner) => child_value_requires_binding(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => child_value_requires_binding(&inner.expression),
        Expression::TSNonNullExpression(inner) => child_value_requires_binding(&inner.expression),
        _ => false,
    }
}

fn attribute_expression_requires_binding(expression: &JSXExpression<'_>) -> bool {
    match expression {
        JSXExpression::Identifier(_)
        | JSXExpression::StaticMemberExpression(_)
        | JSXExpression::ComputedMemberExpression(_)
        | JSXExpression::PrivateFieldExpression(_)
        | JSXExpression::CallExpression(_)
        | JSXExpression::ChainExpression(_)
        | JSXExpression::NewExpression(_)
        | JSXExpression::ThisExpression(_) => true,
        JSXExpression::ParenthesizedExpression(inner) => {
            attribute_value_requires_binding(&inner.expression)
        }
        JSXExpression::TSAsExpression(inner) => attribute_value_requires_binding(&inner.expression),
        JSXExpression::TSSatisfiesExpression(inner) => {
            attribute_value_requires_binding(&inner.expression)
        }
        JSXExpression::TSNonNullExpression(inner) => {
            attribute_value_requires_binding(&inner.expression)
        }
        _ => false,
    }
}

fn attribute_value_requires_binding(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::PrivateFieldExpression(_)
        | Expression::CallExpression(_)
        | Expression::ChainExpression(_)
        | Expression::NewExpression(_)
        | Expression::ThisExpression(_) => true,
        Expression::ParenthesizedExpression(inner) => {
            attribute_value_requires_binding(&inner.expression)
        }
        Expression::TSAsExpression(inner) => attribute_value_requires_binding(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => {
            attribute_value_requires_binding(&inner.expression)
        }
        Expression::TSNonNullExpression(inner) => {
            attribute_value_requires_binding(&inner.expression)
        }
        _ => false,
    }
}

fn references_render_args(text: &str, render_args_name: Option<&str>) -> bool {
    let Some(args_name) = render_args_name else {
        return false;
    };
    let text = text.trim();
    text == args_name
        || text.strip_prefix(args_name).is_some_and(|rest| {
            rest.starts_with('.') || rest.starts_with("?.") || rest.starts_with('[')
        })
}

fn is_render_args_spread(text: &str, render_args_name: Option<&str>) -> bool {
    let Some(args_name) = render_args_name else {
        return false;
    };
    text.trim() == args_name
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
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::{Expression, Statement};
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use vize_l0::String;

    use super::convert_render;

    /// Wrap a JSX expression in `const x = (<JSX>);` and convert it to template.
    fn convert(jsx: &str) -> Option<String> {
        convert_with_args(jsx, Some("args"))
    }

    fn convert_with_args(jsx: &str, render_args_name: Option<&str>) -> Option<String> {
        let mut source = String::from("const x = (");
        source.push_str(jsx);
        source.push_str(");\n");

        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::tsx()).parse();
        assert!(!parsed.panicked, "fixture should parse: {source}");

        let Some(Statement::VariableDeclaration(decl)) = parsed.program.body.first() else {
            panic!("expected variable declaration");
        };
        let init = decl.declarations[0]
            .init
            .as_ref()
            .expect("declarator initializer");
        let expr = match init {
            Expression::ParenthesizedExpression(inner) => &inner.expression,
            other => other,
        };
        convert_render(expr, &source, render_args_name)
    }

    #[test]
    fn converts_element_with_string_attr_and_text() {
        assert_eq!(
            convert(r#"<AfButton color="primary">Primary</AfButton>"#).as_deref(),
            Some(r#"<AfButton color="primary">Primary</AfButton>"#)
        );
    }

    #[test]
    fn converts_self_closing_element() {
        assert_eq!(convert("<AfButton />").as_deref(), Some("<AfButton />"));
    }

    #[test]
    fn converts_expression_attr_and_bare_attr() {
        assert_eq!(
            convert("<AfButton count={1 + 2} disabled />").as_deref(),
            Some(r#"<AfButton :count="1 + 2" disabled />"#)
        );
    }

    #[test]
    fn converts_spread_attr_to_v_bind() {
        assert_eq!(
            convert("<AfButton {...args} />").as_deref(),
            Some(r#"<AfButton v-bind="args" />"#)
        );
        assert_eq!(
            convert_with_args("<AfButton {...props} />", Some("props")).as_deref(),
            Some(r#"<AfButton v-bind="props" />"#)
        );
    }

    #[test]
    fn rejects_non_render_args_spread_attr() {
        assert_eq!(convert("<AfButton {...props} />").as_deref(), None);
    }

    #[test]
    fn converts_expression_child_to_mustache() {
        assert_eq!(
            convert("<AfButton>{label}</AfButton>").as_deref(),
            Some("<AfButton>{{ label }}</AfButton>")
        );
    }

    #[test]
    fn fragment_emits_children_without_wrapper() {
        assert_eq!(
            convert("<><AfButton a=\"x\" /><AfButton b=\"y\" /></>").as_deref(),
            Some(r#"<AfButton a="x" /><AfButton b="y" />"#)
        );
    }

    #[test]
    fn converts_dotted_member_element_name() {
        assert_eq!(
            convert("<Form.Item label=\"name\" />").as_deref(),
            Some(r#"<Form.Item label="name" />"#)
        );
    }

    #[test]
    fn escapes_quotes_in_expression_attr() {
        // An expression attribute whose source contains `"` must be HTML-escaped so
        // it cannot break out of the double-quoted Vue binding.
        assert_eq!(
            convert(r#"<AfButton onClick={() => alert("hi")} />"#).as_deref(),
            Some(r#"<AfButton :onClick="() => alert(&quot;hi&quot;)" />"#)
        );
    }

    #[test]
    fn rejects_identifier_attribute_without_script_binding() {
        assert_eq!(convert("<AfButton data={schema} />").as_deref(), None);
    }

    #[test]
    fn rejects_render_args_member_attribute() {
        assert_eq!(convert("<AfButton value={args.value} />").as_deref(), None);
        assert_eq!(
            convert_with_args("<AfButton value={props.value} />", Some("props")).as_deref(),
            None
        );
    }

    #[test]
    fn rejects_tsx_slot_object_children() {
        assert_eq!(
            convert("<AfTooltip>{{ activator: () => <AfButton>Hover</AfButton> }}</AfTooltip>")
                .as_deref(),
            None
        );
    }

    #[test]
    fn non_jsx_render_returns_none() {
        assert_eq!(convert("42").as_deref(), None);
    }
}
