//! Attribute expression projection; preserved during native AST retention.

use super::{GetSpan, JSXAttributeValue, Lowerer, container_expr_span};

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    pub(super) fn directive_value_expr(
        &self,
        value: Option<&JSXAttributeValue<'_>>,
    ) -> Option<vize_relief::ExpressionNode<'a>> {
        match value? {
            JSXAttributeValue::StringLiteral(string) => {
                Some(self.static_expr(self.bump().alloc_str(string.value.as_str()), string.span))
            }
            JSXAttributeValue::ExpressionContainer(container) => {
                container_expr_span(container).map(|span| self.dyn_expr(span))
            }
            JSXAttributeValue::Element(element) => Some(self.dyn_expr(element.span())),
            JSXAttributeValue::Fragment(fragment) => Some(self.dyn_expr(fragment.span())),
        }
    }

    pub(super) fn retain_typecheck_attribute_value(
        &mut self,
        value: Option<&JSXAttributeValue<'_>>,
    ) {
        match value {
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                if let Some(expression) = container.expression.as_expression() {
                    self.retain_nested_typecheck_roots(expression);
                }
            }
            Some(JSXAttributeValue::Element(element)) => self.retain_typecheck_element(element),
            Some(JSXAttributeValue::Fragment(fragment)) => self.retain_typecheck_fragment(fragment),
            _ => {}
        }
    }
}
