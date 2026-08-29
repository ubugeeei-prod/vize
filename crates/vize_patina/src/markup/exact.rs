use super::{
    MarkupBinding, MarkupBindingInner, MarkupBindingKind, MarkupElement, MarkupElementInner,
    jsx_attribute_arg_name, jsx_attribute_binding_kind, jsx_attribute_ref, jsx_element_ref,
    relief_directive_kind,
};
use oxc_ast::ast::{JSXAttributeName, JSXElementName};
use vize_relief::ExpressionNode;

impl<'a> MarkupElement<'a> {
    /// Check whether this element is an unqualified tag with an exact name.
    ///
    /// This deliberately does not match JSX member or namespaced tags, even when
    /// their local/property name equals `expected`.
    pub fn is_unqualified_tag_exact(&self, expected: &str) -> bool {
        match self.inner {
            MarkupElementInner::Relief(node) => node.tag == expected,
            MarkupElementInner::JsxElement { node, .. } => {
                match &jsx_element_ref(node).opening_element.name {
                    JSXElementName::Identifier(identifier) => identifier.name.as_str() == expected,
                    JSXElementName::IdentifierReference(reference) => {
                        reference.name.as_str() == expected
                    }
                    JSXElementName::NamespacedName(_)
                    | JSXElementName::MemberExpression(_)
                    | JSXElementName::ThisExpression(_) => false,
                }
            }
            MarkupElementInner::JsxFragment { .. } => false,
        }
    }
}

impl<'a> MarkupBinding<'a> {
    /// Whether this binding is an unqualified prop/attribute with an exact name.
    ///
    /// Unlike [`Self::arg_name_eq`], this is case-sensitive and does not collapse
    /// JSX namespace attributes such as `foo:class` into their local name.
    pub fn is_unqualified_arg_exact(&self, expected: &str) -> bool {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(node) => node.name == expected,
            MarkupBindingInner::ReliefDirective(node) => match relief_directive_kind(node) {
                MarkupBindingKind::Custom => node.name == expected,
                _ => match node.arg.as_ref() {
                    Some(ExpressionNode::Simple(simple)) => simple.content == expected,
                    _ => false,
                },
            },
            MarkupBindingInner::Jsx { node, .. } => {
                let attr = jsx_attribute_ref(node);
                match &attr.name {
                    JSXAttributeName::Identifier(identifier) => {
                        match jsx_attribute_binding_kind(attr) {
                            MarkupBindingKind::On => jsx_attribute_arg_name(attr) == Some(expected),
                            _ => identifier.name.as_str() == expected,
                        }
                    }
                    JSXAttributeName::NamespacedName(_) => false,
                }
            }
        }
    }
}
