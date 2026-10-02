use super::{MarkupBinding, MarkupBindingInner, jsx_attribute_ref};
use crate::rules::a11y::helpers::string_literal_value;
use oxc_ast::ast::{JSXAttributeValue, JSXExpression};

impl<'a> MarkupBinding<'a> {
    /// A written or expression-valued string literal, without changing
    /// `static_value`'s distinction between attributes and dynamic bindings.
    pub fn literal_string_value(&self) -> Option<&'a str> {
        if let Some(value) = self.static_value() {
            return Some(value);
        }
        if let MarkupBindingInner::Jsx { node, .. } = self.inner
            && let Some(JSXAttributeValue::ExpressionContainer(container)) =
                &jsx_attribute_ref(node).value
            && let JSXExpression::StringLiteral(value) = &container.expression
        {
            return Some(value.value.as_str());
        }
        self.expression().and_then(string_literal_value)
    }
}
