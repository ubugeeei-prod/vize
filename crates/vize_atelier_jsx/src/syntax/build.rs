use oxc_allocator::Allocator;
use oxc_ast::ast::{
    JSXAttributeItem, JSXAttributeValue, JSXChild, JSXElement, JSXElementName, JSXExpression,
    JSXExpressionContainer, JSXFragment,
};
use oxc_span::{GetSpan, Span};

use super::{
    JsxSyntaxAttribute, JsxSyntaxAttributeValue, JsxSyntaxElement, JsxSyntaxExpression,
    JsxSyntaxNode, JsxSyntaxSnapshot,
};
use crate::{JsxLang, parse};
use vize_atlas::Shared;

pub(super) fn snapshot(
    filename: Option<Box<str>>,
    source: &str,
    lang: JsxLang,
) -> JsxSyntaxSnapshot {
    let allocator = Allocator::default();
    let parser_source = parse::prepare_source_for_parse(source, lang);
    let parsed = parse::parse_module(&allocator, parser_source.as_ref(), lang);
    let collected = super::roots::collect(source, &parsed.program);
    #[cfg(test)]
    super::typecheck::record_lowering();
    let typecheck_roots = super::typecheck::project_roots(&collected.roots, &collected.metadata);
    let mut diagnostics = parsed.diagnostics;
    diagnostics.extend(collected.diagnostics);
    let analysis = Shared::new(crate::analyze_jsx_program(&parsed.program, source));
    JsxSyntaxSnapshot {
        filename,
        source: source.into(),
        lang,
        roots: collected.roots,
        root_metadata: collected.metadata,
        typecheck_roots,
        diagnostics,
        panicked: parsed.panicked,
        source_anchor: None,
        analysis,
    }
}

pub(super) struct SyntaxBuilder<'s> {
    pub(super) source: &'s str,
}

impl<'s> SyntaxBuilder<'s> {
    pub(super) fn new(source: &'s str) -> Self {
        Self { source }
    }

    pub(super) fn slice(&self, span: Span) -> &'s str {
        let start = (span.start as usize).min(self.source.len());
        let end = (span.end as usize).min(self.source.len()).max(start);
        &self.source[start..end]
    }

    pub(super) fn expression(&self, span: Span) -> JsxSyntaxExpression {
        JsxSyntaxExpression::authored(self.slice(span), span)
    }

    pub(super) fn element(&self, element: &JSXElement<'_>) -> JsxSyntaxNode {
        let opening = &element.opening_element;
        JsxSyntaxNode::Element(JsxSyntaxElement {
            name: self.slice(opening.name.span()).into(),
            component: is_component(&opening.name),
            attributes: opening
                .attributes
                .iter()
                .map(|attribute| self.attribute(attribute))
                .collect(),
            children: self.children(&element.children),
            span: element.span.into(),
        })
    }

    pub(super) fn fragment(&self, fragment: &JSXFragment<'_>) -> JsxSyntaxNode {
        JsxSyntaxNode::Fragment {
            children: self.children(&fragment.children),
            span: fragment.span.into(),
        }
    }

    fn children(&self, children: &[JSXChild<'_>]) -> Vec<JsxSyntaxNode> {
        children
            .iter()
            .filter_map(|child| self.child(child))
            .collect()
    }

    fn child(&self, child: &JSXChild<'_>) -> Option<JsxSyntaxNode> {
        match child {
            JSXChild::Text(text) => {
                let value = super::text::clean_jsx_text(text.value.as_str());
                (!value.is_empty()).then(|| JsxSyntaxNode::Text {
                    value: value.as_str().into(),
                    span: text.span.into(),
                })
            }
            JSXChild::Element(element) => Some(self.element(element)),
            JSXChild::Fragment(fragment) => Some(self.fragment(fragment)),
            JSXChild::ExpressionContainer(container) => self.container(container),
            JSXChild::Spread(spread) => Some(JsxSyntaxNode::Expression {
                expression: self.expression(spread.expression.span()),
                span: spread.span.into(),
            }),
        }
    }

    fn container(&self, container: &JSXExpressionContainer<'_>) -> Option<JsxSyntaxNode> {
        match &container.expression {
            JSXExpression::EmptyExpression(_) => self.comment(container.span),
            JSXExpression::StringLiteral(string) => Some(JsxSyntaxNode::Text {
                value: string.value.as_str().into(),
                span: string.span.into(),
            }),
            expression => self
                .render_expression(expression.as_expression()?)
                .or_else(|| {
                    let span = expression.span();
                    Some(JsxSyntaxNode::Expression {
                        expression: self.expression(span),
                        span: container.span.into(),
                    })
                }),
        }
    }

    fn comment(&self, span: Span) -> Option<JsxSyntaxNode> {
        let raw = self.slice(span);
        let inner = raw.strip_prefix('{')?.strip_suffix('}')?.trim();
        let value = inner
            .strip_prefix("/*")
            .and_then(|value| value.strip_suffix("*/"))
            .or_else(|| inner.strip_prefix("//"))?
            .trim();
        Some(JsxSyntaxNode::Comment {
            value: value.into(),
            span: span.into(),
        })
    }

    fn attribute(&self, item: &JSXAttributeItem<'_>) -> JsxSyntaxAttribute {
        match item {
            JSXAttributeItem::SpreadAttribute(spread) => JsxSyntaxAttribute::Spread {
                expression: self.expression(spread.argument.span()),
                span: spread.span.into(),
            },
            JSXAttributeItem::Attribute(attribute) => {
                let name_span = attribute.name.span();
                JsxSyntaxAttribute::Attribute {
                    name: self.slice(name_span).into(),
                    name_span: name_span.into(),
                    value: self.attribute_value(attribute.value.as_ref()),
                    span: attribute.span.into(),
                }
            }
        }
    }

    fn attribute_value(&self, value: Option<&JSXAttributeValue<'_>>) -> JsxSyntaxAttributeValue {
        match value {
            None => JsxSyntaxAttributeValue::Presence,
            Some(JSXAttributeValue::StringLiteral(string)) => JsxSyntaxAttributeValue::Static {
                value: string.value.as_str().into(),
                span: string.span.into(),
            },
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                match &container.expression {
                    JSXExpression::EmptyExpression(_) => JsxSyntaxAttributeValue::Presence,
                    expression => expression
                        .as_expression()
                        .map(|expression| {
                            JsxSyntaxAttributeValue::Expression(self.expression(expression.span()))
                        })
                        .unwrap_or(JsxSyntaxAttributeValue::Presence),
                }
            }
            Some(JSXAttributeValue::Element(element)) => {
                JsxSyntaxAttributeValue::Expression(self.expression(element.span))
            }
            Some(JSXAttributeValue::Fragment(fragment)) => {
                JsxSyntaxAttributeValue::Expression(self.expression(fragment.span))
            }
        }
    }
}

fn is_component(name: &JSXElementName<'_>) -> bool {
    match name {
        JSXElementName::Identifier(identifier) => !is_intrinsic(identifier.name.as_str()),
        JSXElementName::IdentifierReference(reference) => !is_intrinsic(reference.name.as_str()),
        JSXElementName::NamespacedName(name) => !is_intrinsic(name.name.name.as_str()),
        JSXElementName::MemberExpression(_) | JSXElementName::ThisExpression(_) => true,
    }
}

fn is_intrinsic(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|character| character.is_ascii_lowercase())
}
