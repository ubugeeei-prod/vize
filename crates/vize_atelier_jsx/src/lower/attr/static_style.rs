//! Consume the original JSX expression while its parser arena is alive.

use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue};
use oxc_span::{GetSpan, Span};
use vize_relief::{AttributeNode, PropNode, TextNode, literal_object_style};

use super::Lowerer;

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    pub(super) fn literal_style_attribute(
        &self,
        items: &[JSXAttributeItem<'_>],
    ) -> Option<(Span, PropNode<'a>)> {
        let mut selected = None;
        for item in items {
            let JSXAttributeItem::Attribute(attribute) = item else {
                return None;
            };
            let name = match &attribute.name {
                JSXAttributeName::Identifier(name) => name.name.as_str(),
                JSXAttributeName::NamespacedName(name) => {
                    if name.namespace.name == "v-bind" {
                        return None;
                    }
                    continue;
                }
            };
            if name == "v-bind" {
                return None;
            }
            if name.eq_ignore_ascii_case("style") {
                if name != "style" || selected.is_some() {
                    return None;
                }
                selected = Some(attribute);
            }
        }
        let attribute = selected?;
        let JSXAttributeValue::ExpressionContainer(container) = attribute.value.as_ref()? else {
            return None;
        };
        let expression = container.expression.as_expression()?;
        let css = literal_object_style(expression)?;
        let value = TextNode::new(
            self.bump().alloc_str(&css),
            self.mapper().location(expression.span()),
        );
        let property = AttributeNode {
            name: "style",
            name_loc: self.mapper().location(attribute.name.span()),
            value: Some(value),
            loc: self.mapper().location(attribute.span),
        };
        Some((attribute.span, PropNode::Attribute(self.boxed(property))))
    }
}
