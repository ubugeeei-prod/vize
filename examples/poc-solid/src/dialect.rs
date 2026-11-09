//! Solid's JSX dialect, lowered to neutral L2 ops.
//!
//! Lowercase tags are elements and capitalized tags are components. On an
//! element, `on*` attributes are event bindings and other expression attributes
//! are one-way bindings; on a component every attribute is a prop. Expression
//! children are JSX child values, which a Solid runtime inserts as they change.

use oxc_ast::ast::{
    JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement, JSXElementName,
    JSXExpression, JSXFragment,
};
use oxc_ast_visit::Visit;
use oxc_span::GetSpan;
use poc_support::Error;
use vize_l0::{Allocator, Box, Span, String, Vec};
use vize_l2::expr::ExprRef;
use vize_l2::op::{
    Attribute, BindOp, BindingOp, ComponentOp, DynamicName, ElementOp, InterpolationOp, Namespace,
    OnOp, Op, Region, TextOp,
};

/// Each outermost JSX element of a module as one L2 root region.
pub(crate) struct Roots<'a> {
    allocator: &'a Allocator,
    source: &'a str,
    pub(crate) roots: std::vec::Vec<(Span, Region<'a>)>,
    pub(crate) error: Option<Error>,
}

impl<'a> Roots<'a> {
    pub(crate) fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self {
            allocator,
            source,
            roots: std::vec::Vec::new(),
            error: None,
        }
    }

    fn element(&self, element: &JSXElement<'a>) -> Result<Op<'a>, Error> {
        let opening = &element.opening_element;
        let span = span(element.span);
        let children = self.children(&element.children)?;
        let mut attributes = Vec::new_in(&self.allocator);
        let mut bindings = Vec::new_in(&self.allocator);
        match &opening.name {
            JSXElementName::Identifier(tag) => {
                for item in &opening.attributes {
                    self.attribute(item, true, &mut attributes, &mut bindings)?;
                }
                Ok(Op::Element(Box::new_in(
                    ElementOp {
                        tag: tag.name.as_str(),
                        namespace: Namespace::Html,
                        attributes,
                        bindings,
                        children,
                        span,
                    },
                    &self.allocator,
                )))
            }
            JSXElementName::IdentifierReference(name) => {
                for item in &opening.attributes {
                    self.attribute(item, false, &mut attributes, &mut bindings)?;
                }
                Ok(Op::Component(Box::new_in(
                    ComponentOp {
                        name: name.name.as_str(),
                        attributes,
                        bindings,
                        children,
                        span,
                    },
                    &self.allocator,
                )))
            }
            _ => Err(Error::new("unsupported tag name", span)),
        }
    }

    fn attribute(
        &self,
        item: &JSXAttributeItem<'a>,
        element: bool,
        attributes: &mut Vec<'a, Attribute<'a>>,
        bindings: &mut Vec<'a, BindingOp<'a>>,
    ) -> Result<(), Error> {
        let JSXAttributeItem::Attribute(attribute) = item else {
            return Err(Error::new(
                "unsupported spread attribute",
                span(item.span()),
            ));
        };
        let span = span(attribute.span);
        let JSXAttributeName::Identifier(name) = &attribute.name else {
            return Err(Error::new("unsupported namespaced attribute", span));
        };
        let name = name.name.as_str();
        let container = match &attribute.value {
            None => {
                attributes.push(Attribute {
                    name,
                    value: None,
                    span,
                });
                return Ok(());
            }
            Some(JSXAttributeValue::StringLiteral(value)) => {
                attributes.push(Attribute {
                    name,
                    value: Some(value.value.as_str()),
                    span,
                });
                return Ok(());
            }
            Some(JSXAttributeValue::ExpressionContainer(container)) => container,
            Some(_) => return Err(Error::new("unsupported attribute value", span)),
        };
        let value = self.expression(&container.expression)?;
        let modifiers = Vec::new_in(&self.allocator);
        let binding = match event_name(name).filter(|_| element) {
            Some(event) => BindingOp::On(Box::new_in(
                OnOp {
                    name: Some(DynamicName::Static(self.allocator.alloc_str(&event))),
                    modifiers,
                    handler: Some(value.into()),
                    span,
                },
                &self.allocator,
            )),
            None => BindingOp::Bind(Box::new_in(
                BindOp {
                    name: Some(DynamicName::Static(name)),
                    modifiers,
                    value: Some(value),
                    span,
                },
                &self.allocator,
            )),
        };
        bindings.push(binding);
        Ok(())
    }

    fn children(&self, children: &[JSXChild<'a>]) -> Result<Region<'a>, Error> {
        let mut ops = Vec::new_in(&self.allocator);
        for child in children {
            match child {
                JSXChild::Text(text) => {
                    let span = span(text.span);
                    if text.value.as_str().contains('&') {
                        return Err(Error::new("unsupported character reference", span));
                    }
                    if let Some(content) = jsx_text(text.value.as_str()) {
                        ops.push(Op::Text(Box::new_in(
                            TextOp {
                                content: self.allocator.alloc_str(&content),
                                span,
                            },
                            &self.allocator,
                        )));
                    }
                }
                JSXChild::ExpressionContainer(container) => {
                    if matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                        continue;
                    }
                    ops.push(Op::JsxChildValue(Box::new_in(
                        InterpolationOp {
                            expression: self.expression(&container.expression)?,
                            raw_html: false,
                            span: span(container.span),
                        },
                        &self.allocator,
                    )));
                }
                JSXChild::Element(element) => ops.push(self.element(element)?),
                JSXChild::Fragment(fragment) => return Err(fragment_error(fragment)),
                JSXChild::Spread(spread) => {
                    return Err(Error::new("unsupported spread child", span(spread.span)));
                }
            }
        }
        Ok(Region { ops })
    }

    fn expression(&self, expression: &JSXExpression<'a>) -> Result<ExprRef<'a>, Error> {
        let span = span(expression.span());
        let text = self
            .source
            .get(span.start as usize..span.end as usize)
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| Error::new("empty expression", span))?;
        Ok(ExprRef::parse_js_in(self.allocator, text, span))
    }
}

impl<'a> Visit<'a> for Roots<'a> {
    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        if self.error.is_some() {
            return;
        }
        // Nested JSX belongs to this root's lowering, so the walk stops here.
        match self.element(element) {
            Ok(op) => {
                let mut ops = Vec::new_in(&self.allocator);
                ops.push(op);
                self.roots.push((span(element.span), Region { ops }));
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) {
        self.error.get_or_insert_with(|| fragment_error(fragment));
    }
}

fn fragment_error(fragment: &JSXFragment<'_>) -> Error {
    Error::new("unsupported fragment", span(fragment.span))
}

fn span(span: oxc_span::Span) -> Span {
    Span::new(span.start, span.end)
}

/// `onClick` -> `click`.
fn event_name(name: &str) -> Option<String> {
    let rest = name.strip_prefix("on")?;
    rest.starts_with(|first: char| first.is_ascii_uppercase())
        .then(|| String::from(rest.to_ascii_lowercase()))
}

/// JSX text whitespace: lines are trimmed where they meet a line break,
/// blank lines disappear, and the remaining lines join with one space.
fn jsx_text(raw: &str) -> Option<String> {
    let last = raw.split('\n').count().saturating_sub(1);
    let lines: std::vec::Vec<&str> = raw
        .split('\n')
        .enumerate()
        .map(|(index, line)| {
            let line = if index > 0 { line.trim_start() } else { line };
            if index < last { line.trim_end() } else { line }
        })
        .filter(|line| !line.is_empty())
        .collect();
    (!lines.is_empty()).then(|| String::from(lines.join(" ")))
}

#[cfg(test)]
mod tests {
    use super::{event_name, jsx_text};

    #[test]
    fn jsx_text_follows_the_jsx_whitespace_rule() {
        assert_eq!(jsx_text("\n        Clicked ").as_deref(), Some("Clicked "));
        assert_eq!(jsx_text(" times\n      ").as_deref(), Some(" times"));
        assert_eq!(jsx_text("a\n  b\n").as_deref(), Some("a b"));
        assert_eq!(jsx_text("\n      ").as_deref(), None);
    }

    #[test]
    fn only_on_prefixed_camel_case_names_are_events() {
        assert_eq!(event_name("onClick").as_deref(), Some("click"));
        assert_eq!(event_name("once"), None);
        assert_eq!(event_name("class"), None);
    }
}
